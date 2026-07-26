//! Personal record engine (spec section 28).
//!
//! Pure function(s) comparing a finalized [`WorkoutSummary`] against the
//! user's prior best-known values (supplied by the caller — the Dart side
//! is responsible for loading those from local/cloud history). Only
//! called for validated, completed workouts; the caller should never feed
//! discarded/failed workouts into this.

use serde::{Deserialize, Serialize};

use crate::models::{PersonalRecord, PersonalRecordType, WorkoutSummary};

/// The user's previous bests, as known at the time this workout finished.
/// Any field left `None` means "no prior record", so the new workout will
/// unconditionally set that record if it has a qualifying value.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PriorBests {
    pub fastest_mile_sec_per_km: Option<f64>, // stored as sec/km for uniformity
    pub fastest_kilometer_sec_per_km: Option<f64>,
    pub longest_distance_meters: Option<f64>,
    pub longest_duration_ms: Option<i64>,
    pub greatest_elevation_gain_meters: Option<f64>,
    pub highest_average_pace_sec_per_km: Option<f64>, // "highest pace" = fastest average
    pub most_calories: Option<f64>,
}

/// Evaluates a completed workout against prior bests and returns any new
/// records achieved. A workout can set multiple records simultaneously.
pub fn detect_records(summary: &WorkoutSummary, prior: &PriorBests) -> Vec<PersonalRecord> {
    let mut records = Vec::new();
    let achieved_at = summary.ended_at;
    let workout_id = summary.workout_id.clone();

    // Fastest mile / km: look at splits tagged to standard distances. We
    // conservatively use the fastest completed split's pace as a proxy
    // for "fastest mile/km", since split boundaries are already aligned
    // to km or mile depending on caller configuration; here we just look
    // at the overall fastest split pace and let the caller determine
    // which record types apply based on how splits were configured.
    if let Some(fastest_split_pace) = summary
        .splits
        .iter()
        .map(|s| s.average_pace_sec_per_km)
        .filter(|p| *p > 0.0)
        .fold(None, |acc: Option<f64>, p| match acc {
            Some(best) if best <= p => Some(best),
            _ => Some(p),
        })
    {
        if is_new_best_low(prior.fastest_kilometer_sec_per_km, fastest_split_pace) {
            records.push(PersonalRecord {
                record_type: PersonalRecordType::FastestKilometer,
                value: fastest_split_pace,
                workout_id: workout_id.clone(),
                achieved_at,
            });
        }
    }

    if is_new_best_high(prior.longest_distance_meters, summary.distance_meters) {
        records.push(PersonalRecord {
            record_type: PersonalRecordType::LongestDistance,
            value: summary.distance_meters,
            workout_id: workout_id.clone(),
            achieved_at,
        });
    }

    if is_new_best_high_i64(prior.longest_duration_ms, summary.total_duration_ms) {
        records.push(PersonalRecord {
            record_type: PersonalRecordType::LongestDuration,
            value: summary.total_duration_ms as f64,
            workout_id: workout_id.clone(),
            achieved_at,
        });
    }

    if is_new_best_high(
        prior.greatest_elevation_gain_meters,
        summary.elevation_gain_meters,
    ) {
        records.push(PersonalRecord {
            record_type: PersonalRecordType::GreatestElevationGain,
            value: summary.elevation_gain_meters,
            workout_id: workout_id.clone(),
            achieved_at,
        });
    }

    if summary.average_pace_sec_per_km > 0.0
        && is_new_best_low(
            prior.highest_average_pace_sec_per_km,
            summary.average_pace_sec_per_km,
        )
    {
        records.push(PersonalRecord {
            record_type: PersonalRecordType::HighestAveragePace,
            value: summary.average_pace_sec_per_km,
            workout_id: workout_id.clone(),
            achieved_at,
        });
    }

    if is_new_best_high(prior.most_calories, summary.calories_estimated) {
        records.push(PersonalRecord {
            record_type: PersonalRecordType::MostCalories,
            value: summary.calories_estimated,
            workout_id,
            achieved_at,
        });
    }

    records
}

fn is_new_best_high(prior: Option<f64>, candidate: f64) -> bool {
    candidate.is_finite() && candidate > 0.0 && prior.map_or(true, |p| candidate > p)
}

fn is_new_best_high_i64(prior: Option<i64>, candidate: i64) -> bool {
    candidate > 0 && prior.map_or(true, |p| candidate > p)
}

/// "Low is better" comparator, for pace (lower sec/km = faster).
fn is_new_best_low(prior: Option<f64>, candidate: f64) -> bool {
    candidate.is_finite() && candidate > 0.0 && prior.map_or(true, |p| candidate < p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ActivityType;

    fn base_summary() -> WorkoutSummary {
        WorkoutSummary {
            workout_id: "w1".into(),
            user_id: "u1".into(),
            activity_type: ActivityType::Run,
            started_at: 0,
            ended_at: 1_000_000,
            total_duration_ms: 1_800_000,
            moving_ms: 1_700_000,
            paused_ms: 100_000,
            distance_meters: 5000.0,
            average_pace_sec_per_km: 300.0,
            average_speed_mps: 3.3,
            max_speed_mps: 4.5,
            calories_estimated: 350.0,
            calorie_estimate_method: "met".into(),
            step_count: 6000,
            average_cadence_spm: 165.0,
            average_heart_rate_bpm: None,
            max_heart_rate_bpm: None,
            min_heart_rate_bpm: None,
            elevation_gain_meters: 40.0,
            elevation_loss_meters: 35.0,
            splits: vec![],
            start_latitude: None,
            start_longitude: None,
            end_latitude: None,
            end_longitude: None,
            encoded_polyline: None,
            goal_result: None,
            has_flagged_segments: false,
            validation_warnings: vec![],
            is_blocked: false,
            block_reasons: vec![],
        }
    }

    #[test]
    fn no_prior_bests_sets_all_qualifying_records() {
        let summary = base_summary();
        let prior = PriorBests::default();
        let records = detect_records(&summary, &prior);
        let types: Vec<_> = records.iter().map(|r| r.record_type).collect();
        assert!(types.contains(&PersonalRecordType::LongestDistance));
        assert!(types.contains(&PersonalRecordType::LongestDuration));
        assert!(types.contains(&PersonalRecordType::GreatestElevationGain));
        assert!(types.contains(&PersonalRecordType::HighestAveragePace));
        assert!(types.contains(&PersonalRecordType::MostCalories));
    }

    #[test]
    fn does_not_beat_existing_better_record() {
        let summary = base_summary();
        let prior = PriorBests {
            longest_distance_meters: Some(10_000.0), // already longer than this workout
            ..Default::default()
        };
        let records = detect_records(&summary, &prior);
        assert!(!records
            .iter()
            .any(|r| r.record_type == PersonalRecordType::LongestDistance));
    }

    #[test]
    fn beats_prior_record_when_better() {
        let summary = base_summary();
        let prior = PriorBests {
            longest_distance_meters: Some(2_000.0),
            ..Default::default()
        };
        let records = detect_records(&summary, &prior);
        assert!(records
            .iter()
            .any(|r| r.record_type == PersonalRecordType::LongestDistance));
    }

    #[test]
    fn lower_pace_value_is_the_better_record() {
        let summary = base_summary(); // average_pace_sec_per_km = 300.0 (5:00/km)
        let prior = PriorBests {
            highest_average_pace_sec_per_km: Some(280.0), // faster (4:40/km) prior best
            ..Default::default()
        };
        let records = detect_records(&summary, &prior);
        assert!(!records
            .iter()
            .any(|r| r.record_type == PersonalRecordType::HighestAveragePace));
    }

    #[test]
    fn zero_distance_workout_sets_no_records() {
        let mut summary = base_summary();
        summary.distance_meters = 0.0;
        summary.average_pace_sec_per_km = 0.0;
        summary.calories_estimated = 0.0;
        summary.elevation_gain_meters = 0.0;
        summary.total_duration_ms = 0;
        let prior = PriorBests::default();
        let records = detect_records(&summary, &prior);
        assert!(records.is_empty());
    }
}
