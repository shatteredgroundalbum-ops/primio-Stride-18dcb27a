//! Live coaching event generator (spec section 19).
//!
//! Produces structured [`CoachingEvent`] values from state transitions the
//! session controller observes. This is pure/local — it never depends on
//! an AI service being reachable. The Dart layer can render these as
//! on-screen banners, trigger local voice-coaching announcements, or (when
//! online) forward them to a cloud coach for richer natural-language
//! commentary — but the *decision* that something coaching-worthy just
//! happened is made entirely offline, here.

use serde_json::json;

use crate::models::{CoachingEvent, CoachingEventType, EpochMillis};

pub struct CoachingEventBuilder {
    workout_id: String,
}

impl CoachingEventBuilder {
    pub fn new(workout_id: impl Into<String>) -> Self {
        Self {
            workout_id: workout_id.into(),
        }
    }

    pub fn workout_started(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::WorkoutStarted, at)
    }

    pub fn first_kilometer_completed(&self, at: EpochMillis, pace_sec_per_km: f64) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::FirstKilometerCompleted, at)
            .with_data(json!({ "pace_sec_per_km": pace_sec_per_km }))
    }

    pub fn first_mile_completed(&self, at: EpochMillis, pace_sec_per_km: f64) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::FirstMileCompleted, at)
            .with_data(json!({ "pace_sec_per_km": pace_sec_per_km }))
    }

    pub fn halfway_point(&self, at: EpochMillis, remaining_meters: f64) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::HalfwayPoint, at)
            .with_data(json!({ "remaining_meters": remaining_meters }))
    }

    pub fn pace_below_target(&self, at: EpochMillis, current: f64, target: f64) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::PaceBelowTarget, at)
            .with_data(json!({ "current_sec_per_km": current, "target_sec_per_km": target }))
    }

    pub fn pace_above_target(&self, at: EpochMillis, current: f64, target: f64) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::PaceAboveTarget, at)
            .with_data(json!({ "current_sec_per_km": current, "target_sec_per_km": target }))
    }

    pub fn auto_pause_activated(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::AutoPauseActivated, at)
    }

    pub fn auto_resume_activated(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::AutoResumeActivated, at)
    }

    pub fn gps_quality_poor(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::GpsQualityPoor, at)
    }

    pub fn gps_quality_restored(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::GpsQualityRestored, at)
    }

    pub fn heart_rate_unavailable(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::HeartRateUnavailable, at)
    }

    pub fn goal_completed(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::GoalCompleted, at)
    }

    pub fn personal_record_possible(&self, at: EpochMillis, record_type: &str) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::PersonalRecordPossible, at)
            .with_data(json!({ "record_type": record_type }))
    }

    pub fn workout_ending(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::WorkoutEnding, at)
    }

    pub fn split_completed(&self, at: EpochMillis, split_number: u32, pace_sec_per_km: f64) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::SplitCompleted, at)
            .with_data(json!({ "split_number": split_number, "pace_sec_per_km": pace_sec_per_km }))
    }

    pub fn vehicle_motion_suspected(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::VehicleMotionSuspected, at)
    }

    pub fn hydration_reminder(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::HydrationReminder, at)
    }

    pub fn user_stopped(&self, at: EpochMillis) -> CoachingEvent {
        CoachingEvent::new(&self.workout_id, CoachingEventType::UserStopped, at)
    }
}

/// A simple pace-target comparator with a dead-band to avoid firing
/// PaceAboveTarget/PaceBelowTarget events on every single tick when the
/// user is hovering right around the target.
pub fn pace_target_deviation(
    current_sec_per_km: f64,
    target_sec_per_km: f64,
    dead_band_sec: f64,
) -> Option<bool> {
    if current_sec_per_km <= 0.0 || target_sec_per_km <= 0.0 {
        return None;
    }
    let diff = current_sec_per_km - target_sec_per_km;
    if diff.abs() <= dead_band_sec {
        None
    } else {
        // Lower sec/km = faster. Positive diff means current pace is
        // slower (higher sec/km) than target => "below target" pace.
        Some(diff > 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_tags_all_events_with_workout_id() {
        let b = CoachingEventBuilder::new("w1");
        let e = b.workout_started(1000);
        assert_eq!(e.workout_id, "w1");
        assert_eq!(e.event_type, CoachingEventType::WorkoutStarted);
    }

    #[test]
    fn pace_deviation_within_dead_band_is_none() {
        let result = pace_target_deviation(300.0, 305.0, 10.0);
        assert_eq!(result, None);
    }

    #[test]
    fn pace_slower_than_target_is_below() {
        // Higher sec/km = slower pace = "below target"
        let result = pace_target_deviation(330.0, 300.0, 10.0);
        assert_eq!(result, Some(true));
    }

    #[test]
    fn pace_faster_than_target_is_above() {
        let result = pace_target_deviation(270.0, 300.0, 10.0);
        assert_eq!(result, Some(false));
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert_eq!(pace_target_deviation(0.0, 300.0, 10.0), None);
        assert_eq!(pace_target_deviation(300.0, 0.0, 10.0), None);
    }
}
