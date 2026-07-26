//! GPS quality monitor (spec section 31).
//!
//! Aggregates recent accuracy readings, fix age, and accept/reject ratio
//! into a single discrete quality bucket the UI can display, and which
//! gates whether the app should warn the user before starting a workout.

use std::collections::VecDeque;

use crate::models::{EpochMillis, GpsQualityState};

#[derive(Debug, Clone, Copy)]
pub struct GpsQualitySample {
    pub at_ms: EpochMillis,
    pub accuracy_meters: Option<f64>,
    pub accepted: bool,
}

pub struct GpsQualityMonitor {
    window: VecDeque<GpsQualitySample>,
    window_capacity: usize,
}

impl GpsQualityMonitor {
    pub fn new(window_capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_capacity),
            window_capacity: window_capacity.max(1),
        }
    }

    pub fn default_config() -> Self {
        Self::new(10)
    }

    pub fn add_sample(&mut self, sample: GpsQualitySample) {
        if self.window.len() == self.window_capacity {
            self.window.pop_front();
        }
        self.window.push_back(sample);
    }

    /// Evaluates current quality given the most recent samples and the
    /// "now" timestamp (used to detect a stale/no-fix situation even if
    /// the window still holds old samples).
    pub fn evaluate(&self, now_ms: EpochMillis) -> GpsQualityState {
        let Some(last) = self.window.back() else {
            return GpsQualityState::Unavailable;
        };

        let age_ms = now_ms - last.at_ms;
        if age_ms > 15_000 {
            return GpsQualityState::Unavailable;
        }

        let accepted_count = self.window.iter().filter(|s| s.accepted).count();
        let accept_ratio = accepted_count as f64 / self.window.len() as f64;

        let avg_accuracy: Option<f64> = {
            let (sum, n) = self
                .window
                .iter()
                .filter_map(|s| s.accuracy_meters)
                .fold((0.0, 0usize), |(sum, n), a| (sum + a, n + 1));
            if n == 0 {
                None
            } else {
                Some(sum / n as f64)
            }
        };

        match (avg_accuracy, accept_ratio, age_ms) {
            (Some(acc), ratio, age) if acc <= 8.0 && ratio >= 0.9 && age <= 3_000 => {
                GpsQualityState::Excellent
            }
            (Some(acc), ratio, age) if acc <= 15.0 && ratio >= 0.7 && age <= 6_000 => {
                GpsQualityState::Good
            }
            (Some(acc), ratio, _) if acc <= 30.0 && ratio >= 0.4 => GpsQualityState::Weak,
            (None, ratio, _) if ratio >= 0.7 => GpsQualityState::Good,
            _ => GpsQualityState::Poor,
        }
    }

    /// Whether the engine should warn the user before starting a workout.
    pub fn should_warn_before_start(&self, now_ms: EpochMillis) -> bool {
        matches!(
            self.evaluate(now_ms),
            GpsQualityState::Poor | GpsQualityState::Unavailable
        )
    }
}

impl Default for GpsQualityMonitor {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(at: i64, acc: Option<f64>, accepted: bool) -> GpsQualitySample {
        GpsQualitySample {
            at_ms: at,
            accuracy_meters: acc,
            accepted,
        }
    }

    #[test]
    fn empty_monitor_is_unavailable() {
        let m = GpsQualityMonitor::default_config();
        assert_eq!(m.evaluate(1000), GpsQualityState::Unavailable);
    }

    #[test]
    fn stale_last_fix_is_unavailable() {
        let mut m = GpsQualityMonitor::default_config();
        m.add_sample(sample(0, Some(5.0), true));
        assert_eq!(m.evaluate(20_000), GpsQualityState::Unavailable);
    }

    #[test]
    fn tight_accuracy_high_accept_ratio_is_excellent() {
        let mut m = GpsQualityMonitor::default_config();
        for i in 0..10 {
            m.add_sample(sample(i * 1000, Some(5.0), true));
        }
        assert_eq!(m.evaluate(9_500), GpsQualityState::Excellent);
    }

    #[test]
    fn poor_accuracy_and_low_accept_ratio_is_poor() {
        let mut m = GpsQualityMonitor::default_config();
        for i in 0..10 {
            m.add_sample(sample(i * 1000, Some(80.0), i % 3 == 0));
        }
        assert_eq!(m.evaluate(9_500), GpsQualityState::Poor);
    }

    #[test]
    fn should_warn_before_start_when_poor() {
        let mut m = GpsQualityMonitor::default_config();
        for i in 0..5 {
            m.add_sample(sample(i * 1000, Some(90.0), false));
        }
        assert!(m.should_warn_before_start(4_500));
    }
}
