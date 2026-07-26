//! Speed engine (spec section 7).
//!
//! Combines device-reported GPS speed with a distance/time derived speed,
//! and exposes a smoothed "current speed" so the UI doesn't jitter every
//! second. All internal values are meters/second; conversions to mph/km-h
//! happen in `units.rs`.

use std::collections::VecDeque;

pub const MPS_TO_MPH: f64 = 2.23694;
pub const MPS_TO_KMH: f64 = 3.6;

#[derive(Debug, Clone, Copy, Default)]
pub struct SpeedSnapshot {
    pub current_mps: f64,
    pub smoothed_mps: f64,
    pub average_moving_mps: f64,
    pub average_overall_mps: f64,
    pub max_valid_mps: f64,
}

/// Rolling-window smoother + running aggregates for speed.
pub struct SpeedEngine {
    window: VecDeque<f64>,
    window_capacity: usize,
    max_valid_mps: f64,
    /// Guard: readings above this are treated as sensor noise/spikes and
    /// excluded from max-speed tracking (still may contribute to smoothed
    /// average since a single high sample gets diluted by the window).
    plausibility_ceiling_mps: f64,
}

impl SpeedEngine {
    pub fn new(window_capacity: usize, plausibility_ceiling_mps: f64) -> Self {
        Self {
            window: VecDeque::with_capacity(window_capacity),
            window_capacity: window_capacity.max(1),
            max_valid_mps: 0.0,
            plausibility_ceiling_mps,
        }
    }

    /// Default configuration: 5-sample smoothing window, implausible above
    /// 12.5 m/s (~45 km/h, faster than any legitimate run/walk).
    pub fn default_config() -> Self {
        Self::new(5, 12.5)
    }

    /// Feeds a new instantaneous speed sample (m/s), typically computed as
    /// distance-delta / time-delta between the last two accepted GPS
    /// points, optionally blended with device-reported GPS speed by the
    /// caller before being passed in here.
    pub fn add_sample(&mut self, instantaneous_mps: f64) {
        if !instantaneous_mps.is_finite() || instantaneous_mps < 0.0 {
            return;
        }

        if self.window.len() == self.window_capacity {
            self.window.pop_front();
        }
        self.window.push_back(instantaneous_mps);

        if instantaneous_mps <= self.plausibility_ceiling_mps
            && instantaneous_mps > self.max_valid_mps
        {
            self.max_valid_mps = instantaneous_mps;
        }
    }

    pub fn smoothed_mps(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }
        self.window.iter().sum::<f64>() / self.window.len() as f64
    }

    pub fn current_mps(&self) -> f64 {
        self.window.back().copied().unwrap_or(0.0)
    }

    pub fn max_valid_mps(&self) -> f64 {
        self.max_valid_mps
    }

    pub fn snapshot(&self, moving_distance_m: f64, moving_ms: i64, total_distance_m: f64, elapsed_ms: i64) -> SpeedSnapshot {
        SpeedSnapshot {
            current_mps: self.current_mps(),
            smoothed_mps: self.smoothed_mps(),
            average_moving_mps: safe_average_speed(moving_distance_m, moving_ms),
            average_overall_mps: safe_average_speed(total_distance_m, elapsed_ms),
            max_valid_mps: self.max_valid_mps,
        }
    }
}

/// Average speed (m/s) = distance / time, safely handling zero time.
pub fn safe_average_speed(distance_m: f64, duration_ms: i64) -> f64 {
    if duration_ms <= 0 {
        return 0.0;
    }
    distance_m / (duration_ms as f64 / 1000.0)
}

pub fn mps_to_mph(mps: f64) -> f64 {
    mps * MPS_TO_MPH
}

pub fn mps_to_kmh(mps: f64) -> f64 {
    mps * MPS_TO_KMH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothing_averages_recent_samples() {
        let mut e = SpeedEngine::new(3, 100.0);
        e.add_sample(1.0);
        e.add_sample(2.0);
        e.add_sample(3.0);
        assert!((e.smoothed_mps() - 2.0).abs() < 1e-9);
        e.add_sample(6.0); // window drops the 1.0
        assert!((e.smoothed_mps() - (2.0 + 3.0 + 6.0) / 3.0).abs() < 1e-9);
    }

    #[test]
    fn implausible_spike_excluded_from_max_but_not_window() {
        let mut e = SpeedEngine::new(5, 12.5);
        e.add_sample(3.0);
        e.add_sample(50.0); // GPS spike, e.g. from a bad fix
        assert_eq!(e.max_valid_mps(), 3.0);
        // still enters the smoothing window (diluted, not silently dropped)
        assert!(e.smoothed_mps() > 3.0);
    }

    #[test]
    fn negative_or_nan_samples_ignored() {
        let mut e = SpeedEngine::new(5, 12.5);
        e.add_sample(-1.0);
        e.add_sample(f64::NAN);
        assert_eq!(e.current_mps(), 0.0);
    }

    #[test]
    fn safe_average_speed_handles_zero_duration() {
        assert_eq!(safe_average_speed(100.0, 0), 0.0);
        assert!((safe_average_speed(100.0, 10_000) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn unit_conversions() {
        assert!((mps_to_mph(1.0) - 2.23694).abs() < 1e-6);
        assert!((mps_to_kmh(1.0) - 3.6).abs() < 1e-9);
    }
}
