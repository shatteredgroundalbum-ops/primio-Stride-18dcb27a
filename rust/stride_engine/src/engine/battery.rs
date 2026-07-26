//! Battery and sampling manager (spec section 32).
//!
//! Decides GPS update interval, minimum movement distance filter, and
//! sensor sampling rate based on the current workout state (active vs.
//! paused), battery level, and whether battery-saver mode is active. The
//! actual OS-level API calls happen on the Dart/native side; this module
//! is the pure decision logic, easily unit tested without a device.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SamplingProfile {
    /// Desired GPS update interval in milliseconds.
    pub gps_interval_ms: u32,
    /// Minimum movement (meters) before a new GPS update is delivered.
    pub gps_min_distance_m: f32,
    /// Whether to request high-accuracy (GPS+network+sensor fusion) mode.
    pub high_accuracy: bool,
    /// Sensor (accelerometer/step) sampling interval in milliseconds.
    pub sensor_interval_ms: u32,
    /// How often queued points should be flushed to local storage, in
    /// milliseconds (batched writes rather than one write per point).
    pub db_batch_flush_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkoutActivityLevel {
    Active,
    Paused,
    AutoPaused,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BatteryContext {
    pub battery_percent: u8,
    pub battery_saver_enabled: bool,
    pub is_charging: bool,
}

/// Chooses a sampling profile. Rules, in priority order:
///   1. Charging always allows full-accuracy sampling regardless of %.
///   2. Paused/auto-paused states always drop to a low-power profile —
///      there is no need for tight GPS sampling while stationary.
///   3. Battery-saver mode or low battery (<15%) drops interval/accuracy,
///      but never below a floor that would make tracking unreliable
///      (accuracy should not be reduced so much the workout becomes
///      unusable, per spec).
///   4. Otherwise, full active-workout sampling.
pub fn choose_sampling_profile(
    activity: WorkoutActivityLevel,
    battery: BatteryContext,
) -> SamplingProfile {
    if activity != WorkoutActivityLevel::Active {
        return SamplingProfile {
            gps_interval_ms: 10_000,
            gps_min_distance_m: 15.0,
            high_accuracy: false,
            sensor_interval_ms: 5_000,
            db_batch_flush_ms: 15_000,
        };
    }

    if battery.is_charging {
        return full_accuracy_profile();
    }

    let low_battery = battery.battery_percent < 15;
    if battery.battery_saver_enabled || low_battery {
        // Reduced-power mode: still tracks reliably, just less frequently.
        // Never disables high_accuracy entirely for an active workout —
        // that would make distance/pace unreliable — but relaxes interval.
        return SamplingProfile {
            gps_interval_ms: 3_000,
            gps_min_distance_m: 8.0,
            high_accuracy: true,
            sensor_interval_ms: 2_000,
            db_batch_flush_ms: 8_000,
        };
    }

    full_accuracy_profile()
}

fn full_accuracy_profile() -> SamplingProfile {
    SamplingProfile {
        gps_interval_ms: 1_000,
        gps_min_distance_m: 3.0,
        high_accuracy: true,
        sensor_interval_ms: 500,
        db_batch_flush_ms: 5_000,
    }
}

/// Whether to show a low-battery warning banner to the user during an
/// active workout (informational; does not itself change sampling).
pub fn should_show_low_battery_warning(battery: BatteryContext) -> bool {
    !battery.is_charging && battery.battery_percent <= 10
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pct: u8, saver: bool, charging: bool) -> BatteryContext {
        BatteryContext {
            battery_percent: pct,
            battery_saver_enabled: saver,
            is_charging: charging,
        }
    }

    #[test]
    fn active_full_battery_gets_full_accuracy() {
        let p = choose_sampling_profile(WorkoutActivityLevel::Active, ctx(80, false, false));
        assert_eq!(p.gps_interval_ms, 1_000);
        assert!(p.high_accuracy);
    }

    #[test]
    fn paused_always_uses_low_power_regardless_of_battery() {
        let p = choose_sampling_profile(WorkoutActivityLevel::Paused, ctx(100, false, false));
        assert_eq!(p.gps_interval_ms, 10_000);
        assert!(!p.high_accuracy);
    }

    #[test]
    fn low_battery_reduces_interval_but_keeps_high_accuracy() {
        let p = choose_sampling_profile(WorkoutActivityLevel::Active, ctx(10, false, false));
        assert_eq!(p.gps_interval_ms, 3_000);
        assert!(p.high_accuracy, "accuracy must not be sacrificed to unreliability");
    }

    #[test]
    fn battery_saver_reduces_sampling_even_with_decent_battery() {
        let p = choose_sampling_profile(WorkoutActivityLevel::Active, ctx(60, true, false));
        assert_eq!(p.gps_interval_ms, 3_000);
    }

    #[test]
    fn charging_overrides_low_battery_state() {
        let p = choose_sampling_profile(WorkoutActivityLevel::Active, ctx(5, true, true));
        assert_eq!(p.gps_interval_ms, 1_000);
    }

    #[test]
    fn low_battery_warning_only_when_not_charging() {
        assert!(should_show_low_battery_warning(ctx(8, false, false)));
        assert!(!should_show_low_battery_warning(ctx(8, false, true)));
        assert!(!should_show_low_battery_warning(ctx(50, false, false)));
    }
}
