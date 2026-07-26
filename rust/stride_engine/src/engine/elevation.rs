//! Elevation engine (spec section 14).
//!
//! Raw GPS altitude is noisy — feeding every fluctuation directly into a
//! gain/loss accumulator can massively inflate "climbing" totals on a
//! flat route. This module applies an exponential smoothing filter to
//! incoming altitude samples and only accumulates gain/loss once smoothed
//! altitude moves past a minimum-change threshold (a simple, dependency
//! free hysteresis band).

#[derive(Debug, Clone, Copy, Default)]
pub struct ElevationTotals {
    pub gain_meters: f64,
    pub loss_meters: f64,
    pub min_altitude_meters: Option<f64>,
    pub max_altitude_meters: Option<f64>,
    pub current_smoothed_altitude_meters: Option<f64>,
}

pub struct ElevationEngine {
    smoothed_altitude: Option<f64>,
    /// Exponential smoothing factor in (0, 1]; lower = smoother/slower to
    /// react, higher = tracks raw input more closely.
    alpha: f64,
    /// Minimum smoothed-altitude change (meters) required before it counts
    /// toward gain/loss, to reject sub-noise-floor fluctuations.
    hysteresis_meters: f64,
    /// The smoothed altitude at the last point we "settled" at, i.e. the
    /// reference for measuring the next gain/loss increment.
    reference_altitude: Option<f64>,
    totals: ElevationTotals,
}

impl ElevationEngine {
    pub fn new(alpha: f64, hysteresis_meters: f64) -> Self {
        Self {
            smoothed_altitude: None,
            alpha: alpha.clamp(0.01, 1.0),
            hysteresis_meters,
            reference_altitude: None,
            totals: ElevationTotals::default(),
        }
    }

    /// Default: moderate smoothing (alpha=0.25), 1.5m hysteresis band —
    /// tuned to reject typical consumer-GPS altitude jitter (which is
    /// commonly ±3-5m) while still capturing real hill climbs.
    pub fn default_config() -> Self {
        Self::new(0.25, 1.5)
    }

    pub fn totals(&self) -> ElevationTotals {
        self.totals
    }

    pub fn add_sample(&mut self, raw_altitude_meters: f64) {
        if !raw_altitude_meters.is_finite() {
            return;
        }

        let smoothed = match self.smoothed_altitude {
            Some(prev) => prev + self.alpha * (raw_altitude_meters - prev),
            None => raw_altitude_meters,
        };
        self.smoothed_altitude = Some(smoothed);

        self.totals.min_altitude_meters = Some(
            self.totals
                .min_altitude_meters
                .map_or(smoothed, |m| m.min(smoothed)),
        );
        self.totals.max_altitude_meters = Some(
            self.totals
                .max_altitude_meters
                .map_or(smoothed, |m| m.max(smoothed)),
        );
        self.totals.current_smoothed_altitude_meters = Some(smoothed);

        let reference = *self.reference_altitude.get_or_insert(smoothed);
        let delta = smoothed - reference;

        if delta >= self.hysteresis_meters {
            self.totals.gain_meters += delta;
            self.reference_altitude = Some(smoothed);
        } else if delta <= -self.hysteresis_meters {
            self.totals.loss_meters += -delta;
            self.reference_altitude = Some(smoothed);
        }
        // else: within the noise band, don't move the reference — this is
        // what prevents tiny back-and-forth jitter from accumulating.
    }
}

impl Default for ElevationEngine {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Grade (slope) as a percentage between two points: rise/run * 100.
pub fn grade_percent(rise_meters: f64, run_meters: f64) -> f64 {
    if run_meters <= 0.0 {
        return 0.0;
    }
    (rise_meters / run_meters) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_route_produces_no_gain_or_loss() {
        let mut e = ElevationEngine::default_config();
        for _ in 0..20 {
            e.add_sample(100.0);
        }
        let t = e.totals();
        assert!(t.gain_meters < 0.1);
        assert!(t.loss_meters < 0.1);
    }

    #[test]
    fn tiny_jitter_does_not_accumulate_as_gain() {
        let mut e = ElevationEngine::default_config();
        // Oscillate by +-0.3m around 100m repeatedly - should be absorbed
        // by smoothing + hysteresis.
        for i in 0..40 {
            let jitter = if i % 2 == 0 { 0.3 } else { -0.3 };
            e.add_sample(100.0 + jitter);
        }
        let t = e.totals();
        assert!(t.gain_meters < 1.0, "gain={}", t.gain_meters);
        assert!(t.loss_meters < 1.0, "loss={}", t.loss_meters);
    }

    #[test]
    fn real_climb_is_captured() {
        let mut e = ElevationEngine::new(0.5, 1.0); // faster-reacting for the test
        for alt in (100..150).step_by(5) {
            e.add_sample(alt as f64);
        }
        let t = e.totals();
        assert!(t.gain_meters > 30.0, "gain={}", t.gain_meters);
        assert_eq!(t.loss_meters, 0.0);
    }

    #[test]
    fn descent_is_captured_as_loss() {
        let mut e = ElevationEngine::new(0.5, 1.0);
        for alt in (0..50).rev().step_by(5) {
            e.add_sample(150.0 + alt as f64 - 50.0);
        }
        // simpler: explicit descending sequence
        let mut e2 = ElevationEngine::new(0.5, 1.0);
        for alt in [150.0, 145.0, 140.0, 130.0, 120.0, 110.0] {
            e2.add_sample(alt);
        }
        let t = e2.totals();
        assert!(t.loss_meters > 20.0, "loss={}", t.loss_meters);
        let _ = e; // silence unused warning for first experiment
    }

    #[test]
    fn min_max_altitude_tracked() {
        let mut e = ElevationEngine::default_config();
        e.add_sample(50.0);
        e.add_sample(80.0);
        e.add_sample(40.0);
        let t = e.totals();
        assert!(t.min_altitude_meters.unwrap() <= 50.0);
        assert!(t.max_altitude_meters.unwrap() >= 50.0);
    }

    #[test]
    fn non_finite_samples_ignored() {
        let mut e = ElevationEngine::default_config();
        e.add_sample(f64::NAN);
        e.add_sample(f64::INFINITY);
        assert!(e.totals().current_smoothed_altitude_meters.is_none());
    }

    #[test]
    fn grade_percent_handles_zero_run() {
        assert_eq!(grade_percent(10.0, 0.0), 0.0);
        assert!((grade_percent(5.0, 100.0) - 5.0).abs() < 1e-9);
    }
}
