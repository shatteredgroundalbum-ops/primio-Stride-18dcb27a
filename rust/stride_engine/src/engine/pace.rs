//! Pace engine (spec section 8).
//!
//! Pace is expressed internally as seconds-per-kilometer (a plain f64) so
//! it is trivial to serialize and never involves an ambiguous "duration"
//! type. Conversions to sec-per-mile happen in `units.rs`. Zero-distance
//! and zero-time cases always return `0.0` rather than `Infinity`/`NaN` —
//! callers should treat `0.0` as "no pace yet" and avoid displaying it as
//! a real pace value.

use crate::engine::distance::{METERS_PER_KILOMETER, METERS_PER_MILE};

#[derive(Debug, Clone, Copy, Default)]
pub struct PaceSnapshot {
    pub current_sec_per_km: f64,
    pub smoothed_sec_per_km: f64,
    pub average_moving_sec_per_km: f64,
    pub average_elapsed_sec_per_km: f64,
    pub fastest_valid_sec_per_km: f64,
}

/// Converts a speed (m/s) into pace (seconds per kilometer), safely.
pub fn speed_to_pace_sec_per_km(speed_mps: f64) -> f64 {
    if !speed_mps.is_finite() || speed_mps <= 0.0 {
        return 0.0;
    }
    METERS_PER_KILOMETER / speed_mps
}

/// distance/time based pace, safe against zero distance.
pub fn pace_sec_per_km(duration_ms: i64, distance_meters: f64) -> f64 {
    if distance_meters <= 0.0 || duration_ms <= 0 {
        return 0.0;
    }
    let km = distance_meters / METERS_PER_KILOMETER;
    (duration_ms as f64 / 1000.0) / km
}

pub fn pace_sec_per_km_to_sec_per_mile(pace_sec_per_km: f64) -> f64 {
    if pace_sec_per_km <= 0.0 {
        return 0.0;
    }
    pace_sec_per_km * (METERS_PER_MILE / METERS_PER_KILOMETER)
}

/// Tracks fastest-ever valid pace across a workout (used for "fastest
/// pace" stat and personal-record detection). A pace sample is only
/// considered "valid" if the caller has already vetted the underlying
/// speed as plausible (i.e. came from `SpeedEngine::max_valid_mps` or
/// similar) — this function does not re-validate physical plausibility.
pub struct FastestPaceTracker {
    fastest_sec_per_km: Option<f64>,
    /// Ignore any sample corresponding to less than this many meters of
    /// underlying movement, to avoid a single noisy short burst being
    /// recorded as a "fastest pace ever".
    min_sample_distance_m: f64,
}

impl FastestPaceTracker {
    pub fn new(min_sample_distance_m: f64) -> Self {
        Self {
            fastest_sec_per_km: None,
            min_sample_distance_m,
        }
    }

    pub fn observe(&mut self, sample_sec_per_km: f64, sample_distance_m: f64) {
        if sample_sec_per_km <= 0.0 || sample_distance_m < self.min_sample_distance_m {
            return;
        }
        match self.fastest_sec_per_km {
            Some(best) if sample_sec_per_km >= best => {}
            _ => self.fastest_sec_per_km = Some(sample_sec_per_km),
        }
    }

    pub fn fastest_sec_per_km(&self) -> f64 {
        self.fastest_sec_per_km.unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_distance_gives_zero_pace_not_infinity() {
        assert_eq!(pace_sec_per_km(60_000, 0.0), 0.0);
    }

    #[test]
    fn zero_time_gives_zero_pace_not_nan() {
        assert_eq!(pace_sec_per_km(0, 500.0), 0.0);
    }

    #[test]
    fn known_pace_calculation() {
        // 1 km in 6 minutes => 360 sec/km
        let p = pace_sec_per_km(6 * 60 * 1000, 1000.0);
        assert!((p - 360.0).abs() < 1e-6);
    }

    #[test]
    fn speed_to_pace_roundtrip() {
        // 3.0 m/s => 1000/3.0 = 333.33 sec/km
        let p = speed_to_pace_sec_per_km(3.0);
        assert!((p - 333.333).abs() < 0.01);
    }

    #[test]
    fn negative_or_zero_speed_yields_zero_pace() {
        assert_eq!(speed_to_pace_sec_per_km(0.0), 0.0);
        assert_eq!(speed_to_pace_sec_per_km(-5.0), 0.0);
    }

    #[test]
    fn fastest_pace_tracker_keeps_lowest_value() {
        let mut t = FastestPaceTracker::new(50.0);
        t.observe(300.0, 200.0);
        t.observe(250.0, 200.0); // faster (lower sec/km)
        t.observe(400.0, 200.0); // slower, ignored
        assert!((t.fastest_sec_per_km() - 250.0).abs() < 1e-9);
    }

    #[test]
    fn fastest_pace_tracker_ignores_tiny_bursts() {
        let mut t = FastestPaceTracker::new(50.0);
        t.observe(100.0, 5.0); // too short a sample distance to trust
        assert_eq!(t.fastest_sec_per_km(), 0.0);
    }

    #[test]
    fn mile_conversion() {
        let km_pace = 300.0; // 5:00/km
        let mile_pace = pace_sec_per_km_to_sec_per_mile(km_pace);
        assert!((mile_pace - 482.8).abs() < 0.5);
    }
}
