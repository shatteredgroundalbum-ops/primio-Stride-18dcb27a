use serde::{Deserialize, Serialize};

/// How splits are automatically generated during a workout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitTrigger {
    EveryKilometer,
    EveryMile,
    CustomDistanceMeters,
    CustomTimeMs,
    ManualLap,
}

/// A single completed split/lap. See spec section 17.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutSplit {
    pub split_number: u32,
    pub distance_meters: f64,
    pub duration_ms: i64,
    pub average_pace_sec_per_km: f64,
    pub average_speed_mps: f64,
    pub elevation_gain_meters: f64,
    pub elevation_loss_meters: f64,
    pub average_heart_rate_bpm: Option<f64>,
    pub average_cadence_spm: f64,
    pub calories_estimated: f64,
}
