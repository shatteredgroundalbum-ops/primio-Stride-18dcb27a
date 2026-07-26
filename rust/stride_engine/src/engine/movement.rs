//! Stationary vs. movement detection (spec section 10).
//!
//! Determines whether the user is genuinely moving using a rolling window
//! of recent accepted-point distances/speeds plus (optionally) step and
//! accuracy signals. This feeds both auto-pause (`autopause.rs`) and
//! moving-time accounting (`timekeeping.rs`).

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementSignal {
    Moving,
    Stationary,
    /// Not enough data yet to make a confident call.
    Indeterminate,
}

#[derive(Debug, Clone, Copy)]
pub struct MovementSample {
    pub at_ms: i64,
    pub distance_since_last_m: f64,
    pub speed_mps: f64,
    pub accuracy_m: Option<f64>,
    /// True if a step was detected in this interval (from step sensor),
    /// when available. `None` if no step sensor is present.
    pub step_detected: Option<bool>,
}

pub struct MovementDetector {
    window: VecDeque<MovementSample>,
    window_span_ms: i64,
    /// Minimum smoothed speed (m/s) to call it "moving".
    moving_speed_threshold_mps: f64,
    /// Minimum cumulative distance across the window (m) to call it
    /// "moving" even if instantaneous speed dips (guards against a single
    /// slow sample flipping the state).
    moving_distance_threshold_m: f64,
}

impl MovementDetector {
    pub fn new(window_span_ms: i64, moving_speed_threshold_mps: f64, moving_distance_threshold_m: f64) -> Self {
        Self {
            window: VecDeque::new(),
            window_span_ms,
            moving_speed_threshold_mps,
            moving_distance_threshold_m,
        }
    }

    /// Default: 10-second rolling window, 0.4 m/s (~1.4 km/h) minimum
    /// speed, 1.5m minimum cumulative movement in the window. These are
    /// deliberately permissive at the low end to capture slow, deliberate
    /// walking while still catching GPS drift (drift tends to be smaller
    /// and less directionally consistent than real walking, but this
    /// module treats any sustained distance accrual as movement — outlier
    /// single-point drift is already filtered upstream by `GpsFilter`).
    pub fn default_config() -> Self {
        Self::new(10_000, 0.4, 1.5)
    }

    pub fn add_sample(&mut self, sample: MovementSample) {
        self.window.push_back(sample);
        self.trim_window(sample.at_ms);
    }

    fn trim_window(&mut self, now_ms: i64) {
        while let Some(front) = self.window.front() {
            if now_ms - front.at_ms > self.window_span_ms {
                self.window.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn current_signal(&self) -> MovementSignal {
        if self.window.is_empty() {
            return MovementSignal::Indeterminate;
        }

        let total_distance: f64 = self.window.iter().map(|s| s.distance_since_last_m).sum();
        let avg_speed: f64 =
            self.window.iter().map(|s| s.speed_mps).sum::<f64>() / self.window.len() as f64;

        // Step sensor, if present, is a strong corroborating signal.
        let any_step = self.window.iter().any(|s| s.step_detected == Some(true));
        let step_sensor_present = self.window.iter().any(|s| s.step_detected.is_some());

        if step_sensor_present {
            if any_step && (total_distance >= self.moving_distance_threshold_m || avg_speed >= self.moving_speed_threshold_mps) {
                return MovementSignal::Moving;
            }
            if !any_step {
                return MovementSignal::Stationary;
            }
        }

        if total_distance >= self.moving_distance_threshold_m || avg_speed >= self.moving_speed_threshold_mps {
            MovementSignal::Moving
        } else {
            MovementSignal::Stationary
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(at_ms: i64, dist: f64, speed: f64) -> MovementSample {
        MovementSample {
            at_ms,
            distance_since_last_m: dist,
            speed_mps: speed,
            accuracy_m: Some(8.0),
            step_detected: None,
        }
    }

    #[test]
    fn empty_window_is_indeterminate() {
        let d = MovementDetector::default_config();
        assert_eq!(d.current_signal(), MovementSignal::Indeterminate);
    }

    #[test]
    fn sustained_walking_speed_detected_as_moving() {
        let mut d = MovementDetector::default_config();
        for i in 0..5 {
            d.add_sample(sample(i * 2_000, 3.0, 1.4));
        }
        assert_eq!(d.current_signal(), MovementSignal::Moving);
    }

    #[test]
    fn standing_still_detected_as_stationary() {
        let mut d = MovementDetector::default_config();
        for i in 0..5 {
            d.add_sample(sample(i * 2_000, 0.05, 0.02));
        }
        assert_eq!(d.current_signal(), MovementSignal::Stationary);
    }

    #[test]
    fn window_trims_old_samples() {
        let mut d = MovementDetector::new(5_000, 0.4, 1.5);
        d.add_sample(sample(0, 10.0, 2.0));
        d.add_sample(sample(20_000, 0.0, 0.0)); // 20s later, old sample should be trimmed
        // Only the stationary sample remains in-window.
        assert_eq!(d.current_signal(), MovementSignal::Stationary);
    }

    #[test]
    fn step_sensor_corroborates_stationary_despite_gps_noise() {
        let mut d = MovementDetector::default_config();
        for i in 0..5 {
            let mut s = sample(i * 2_000, 0.3, 0.1); // small GPS drift noise
            s.step_detected = Some(false);
            d.add_sample(s);
        }
        assert_eq!(d.current_signal(), MovementSignal::Stationary);
    }
}
