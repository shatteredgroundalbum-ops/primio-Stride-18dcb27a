use serde::{Deserialize, Serialize};

use super::{split::WorkoutSplit, EpochMillis};

/// Final, immutable record produced once a workout is validated and
/// finalized. This is what gets handed to the sync layer (Dart) to persist
/// into the `pending_uploads` table and, later, Firestore. See section 27.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutSummary {
    pub workout_id: String,
    pub user_id: String,
    pub activity_type: super::session::ActivityType,
    pub started_at: EpochMillis,
    pub ended_at: EpochMillis,

    pub total_duration_ms: i64,
    pub moving_ms: i64,
    pub paused_ms: i64,

    pub distance_meters: f64,
    pub average_pace_sec_per_km: f64,
    pub average_speed_mps: f64,
    pub max_speed_mps: f64,

    pub calories_estimated: f64,
    pub calorie_estimate_method: String,

    pub step_count: u32,
    pub average_cadence_spm: f64,

    pub average_heart_rate_bpm: Option<f64>,
    pub max_heart_rate_bpm: Option<u16>,
    pub min_heart_rate_bpm: Option<u16>,

    pub elevation_gain_meters: f64,
    pub elevation_loss_meters: f64,

    pub splits: Vec<WorkoutSplit>,

    pub start_latitude: Option<f64>,
    pub start_longitude: Option<f64>,
    pub end_latitude: Option<f64>,
    pub end_longitude: Option<f64>,

    /// Google-style encoded polyline of the (simplified) route, for fast
    /// map previews without loading every raw point.
    pub encoded_polyline: Option<String>,

    pub goal_result: Option<super::goal::GoalProgress>,

    /// True if any segment was flagged as suspicious (e.g. vehicle-speed
    /// motion) during validation. The workout is still saved but flagged
    /// for user review. See section 26.
    pub has_flagged_segments: bool,
    pub validation_warnings: Vec<String>,
    /// True if `engine::validation::validate()` found a hard-blocking data
    /// integrity problem (missing timestamps, corrupt totals, duplicate
    /// workout). The engine still returns a summary either way (it never
    /// performs I/O), but Dart should surface an error and avoid silently
    /// persisting a blocked record as a normal completed workout.
    #[serde(default)]
    pub is_blocked: bool,
    #[serde(default)]
    pub block_reasons: Vec<String>,
}

/// Personal-record categories the engine can detect. See section 28.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonalRecordType {
    FastestMile,
    FastestKilometer,
    LongestDistance,
    LongestDuration,
    GreatestElevationGain,
    HighestAveragePace,
    MostCalories,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalRecord {
    pub record_type: PersonalRecordType,
    pub value: f64,
    pub workout_id: String,
    pub achieved_at: EpochMillis,
}
