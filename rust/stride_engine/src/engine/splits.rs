//! Lap and split engine (spec section 17).
//!
//! Wraps [`crate::engine::distance::SplitBoundaryTracker`] with the extra
//! bookkeeping needed to close out a full [`WorkoutSplit`] record (pace,
//! speed, elevation, heart rate, cadence, calories) whenever a boundary is
//! crossed, plus support for manual laps.

use crate::engine::calories::{self, CalorieInputs, CalorieMethod};
use crate::engine::distance::SplitBoundaryTracker;
use crate::engine::pace::pace_sec_per_km;
use crate::engine::speed::safe_average_speed;
use crate::models::{ActivityType, WorkoutSplit};

/// Running accumulators for the split currently in progress.
#[derive(Debug, Clone, Copy, Default)]
struct InProgressSplit {
    start_distance_m: f64,
    start_time_ms: i64,
    start_elevation_gain_m: f64,
    start_elevation_loss_m: f64,
    hr_sum: f64,
    hr_samples: u64,
    cadence_sum: f64,
    cadence_samples: u64,
}

pub struct SplitEngine {
    boundary_tracker: SplitBoundaryTracker,
    in_progress: InProgressSplit,
    completed: Vec<WorkoutSplit>,
    next_split_number: u32,
}

impl SplitEngine {
    pub fn new(split_distance_meters: f64, workout_start_ms: i64) -> Self {
        Self {
            boundary_tracker: SplitBoundaryTracker::new(split_distance_meters),
            in_progress: InProgressSplit {
                start_time_ms: workout_start_ms,
                ..Default::default()
            },
            completed: Vec::new(),
            next_split_number: 1,
        }
    }

    pub fn completed_splits(&self) -> &[WorkoutSplit] {
        &self.completed
    }

    /// Records a live heart-rate / cadence sample for averaging into the
    /// split currently in progress.
    pub fn observe_live_metrics(&mut self, hr_bpm: Option<f64>, cadence_spm: Option<f64>) {
        if let Some(hr) = hr_bpm {
            self.in_progress.hr_sum += hr;
            self.in_progress.hr_samples += 1;
        }
        if let Some(cad) = cadence_spm {
            self.in_progress.cadence_sum += cad;
            self.in_progress.cadence_samples += 1;
        }
    }

    /// Call after each distance update. Automatically closes out any
    /// splits whose boundary was just crossed. Returns the newly completed
    /// splits (usually 0 or 1).
    pub fn update(
        &mut self,
        now_ms: i64,
        cumulative_distance_m: f64,
        cumulative_elevation_gain_m: f64,
        cumulative_elevation_loss_m: f64,
        weight_kg: f64,
        activity_type: ActivityType,
    ) -> Vec<WorkoutSplit> {
        let crossed = self.boundary_tracker.check(cumulative_distance_m);
        let mut newly_completed = Vec::new();

        for _ in crossed {
            let split = self.finalize_split(
                now_ms,
                cumulative_distance_m,
                cumulative_elevation_gain_m,
                cumulative_elevation_loss_m,
                weight_kg,
                activity_type,
            );
            newly_completed.push(split.clone());
            self.completed.push(split);
        }

        newly_completed
    }

    /// Manually closes the current split early (spec: "Manual lap button"),
    /// regardless of distance boundary.
    pub fn manual_lap(
        &mut self,
        now_ms: i64,
        cumulative_distance_m: f64,
        cumulative_elevation_gain_m: f64,
        cumulative_elevation_loss_m: f64,
        weight_kg: f64,
        activity_type: ActivityType,
    ) -> WorkoutSplit {
        let split = self.finalize_split(
            now_ms,
            cumulative_distance_m,
            cumulative_elevation_gain_m,
            cumulative_elevation_loss_m,
            weight_kg,
            activity_type,
        );
        self.completed.push(split.clone());
        split
    }

    fn finalize_split(
        &mut self,
        now_ms: i64,
        cumulative_distance_m: f64,
        cumulative_elevation_gain_m: f64,
        cumulative_elevation_loss_m: f64,
        weight_kg: f64,
        activity_type: ActivityType,
    ) -> WorkoutSplit {
        let distance = (cumulative_distance_m - self.in_progress.start_distance_m).max(0.0);
        let duration_ms = (now_ms - self.in_progress.start_time_ms).max(0);
        let elevation_gain =
            (cumulative_elevation_gain_m - self.in_progress.start_elevation_gain_m).max(0.0);
        let elevation_loss =
            (cumulative_elevation_loss_m - self.in_progress.start_elevation_loss_m).max(0.0);

        let avg_speed = safe_average_speed(distance, duration_ms);
        let avg_pace = pace_sec_per_km(duration_ms, distance);

        let avg_hr = if self.in_progress.hr_samples > 0 {
            Some(self.in_progress.hr_sum / self.in_progress.hr_samples as f64)
        } else {
            None
        };
        let avg_cadence = if self.in_progress.cadence_samples > 0 {
            self.in_progress.cadence_sum / self.in_progress.cadence_samples as f64
        } else {
            0.0
        };

        let calorie_inputs = CalorieInputs {
            wearable_kcal: None,
            weight_kg: Some(weight_kg),
            duration_ms,
            distance_meters: distance,
            average_speed_mps: avg_speed,
            average_heart_rate_bpm: avg_hr,
            age_years: None,
            is_male: None,
            activity_type,
            elevation_gain_meters: elevation_gain,
        };
        let calorie_result = calories::estimate(&calorie_inputs);
        let _ = CalorieMethod::Met; // documents intended default path in tests

        let split = WorkoutSplit {
            split_number: self.next_split_number,
            distance_meters: distance,
            duration_ms,
            average_pace_sec_per_km: avg_pace,
            average_speed_mps: avg_speed,
            elevation_gain_meters: elevation_gain,
            elevation_loss_meters: elevation_loss,
            average_heart_rate_bpm: avg_hr,
            average_cadence_spm: avg_cadence,
            calories_estimated: calorie_result.kcal,
        };

        self.next_split_number += 1;
        self.in_progress = InProgressSplit {
            start_distance_m: cumulative_distance_m,
            start_time_ms: now_ms,
            start_elevation_gain_m: cumulative_elevation_gain_m,
            start_elevation_loss_m: cumulative_elevation_loss_m,
            ..Default::default()
        };

        split
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::distance::METERS_PER_KILOMETER;

    #[test]
    fn no_split_before_boundary_crossed() {
        let mut e = SplitEngine::new(METERS_PER_KILOMETER, 0);
        let out = e.update(60_000, 500.0, 0.0, 0.0, 70.0, ActivityType::Walk);
        assert!(out.is_empty());
    }

    #[test]
    fn split_created_on_boundary_cross() {
        let mut e = SplitEngine::new(METERS_PER_KILOMETER, 0);
        let out = e.update(6 * 60 * 1000, 1000.0, 5.0, 2.0, 70.0, ActivityType::Walk);
        assert_eq!(out.len(), 1);
        let split = &out[0];
        assert_eq!(split.split_number, 1);
        assert!((split.distance_meters - 1000.0).abs() < 1e-6);
        assert!(split.duration_ms == 6 * 60 * 1000);
        assert!(split.calories_estimated > 0.0);
    }

    #[test]
    fn second_split_measures_delta_not_cumulative() {
        let mut e = SplitEngine::new(METERS_PER_KILOMETER, 0);
        e.update(6 * 60 * 1000, 1000.0, 0.0, 0.0, 70.0, ActivityType::Walk);
        let out = e.update(11 * 60 * 1000, 2000.0, 0.0, 0.0, 70.0, ActivityType::Walk);
        assert_eq!(out.len(), 1);
        assert!((out[0].distance_meters - 1000.0).abs() < 1e-6);
        assert_eq!(out[0].duration_ms, 5 * 60 * 1000);
    }

    #[test]
    fn manual_lap_closes_early_regardless_of_distance() {
        let mut e = SplitEngine::new(METERS_PER_KILOMETER, 0);
        let split = e.manual_lap(90_000, 400.0, 0.0, 0.0, 70.0, ActivityType::Run);
        assert!((split.distance_meters - 400.0).abs() < 1e-6);
        assert_eq!(e.completed_splits().len(), 1);
    }

    #[test]
    fn live_metrics_averaged_into_next_closed_split() {
        let mut e = SplitEngine::new(METERS_PER_KILOMETER, 0);
        e.observe_live_metrics(Some(140.0), Some(160.0));
        e.observe_live_metrics(Some(150.0), Some(170.0));
        let out = e.update(6 * 60 * 1000, 1000.0, 0.0, 0.0, 70.0, ActivityType::Walk);
        assert!((out[0].average_heart_rate_bpm.unwrap() - 145.0).abs() < 1e-6);
        assert!((out[0].average_cadence_spm - 165.0).abs() < 1e-6);
    }
}
