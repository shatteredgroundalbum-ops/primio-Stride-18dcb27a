use serde::{Deserialize, Serialize};

use super::EpochMillis;

/// A durable snapshot written to local storage (SQLite, via the Dart side)
/// so the workout can be recovered after a crash, OS kill, or restart.
/// See spec section 23.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutCheckpoint {
    pub workout_id: String,
    pub state: super::WorkoutState,
    pub started_at: EpochMillis,
    pub last_point_lat: Option<f64>,
    pub last_point_lon: Option<f64>,
    pub last_point_at: Option<EpochMillis>,
    pub total_distance_meters: f64,
    pub active_ms: i64,
    pub paused_ms: i64,
    pub current_split_number: u32,
    /// Number of route points already flushed to the local route-point
    /// table, so recovery knows where to resume reading/writing.
    pub route_file_position: u64,
    pub checkpoint_at: EpochMillis,
}
