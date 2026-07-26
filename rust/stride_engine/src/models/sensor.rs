use serde::{Deserialize, Serialize};

use super::EpochMillis;

/// Where a given metric's value originated. Used to prevent duplicate or
/// contradictory measurements when multiple sources exist (phone + watch).
/// See spec section 33.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorSource {
    PhoneGps,
    PhoneStepSensor,
    PhoneAccelerometer,
    WearOs,
    HealthConnect,
    ManualEntry,
    ServerCorrected,
    Estimated,
}

impl SensorSource {
    /// Higher number = higher trust priority when the same metric arrives
    /// from multiple sources at the same time. Wearable-reported physiology
    /// (heart rate, steps) is trusted over phone-derived estimates.
    pub fn priority(self) -> u8 {
        match self {
            SensorSource::HealthConnect => 100,
            SensorSource::WearOs => 90,
            SensorSource::PhoneStepSensor => 70,
            SensorSource::PhoneGps => 60,
            SensorSource::PhoneAccelerometer => 50,
            SensorSource::ServerCorrected => 40,
            SensorSource::ManualEntry => 30,
            SensorSource::Estimated => 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateSample {
    pub workout_id: String,
    pub bpm: u16,
    pub recorded_at: EpochMillis,
    pub source: SensorSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSample {
    pub workout_id: String,
    pub step_count_delta: u32,
    pub recorded_at: EpochMillis,
    pub source: SensorSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationSample {
    pub workout_id: String,
    pub altitude_meters: f64,
    pub recorded_at: EpochMillis,
    pub source: SensorSource,
}

/// Heart-rate training zones (percent of max HR), and time spent in each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeartRateZone {
    Zone1Rest,
    Zone2Easy,
    Zone3Moderate,
    Zone4Hard,
    Zone5Max,
}

impl HeartRateZone {
    /// Classifies a heart rate given an estimated maximum HR, using the
    /// common 5-zone percentage model. This is descriptive only — the
    /// engine never performs medical interpretation of the result.
    pub fn classify(bpm: u16, max_hr: u16) -> Self {
        if max_hr == 0 {
            return HeartRateZone::Zone1Rest;
        }
        let pct = bpm as f64 / max_hr as f64 * 100.0;
        if pct < 60.0 {
            HeartRateZone::Zone1Rest
        } else if pct < 70.0 {
            HeartRateZone::Zone2Easy
        } else if pct < 80.0 {
            HeartRateZone::Zone3Moderate
        } else if pct < 90.0 {
            HeartRateZone::Zone4Hard
        } else {
            HeartRateZone::Zone5Max
        }
    }
}
