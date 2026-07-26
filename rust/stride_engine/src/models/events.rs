use serde::{Deserialize, Serialize};

use super::EpochMillis;

/// Structured coaching events the engine emits for the AI/rule-based coach
/// and for local voice-coaching announcements. See spec sections 19-20.
///
/// The live workout engine never depends on these being consumed by an
/// online AI — they are plain data the Dart layer can act on locally
/// (e.g. trigger a TTS announcement) even fully offline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachingEventType {
    WorkoutStarted,
    FirstMileCompleted,
    FirstKilometerCompleted,
    HalfwayPoint,
    PaceBelowTarget,
    PaceAboveTarget,
    UserStopped,
    AutoPauseActivated,
    AutoResumeActivated,
    GpsQualityPoor,
    GpsQualityRestored,
    HeartRateUnavailable,
    GoalCompleted,
    PersonalRecordPossible,
    WorkoutEnding,
    HydrationReminder,
    SplitCompleted,
    VehicleMotionSuspected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingEvent {
    pub workout_id: String,
    pub event_type: CoachingEventType,
    pub occurred_at: EpochMillis,
    /// Optional human-readable payload, e.g. `{"split": 3, "pace": 342.1}`.
    pub data: Option<serde_json::Value>,
}

impl CoachingEvent {
    pub fn new(
        workout_id: impl Into<String>,
        event_type: CoachingEventType,
        occurred_at: EpochMillis,
    ) -> Self {
        Self {
            workout_id: workout_id.into(),
            event_type,
            occurred_at,
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Discrete GPS signal quality bucket. See spec section 31.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpsQualityState {
    Unavailable,
    Poor,
    Weak,
    Good,
    Excellent,
}

/// Sync lifecycle for a locally-completed record awaiting upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncQueueState {
    Pending,
    Uploading,
    Synced,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueItem {
    pub workout_id: String,
    pub state: SyncQueueState,
    pub retry_count: u32,
    pub last_attempt_at: Option<EpochMillis>,
    pub error_message: Option<String>,
}
