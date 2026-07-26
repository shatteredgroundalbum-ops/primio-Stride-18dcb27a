//! Walk-vs-run detection and vehicle/invalid-movement detection
//! (spec sections 9 and 12).
//!
//! Classification requires a *sustained* pattern across a rolling window
//! — never flips based on one GPS point. The user-selected activity type
//! remains authoritative unless auto-detection is explicitly enabled by
//! the caller (the Dart layer decides that policy; this module just
//! reports what the sensor data looks like).

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::models::MovementState;

#[derive(Debug, Clone, Copy)]
pub struct ClassificationSample {
    pub at_ms: i64,
    pub speed_mps: f64,
    pub cadence_spm: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClassificationThresholds {
    pub walking_max_mps: f64,
    pub brisk_walking_max_mps: f64,
    pub jogging_max_mps: f64,
    // Anything above jogging_max_mps and below vehicle_min_mps is "running".
    pub vehicle_min_mps: f64,
    /// Number of consecutive samples in the rolling window required before
    /// a classification is confirmed (guards against single-sample flips).
    pub confirmation_samples: usize,
    pub window_capacity: usize,
}

impl Default for ClassificationThresholds {
    fn default() -> Self {
        Self {
            walking_max_mps: 1.5,       // ~5.4 km/h
            brisk_walking_max_mps: 2.0, // ~7.2 km/h
            jogging_max_mps: 2.8,       // ~10 km/h
            vehicle_min_mps: 6.0,       // ~21.6 km/h sustained -> likely a vehicle
            confirmation_samples: 5,
            window_capacity: 8,
        }
    }
}

pub struct ActivityClassifier {
    thresholds: ClassificationThresholds,
    window: VecDeque<ClassificationSample>,
    confirmed_state: MovementState,
}

impl ActivityClassifier {
    pub fn new(thresholds: ClassificationThresholds) -> Self {
        Self {
            thresholds,
            window: VecDeque::new(),
            confirmed_state: MovementState::Unknown,
        }
    }

    fn instantaneous_bucket(&self, s: &ClassificationSample) -> MovementState {
        let t = &self.thresholds;
        if s.speed_mps < 0.3 {
            MovementState::Stationary
        } else if s.speed_mps <= t.walking_max_mps {
            MovementState::Walking
        } else if s.speed_mps <= t.brisk_walking_max_mps {
            MovementState::BriskWalking
        } else if s.speed_mps <= t.jogging_max_mps {
            MovementState::Jogging
        } else if s.speed_mps < t.vehicle_min_mps {
            MovementState::Running
        } else {
            MovementState::VehicleMotion
        }
    }

    pub fn add_sample(&mut self, sample: ClassificationSample) -> MovementState {
        if self.window.len() == self.thresholds.window_capacity {
            self.window.pop_front();
        }
        self.window.push_back(sample);

        // Only attempt confirmation once we actually have a full window of
        // `confirmation_samples` readings — a single (or partial) sample
        // must never be able to flip the confirmed state.
        let n = self.thresholds.confirmation_samples;
        if self.window.len() < n {
            return self.confirmed_state;
        }
        let recent: Vec<MovementState> = self
            .window
            .iter()
            .rev()
            .take(n)
            .map(|s| self.instantaneous_bucket(s))
            .collect();

        let first = recent[0];
        if recent.iter().all(|b| *b == first) {
            self.confirmed_state = first;
        }

        self.confirmed_state
    }

    pub fn confirmed_state(&self) -> MovementState {
        self.confirmed_state
    }
}

/// Lightweight, standalone vehicle-motion heuristic combining sustained
/// speed with absence of step activity, per spec section 12. Distinct
/// from `ActivityClassifier` because vehicle detection may want to react
/// faster (fewer confirmation samples) given the safety implications.
pub struct VehicleMotionDetector {
    sustained_high_speed_ms: i64,
    threshold_ms: i64,
    vehicle_min_mps: f64,
    streak_started_at: Option<i64>,
}

impl VehicleMotionDetector {
    pub fn new(vehicle_min_mps: f64, sustained_threshold_ms: i64) -> Self {
        Self {
            sustained_high_speed_ms: 0,
            threshold_ms: sustained_threshold_ms,
            vehicle_min_mps,
            streak_started_at: None,
        }
    }

    pub fn default_config() -> Self {
        Self::new(6.0, 15_000)
    }

    /// Returns true once vehicle-level speed with no corroborating step
    /// activity has been sustained past the configured threshold.
    pub fn observe(&mut self, at_ms: i64, speed_mps: f64, step_detected: Option<bool>) -> bool {
        let looks_like_vehicle = speed_mps >= self.vehicle_min_mps && step_detected != Some(true);

        if looks_like_vehicle {
            let start = *self.streak_started_at.get_or_insert(at_ms);
            self.sustained_high_speed_ms = at_ms - start;
        } else {
            self.streak_started_at = None;
            self.sustained_high_speed_ms = 0;
        }

        self.sustained_high_speed_ms >= self.threshold_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(at: i64, speed: f64) -> ClassificationSample {
        ClassificationSample {
            at_ms: at,
            speed_mps: speed,
            cadence_spm: None,
        }
    }

    #[test]
    fn single_sample_does_not_confirm_classification() {
        let mut c = ActivityClassifier::new(ClassificationThresholds::default());
        let state = c.add_sample(s(0, 4.0)); // running-speed single sample
        assert_eq!(state, MovementState::Unknown);
    }

    #[test]
    fn sustained_running_speed_confirms_running() {
        let mut c = ActivityClassifier::new(ClassificationThresholds::default());
        let mut last = MovementState::Unknown;
        for i in 0..5 {
            last = c.add_sample(s(i * 1000, 3.5));
        }
        assert_eq!(last, MovementState::Running);
    }

    #[test]
    fn mixed_speeds_do_not_confirm() {
        let mut c = ActivityClassifier::new(ClassificationThresholds::default());
        let speeds = [1.0, 3.5, 1.0, 3.5, 1.0];
        let mut last = MovementState::Unknown;
        for (i, sp) in speeds.iter().enumerate() {
            last = c.add_sample(s(i as i64 * 1000, *sp));
        }
        assert_eq!(last, MovementState::Unknown);
    }

    #[test]
    fn sustained_walking_confirms_walking() {
        let mut c = ActivityClassifier::new(ClassificationThresholds::default());
        let mut last = MovementState::Unknown;
        for i in 0..5 {
            last = c.add_sample(s(i * 1000, 1.2));
        }
        assert_eq!(last, MovementState::Walking);
    }

    #[test]
    fn vehicle_detector_ignores_brief_high_speed() {
        let mut v = VehicleMotionDetector::new(6.0, 15_000);
        assert!(!v.observe(0, 8.0, None));
        assert!(!v.observe(5_000, 8.0, None));
    }

    #[test]
    fn vehicle_detector_triggers_after_sustained_high_speed_no_steps() {
        let mut v = VehicleMotionDetector::new(6.0, 15_000);
        v.observe(0, 8.0, Some(false));
        v.observe(5_000, 8.0, Some(false));
        let triggered = v.observe(16_000, 8.0, Some(false));
        assert!(triggered);
    }

    #[test]
    fn vehicle_detector_reset_by_step_activity() {
        let mut v = VehicleMotionDetector::new(6.0, 15_000);
        v.observe(0, 8.0, None);
        v.observe(5_000, 8.0, Some(true)); // steps detected -> resets streak
        let triggered = v.observe(16_000, 8.0, None);
        assert!(!triggered);
    }
}
