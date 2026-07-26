//! Heart-rate integration (spec section 16).
//!
//! Tracks live/average/max/min heart rate and time-in-zone. The workout
//! must function identically whether or not a wearable is connected —
//! every method here degrades gracefully to "no data" rather than
//! erroring. No medical interpretation is performed; zones are purely
//! descriptive training-intensity buckets.

use std::collections::HashMap;

use crate::models::{EpochMillis, HeartRateZone};

pub struct HeartRateEngine {
    max_hr_estimate: u16,
    samples_sum: u64,
    samples_count: u64,
    min_bpm: Option<u16>,
    max_bpm: Option<u16>,
    current_bpm: Option<u16>,
    last_sample_at: Option<EpochMillis>,
    time_in_zone_ms: HashMap<HeartRateZone, i64>,
    last_zone_update_at: Option<EpochMillis>,
    current_zone: Option<HeartRateZone>,
}

impl HeartRateEngine {
    /// `max_hr_estimate` typically comes from the classic 220-age formula,
    /// computed by the caller (Dart side) if the user's age is known, or a
    /// sensible population default (e.g. 190) otherwise.
    pub fn new(max_hr_estimate: u16) -> Self {
        Self {
            max_hr_estimate,
            samples_sum: 0,
            samples_count: 0,
            min_bpm: None,
            max_bpm: None,
            current_bpm: None,
            last_sample_at: None,
            time_in_zone_ms: HashMap::new(),
            last_zone_update_at: None,
            current_zone: None,
        }
    }

    /// Feeds a new heart-rate sample. `at_ms` is used both to detect
    /// missing-signal gaps and to accumulate time-in-zone since the last
    /// sample.
    pub fn add_sample(&mut self, at_ms: EpochMillis, bpm: u16) {
        if bpm == 0 || bpm > 300 {
            return; // sensor noise / disconnection artifact
        }

        // Accumulate time in the *previous* zone up to this new sample.
        if let (Some(zone), Some(last_at)) = (self.current_zone, self.last_zone_update_at) {
            let delta = (at_ms - last_at).max(0);
            *self.time_in_zone_ms.entry(zone).or_insert(0) += delta;
        }

        self.samples_sum += bpm as u64;
        self.samples_count += 1;
        self.min_bpm = Some(self.min_bpm.map_or(bpm, |m| m.min(bpm)));
        self.max_bpm = Some(self.max_bpm.map_or(bpm, |m| m.max(bpm)));
        self.current_bpm = Some(bpm);
        self.last_sample_at = Some(at_ms);

        self.current_zone = Some(HeartRateZone::classify(bpm, self.max_hr_estimate));
        self.last_zone_update_at = Some(at_ms);
    }

    pub fn current_bpm(&self) -> Option<u16> {
        self.current_bpm
    }

    pub fn average_bpm(&self) -> Option<f64> {
        if self.samples_count == 0 {
            return None;
        }
        Some(self.samples_sum as f64 / self.samples_count as f64)
    }

    pub fn min_bpm(&self) -> Option<u16> {
        self.min_bpm
    }

    pub fn max_bpm(&self) -> Option<u16> {
        self.max_bpm
    }

    /// Whether the signal has gone quiet for longer than `stale_after_ms`
    /// — used to emit a `HeartRateUnavailable` coaching event.
    pub fn is_stale(&self, now_ms: EpochMillis, stale_after_ms: i64) -> bool {
        match self.last_sample_at {
            Some(t) => now_ms - t > stale_after_ms,
            None => true,
        }
    }

    pub fn time_in_zone_ms(&self, zone: HeartRateZone) -> i64 {
        *self.time_in_zone_ms.get(&zone).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_samples_yields_none_everywhere() {
        let e = HeartRateEngine::new(190);
        assert!(e.current_bpm().is_none());
        assert!(e.average_bpm().is_none());
        assert!(e.is_stale(0, 1000));
    }

    #[test]
    fn tracks_min_max_average() {
        let mut e = HeartRateEngine::new(190);
        e.add_sample(0, 120);
        e.add_sample(1000, 140);
        e.add_sample(2000, 110);
        assert_eq!(e.min_bpm(), Some(110));
        assert_eq!(e.max_bpm(), Some(140));
        assert!((e.average_bpm().unwrap() - 123.333).abs() < 0.01);
    }

    #[test]
    fn rejects_implausible_samples() {
        let mut e = HeartRateEngine::new(190);
        e.add_sample(0, 0);
        e.add_sample(1000, 400);
        assert!(e.current_bpm().is_none());
    }

    #[test]
    fn staleness_detection() {
        let mut e = HeartRateEngine::new(190);
        e.add_sample(0, 120);
        assert!(!e.is_stale(5_000, 10_000));
        assert!(e.is_stale(20_000, 10_000));
    }

    #[test]
    fn accumulates_time_in_zone() {
        let mut e = HeartRateEngine::new(200); // easy math: zone thresholds at 120,140,160,180
        e.add_sample(0, 110); // zone1 (<60%: <120)
        e.add_sample(5_000, 110); // still zone1, accumulate 5s in zone1
        e.add_sample(10_000, 170); // move to zone4, accumulate another 5s in zone1
        let z1 = e.time_in_zone_ms(HeartRateZone::Zone1Rest);
        assert_eq!(z1, 10_000);
    }
}
