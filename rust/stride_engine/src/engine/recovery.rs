//! Workout recovery system (spec section 23).
//!
//! Pure logic for deciding what checkpoint-based recovery options to offer
//! when the app restarts and finds an unfinished workout row in local
//! storage. Actual checkpoint persistence happens on the Dart/SQLite side;
//! this module only decides *what* to do with a given checkpoint.

use serde::{Deserialize, Serialize};

use crate::models::{EpochMillis, WorkoutCheckpoint, WorkoutState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryOption {
    ResumeWorkout,
    FinishAndSave,
    DiscardWorkout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryDecision {
    pub offered_options: Vec<RecoveryOption>,
    pub recommended_option: RecoveryOption,
    pub reason: String,
}

/// If a checkpoint is older than this, we no longer trust a clean resume
/// and recommend finishing instead (the workout likely continued for a
/// long time without any checkpoint being written, e.g. after a crash).
const STALE_CHECKPOINT_MS: i64 = 6 * 60 * 60 * 1000; // 6 hours

/// Evaluates a recovered checkpoint against the current time and decides
/// what recovery UI to present.
pub fn evaluate_checkpoint(checkpoint: &WorkoutCheckpoint, now_ms: EpochMillis) -> RecoveryDecision {
    let age_ms = now_ms - checkpoint.checkpoint_at;

    if !checkpoint.state.is_recording() {
        // Nothing to recover — it wasn't actively recording when the
        // checkpoint was written (e.g. it had already finished but crashed
        // before cleanup).
        return RecoveryDecision {
            offered_options: vec![RecoveryOption::FinishAndSave, RecoveryOption::DiscardWorkout],
            recommended_option: RecoveryOption::FinishAndSave,
            reason: "Checkpoint was not in an active recording state.".to_string(),
        };
    }

    if age_ms < 0 {
        // Clock skew / corrupted checkpoint timestamp: be conservative.
        return RecoveryDecision {
            offered_options: vec![RecoveryOption::FinishAndSave, RecoveryOption::DiscardWorkout],
            recommended_option: RecoveryOption::DiscardWorkout,
            reason: "Checkpoint timestamp is invalid.".to_string(),
        };
    }

    if age_ms > STALE_CHECKPOINT_MS {
        return RecoveryDecision {
            offered_options: vec![
                RecoveryOption::FinishAndSave,
                RecoveryOption::DiscardWorkout,
            ],
            recommended_option: RecoveryOption::FinishAndSave,
            reason: format!(
                "Checkpoint is {} hours old; resuming live tracking is unreliable.",
                age_ms / 3_600_000
            ),
        };
    }

    // Fresh, actively-recording checkpoint: offer full choice, recommend resume.
    RecoveryDecision {
        offered_options: vec![
            RecoveryOption::ResumeWorkout,
            RecoveryOption::FinishAndSave,
            RecoveryOption::DiscardWorkout,
        ],
        recommended_option: RecoveryOption::ResumeWorkout,
        reason: "Recent checkpoint found from an active workout.".to_string(),
    }
}

/// Builds a fresh checkpoint snapshot. Called by the session controller on
/// every periodic save tick (e.g. every N accepted points or every few
/// seconds) — cheap enough to call frequently since it's just a struct
/// construction; the actual disk write is batched on the Dart side.
#[allow(clippy::too_many_arguments)]
pub fn build_checkpoint(
    workout_id: impl Into<String>,
    state: WorkoutState,
    started_at: EpochMillis,
    last_point: Option<(f64, f64)>,
    last_point_at: Option<EpochMillis>,
    total_distance_meters: f64,
    active_ms: i64,
    paused_ms: i64,
    current_split_number: u32,
    route_file_position: u64,
    now_ms: EpochMillis,
) -> WorkoutCheckpoint {
    WorkoutCheckpoint {
        workout_id: workout_id.into(),
        state,
        started_at,
        last_point_lat: last_point.map(|p| p.0),
        last_point_lon: last_point.map(|p| p.1),
        last_point_at,
        total_distance_meters,
        active_ms,
        paused_ms,
        current_split_number,
        route_file_position,
        checkpoint_at: now_ms,
    }
}

/// Ensures a workout ID is never reused across a discard-then-restart
/// cycle in a way that could create sync duplicates. Given the last known
/// workout ID that existed locally (if any) and a freshly-generated
/// candidate ID, this simply asserts they differ — the actual uniqueness
/// guarantee comes from UUID v4 generation, but this helper documents and
/// enforces the invariant at the boundary where recovery meets "start a
/// new workout".
pub fn assert_new_workout_id_is_unique(previous_id: Option<&str>, candidate_id: &str) -> bool {
    match previous_id {
        Some(prev) => prev != candidate_id,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(state: WorkoutState, age_ms_ago: i64, now: i64) -> WorkoutCheckpoint {
        WorkoutCheckpoint {
            workout_id: "w1".into(),
            state,
            started_at: 0,
            last_point_lat: Some(40.0),
            last_point_lon: Some(-73.0),
            last_point_at: Some(now - age_ms_ago),
            total_distance_meters: 1000.0,
            active_ms: 600_000,
            paused_ms: 0,
            current_split_number: 1,
            route_file_position: 42,
            checkpoint_at: now - age_ms_ago,
        }
    }

    #[test]
    fn fresh_active_checkpoint_recommends_resume() {
        let now = 1_000_000;
        let cp = checkpoint(WorkoutState::Active, 10_000, now);
        let decision = evaluate_checkpoint(&cp, now);
        assert_eq!(decision.recommended_option, RecoveryOption::ResumeWorkout);
        assert!(decision.offered_options.contains(&RecoveryOption::ResumeWorkout));
    }

    #[test]
    fn stale_checkpoint_recommends_finish_not_resume() {
        let now = 100_000_000;
        let cp = checkpoint(WorkoutState::Active, 7 * 3_600_000, now);
        let decision = evaluate_checkpoint(&cp, now);
        assert_eq!(decision.recommended_option, RecoveryOption::FinishAndSave);
        assert!(!decision.offered_options.contains(&RecoveryOption::ResumeWorkout));
    }

    #[test]
    fn non_recording_state_offers_no_resume_option() {
        let now = 1_000_000;
        let cp = checkpoint(WorkoutState::Completed, 5_000, now);
        let decision = evaluate_checkpoint(&cp, now);
        assert!(!decision.offered_options.contains(&RecoveryOption::ResumeWorkout));
    }

    #[test]
    fn negative_age_is_treated_conservatively() {
        let now = 1_000;
        let mut cp = checkpoint(WorkoutState::Active, 0, now);
        cp.checkpoint_at = now + 50_000; // checkpoint "in the future"
        let decision = evaluate_checkpoint(&cp, now);
        assert_eq!(decision.recommended_option, RecoveryOption::DiscardWorkout);
    }

    #[test]
    fn build_checkpoint_populates_all_fields() {
        let cp = build_checkpoint(
            "w42",
            WorkoutState::Active,
            0,
            Some((1.0, 2.0)),
            Some(500),
            1234.5,
            60_000,
            5_000,
            2,
            10,
            65_000,
        );
        assert_eq!(cp.workout_id, "w42");
        assert_eq!(cp.last_point_lat, Some(1.0));
        assert_eq!(cp.route_file_position, 10);
    }

    #[test]
    fn new_workout_id_uniqueness_check() {
        assert!(assert_new_workout_id_is_unique(Some("a"), "b"));
        assert!(!assert_new_workout_id_is_unique(Some("a"), "a"));
        assert!(assert_new_workout_id_is_unique(None, "a"));
    }
}
