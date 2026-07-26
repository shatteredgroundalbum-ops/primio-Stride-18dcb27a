use serde::{Deserialize, Serialize};

use super::EpochMillis;

/// Activity type as selected by the user (or resolved by auto-detection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Walk,
    Run,
    Hike,
    /// User asked the engine to auto-detect walk vs. run.
    #[default]
    AutoDetect,
}

/// The lifecycle state of a workout. See spec section 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkoutState {
    NotStarted,
    Starting,
    Active,
    Paused,
    Finishing,
    Completed,
    Saved,
    SyncPending,
    Synced,
    Failed,
    Discarded,
}

impl WorkoutState {
    /// Valid forward transitions. Used by the session controller to reject
    /// illegal state changes (e.g. resuming a workout that was discarded).
    pub fn can_transition_to(self, next: WorkoutState) -> bool {
        use WorkoutState::*;
        matches!(
            (self, next),
            (NotStarted, Starting)
                | (Starting, Active)
                | (Starting, Failed)
                | (Active, Paused)
                | (Active, Finishing)
                | (Active, Failed)
                | (Paused, Active)
                | (Paused, Finishing)
                | (Paused, Discarded)
                | (Finishing, Completed)
                | (Finishing, Failed)
                | (Completed, Saved)
                | (Completed, Discarded)
                | (Saved, SyncPending)
                | (SyncPending, Synced)
                | (SyncPending, Failed)
                | (Failed, SyncPending) // retry
        )
    }

    pub fn is_recording(self) -> bool {
        matches!(self, WorkoutState::Active | WorkoutState::Paused)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, WorkoutState::Synced | WorkoutState::Discarded)
    }
}

/// Reason why the auto-pause / classification engine believes the user
/// is or isn't moving under their own power. See sections 9-12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementState {
    Stationary,
    Walking,
    BriskWalking,
    Jogging,
    Running,
    Unknown,
    VehicleMotion,
}

/// A pause/resume event, recorded for accurate active/paused time math.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutPause {
    pub pause_id: String,
    pub workout_id: String,
    pub started_at: EpochMillis,
    pub ended_at: Option<EpochMillis>,
    /// True if the engine triggered this (auto-pause) rather than the user.
    pub is_automatic: bool,
}

impl WorkoutPause {
    pub fn duration_ms(&self, now: EpochMillis) -> i64 {
        self.ended_at.unwrap_or(now) - self.started_at
    }
}

/// The live/finalized workout record. This is the aggregate root the
/// engine mutates as GPS/sensor data streams in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutSession {
    pub workout_id: String,
    pub user_id: String,
    pub requested_activity_type: ActivityType,
    /// The activity type actually used for calorie MET tables etc.
    /// Equals `requested_activity_type` unless that was `AutoDetect`.
    pub resolved_activity_type: ActivityType,
    pub state: WorkoutState,
    pub started_at: EpochMillis,
    pub ended_at: Option<EpochMillis>,

    // Running totals, all in canonical (metric) units.
    pub total_distance_meters: f64,
    pub elapsed_ms: i64,
    pub active_ms: i64,
    pub moving_ms: i64,
    pub paused_ms: i64,

    pub current_speed_mps: f64,
    pub average_speed_mps: f64,
    pub max_speed_mps: f64,

    pub current_pace_sec_per_km: f64,
    pub average_pace_sec_per_km: f64,
    pub fastest_pace_sec_per_km: f64,

    pub step_count: u32,
    pub current_cadence_spm: f64,
    pub average_cadence_spm: f64,

    pub elevation_gain_meters: f64,
    pub elevation_loss_meters: f64,
    pub min_altitude_meters: Option<f64>,
    pub max_altitude_meters: Option<f64>,

    pub calories_estimated: f64,
    pub calorie_estimate_method: String,

    pub movement_state: MovementState,
    pub is_auto_paused: bool,

    pub last_point_at: Option<EpochMillis>,
    pub weight_kg: f64,

    /// Rolling average heart rate (bpm) for the session so far, if any
    /// heart-rate samples have been received. Kept in sync by the
    /// session controller whenever a new HR sample arrives.
    pub average_heart_rate: Option<i32>,
}

impl WorkoutSession {
    pub fn new(
        user_id: impl Into<String>,
        activity_type: ActivityType,
        started_at: EpochMillis,
        weight_kg: f64,
    ) -> Self {
        let resolved = if activity_type == ActivityType::AutoDetect {
            ActivityType::Walk // default assumption until classified
        } else {
            activity_type
        };
        Self {
            workout_id: super::new_id(),
            user_id: user_id.into(),
            requested_activity_type: activity_type,
            resolved_activity_type: resolved,
            state: WorkoutState::NotStarted,
            started_at,
            ended_at: None,
            total_distance_meters: 0.0,
            elapsed_ms: 0,
            active_ms: 0,
            moving_ms: 0,
            paused_ms: 0,
            current_speed_mps: 0.0,
            average_speed_mps: 0.0,
            max_speed_mps: 0.0,
            current_pace_sec_per_km: 0.0,
            average_pace_sec_per_km: 0.0,
            fastest_pace_sec_per_km: 0.0,
            step_count: 0,
            current_cadence_spm: 0.0,
            average_cadence_spm: 0.0,
            elevation_gain_meters: 0.0,
            elevation_loss_meters: 0.0,
            min_altitude_meters: None,
            max_altitude_meters: None,
            calories_estimated: 0.0,
            calorie_estimate_method: "fallback".to_string(),
            movement_state: MovementState::Unknown,
            is_auto_paused: false,
            last_point_at: None,
            weight_kg,
            average_heart_rate: None,
        }
    }
}
