//! Cloud synchronization engine (spec section 3).
//!
//! Pure decision logic for the local → cloud sync pipeline. The actual
//! network I/O (Firestore reads/writes, Cloud Storage uploads) happens on
//! the Dart/Kotlin side; this module decides *when* to retry, *how* to
//! resolve conflicts, *whether* a record is a duplicate, and *what state*
//! the sync queue item should transition to.
//!
//! Key responsibilities:
//!   - Exponential backoff with jitter for retry scheduling
//!   - Conflict resolution (last-write-wins vs server-authoritative)
//!   - Duplicate prevention (idempotent upsert keyed by workout_id)
//!   - Upload progress tracking
//!   - Deletion propagation (tombstone lifecycle)
//!   - Last-sync timestamp management
//!   - Device-to-device sync state coordination

use serde::{Deserialize, Serialize};

use crate::models::EpochMillis;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of retry attempts before a sync item is marked
/// permanently failed and requires manual intervention.
pub const MAX_RETRY_COUNT: u32 = 5;

/// Base delay for exponential backoff (1 second, in milliseconds).
const BASE_BACKOFF_MS: i64 = 1_000;

/// Maximum backoff cap (10 minutes, in milliseconds). Retries never wait
/// longer than this regardless of attempt number.
const MAX_BACKOFF_MS: i64 = 10 * 60 * 1_000;

/// Multiplier for exponential backoff. delay = base * 2^attempt, capped.
const BACKOFF_MULTIPLIER: f64 = 2.0;

/// Jitter range: ±25% of the computed delay. Prevents thundering-herd
/// retry storms when many devices come back online simultaneously.
const JITTER_FRACTION: f64 = 0.25;

/// Tombstones older than this (and already synced) are garbage-collected
/// from the pending_deletions table. 7 days.
pub const TOMBSTONE_GC_AGE_MS: i64 = 7 * 24 * 60 * 60 * 1_000;

// ---------------------------------------------------------------------------
// Sync operation types
// ---------------------------------------------------------------------------

/// What kind of sync operation is being performed for a given item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncOperation {
    /// Upload a completed workout summary to Firestore.
    UploadSummary,
    /// Upload a route file to Cloud Storage.
    UploadRoute,
    /// Delete a workout from the cloud (tombstone propagation).
    DeleteWorkout,
    /// Download updates from cloud to local (device-to-device sync).
    DownloadUpdates,
}

/// The outcome of a sync attempt for a single item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAttemptResult {
    /// Operation succeeded — item is fully synced.
    Success,
    /// Operation failed with a retryable error (network timeout, 5xx,
    /// transient Firestore error). Will be retried with backoff.
    RetryableFailure,
    /// Operation failed with a permanent error (403 Forbidden, 404 Not
    /// Found, malformed data). No retry — requires user/developer
    /// intervention.
    PermanentFailure,
    /// Operation failed because of a conflict — the cloud version is
    /// newer or different. Needs conflict resolution.
    Conflict,
}

// ---------------------------------------------------------------------------
// Conflict resolution
// ---------------------------------------------------------------------------

/// Strategy for resolving a sync conflict between local and cloud versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolutionStrategy {
    /// The version with the later `updated_at` timestamp wins.
    LastWriteWins,
    /// The cloud/server version is always authoritative; local changes
    /// are discarded.
    ServerAuthoritative,
    /// The local version wins (used when the local device is the
    /// originator and the cloud version is stale from another device's
    /// incomplete sync).
    LocalWins,
    /// Conflict requires user interaction to resolve (rare; used when
    /// both versions have meaningful but different data).
    ManualMerge,
}

/// Metadata about a conflicting item, used to decide resolution strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub workout_id: String,
    /// Timestamp of the local version's last update.
    pub local_updated_at: EpochMillis,
    /// Timestamp of the cloud version's last update.
    pub cloud_updated_at: EpochMillis,
    /// ID of the device that last wrote the cloud version, if known.
    pub cloud_device_id: Option<String>,
    /// ID of the local device.
    pub local_device_id: String,
}

/// The decision returned by conflict resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictDecision {
    /// Use the local version (push local to cloud).
    KeepLocal,
    /// Use the cloud version (pull cloud to local, discard local changes).
    KeepCloud,
    /// Both versions are identical — no action needed, just mark synced.
    NoConflict,
    /// Requires user interaction.
    RequireManualMerge,
}

/// Resolves a sync conflict using the given strategy.
///
/// The default strategy is `LastWriteWins` with a 1-second tolerance
/// window — if both timestamps are within 1 second of each other, the
/// versions are considered identical (no conflict).
pub fn resolve_conflict(
    info: &ConflictInfo,
    strategy: ConflictResolutionStrategy,
) -> ConflictDecision {
    match strategy {
        ConflictResolutionStrategy::ServerAuthoritative => {
            ConflictDecision::KeepCloud
        }
        ConflictResolutionStrategy::LocalWins => ConflictDecision::KeepLocal,
        ConflictResolutionStrategy::ManualMerge => {
            ConflictDecision::RequireManualMerge
        }
        ConflictResolutionStrategy::LastWriteWins => {
            let diff = info.local_updated_at - info.cloud_updated_at;
            // 1-second tolerance: if both timestamps are within 1s, treat
            // as identical (clock skew between devices).
            if diff.abs() < 1_000 {
                ConflictDecision::NoConflict
            } else if diff > 0 {
                // Local is newer.
                ConflictDecision::KeepLocal
            } else {
                // Cloud is newer.
                ConflictDecision::KeepCloud
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Exponential backoff with jitter
// ---------------------------------------------------------------------------

/// Computes the retry delay (in milliseconds) for a given attempt number
/// using exponential backoff with jitter.
///
/// Formula: `delay = min(base * 2^attempt, max) * (1 ± jitter)`
///
/// - `attempt` is 0-indexed (0 = first retry, 1 = second retry, etc.)
/// - Returns 0 for attempt 0 if `immediate_first_retry` is true.
/// - The jitter is deterministic given the seed, so tests are reproducible.
///   In production, the caller should pass a random seed.
pub fn compute_backoff_delay(
    attempt: u32,
    jitter_seed: u64,
) -> i64 {
    if attempt == 0 {
        // First retry is immediate (no delay).
        return 0;
    }

    // Exponential component: base * 2^attempt, capped at MAX_BACKOFF_MS.
    let exponential = BASE_BACKOFF_MS as f64
        * (BACKOFF_MULTIPLIER.powi(attempt as i32));
    let capped = exponential.min(MAX_BACKOFF_MS as f64);

    // Jitter: deterministic pseudo-random in [-JITTER_FRACTION, +JITTER_FRACTION]
    // derived from the seed. This avoids thundering-herd without requiring
    // a rand crate dependency.
    let jitter_raw = pseudo_random(jitter_seed.wrapping_add(attempt as u64));
    let jitter_factor = 1.0 + (jitter_raw * 2.0 - 1.0) * JITTER_FRACTION;

    (capped * jitter_factor) as i64
}

/// Simple deterministic pseudo-random in [0.0, 1.0) from a u64 seed.
/// Uses a splitmix64-style mixing step to ensure good bit dispersion
/// even for small seed values. Not cryptographically secure, but
/// sufficient for jitter scheduling where reproducibility is desirable.
fn pseudo_random(seed: u64) -> f64 {
    let mut x = seed;
    // splitmix64 mixing — produces well-distributed values even from
    // small inputs.
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x = x ^ (x >> 31);
    (x as f64) / (u64::MAX as f64)
}

/// Decides whether a sync item should be retried or permanently failed,
/// based on the attempt result and current retry count.
///
/// Returns `Some(delay_ms)` if the item should be retried after the given
/// delay, or `None` if it should be permanently failed (max retries
/// exceeded or permanent failure).
pub fn decide_retry(
    result: SyncAttemptResult,
    current_retry_count: u32,
    jitter_seed: u64,
) -> Option<i64> {
    match result {
        SyncAttemptResult::Success => None, // No retry needed.
        SyncAttemptResult::PermanentFailure => None, // No retry.
        SyncAttemptResult::Conflict => {
            // Conflicts get retried after conflict resolution, but only
            // up to MAX_RETRY_COUNT.
            if current_retry_count >= MAX_RETRY_COUNT {
                None
            } else {
                Some(compute_backoff_delay(current_retry_count, jitter_seed))
            }
        }
        SyncAttemptResult::RetryableFailure => {
            if current_retry_count >= MAX_RETRY_COUNT {
                None // Exceeded max retries — permanent failure.
            } else {
                Some(compute_backoff_delay(current_retry_count, jitter_seed))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Duplicate detection
// ---------------------------------------------------------------------------

/// Checks whether a local workout is a duplicate of an existing cloud
/// workout. Duplicates are identified by workout_id (the primary key)
/// — if the same workout_id already exists in the cloud with matching
/// content, it's a duplicate and should not be re-uploaded.
///
/// Returns `true` if the workout should be skipped (it's already synced
/// with identical content), `false` if it should be uploaded.
pub fn is_duplicate(
    local_workout_id: &str,
    local_updated_at: EpochMillis,
    cloud_workout_id: Option<&str>,
    cloud_updated_at: Option<EpochMillis>,
) -> bool {
    match (cloud_workout_id, cloud_updated_at) {
        (Some(cloud_id), Some(cloud_ts)) => {
            cloud_id == local_workout_id && cloud_ts == local_updated_at
        }
        _ => false,
    }
}

/// Determines whether a workout should be upserted (inserted or updated)
/// or skipped. This is the idempotent upsert check: if the cloud already
/// has this exact workout, skip; otherwise, upsert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpsertDecision {
    /// Cloud doesn't have this workout — insert it.
    Insert,
    /// Cloud has an older version — update it.
    Update,
    /// Cloud has the identical version — skip.
    Skip,
}

pub fn decide_upsert(
    local_workout_id: &str,
    local_updated_at: EpochMillis,
    cloud_workout_id: Option<&str>,
    cloud_updated_at: Option<EpochMillis>,
) -> UpsertDecision {
    match (cloud_workout_id, cloud_updated_at) {
        (None, _) => UpsertDecision::Insert,
        (Some(cloud_id), None) if cloud_id == local_workout_id => {
            UpsertDecision::Update
        }
        (Some(cloud_id), Some(cloud_ts)) if cloud_id == local_workout_id => {
            if cloud_ts == local_updated_at {
                UpsertDecision::Skip
            } else {
                UpsertDecision::Update
            }
        }
        _ => UpsertDecision::Insert,
    }
}

// ---------------------------------------------------------------------------
// Upload progress
// ---------------------------------------------------------------------------

/// Progress tracking for a multi-step upload (route file + summary +
/// daily summary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadStep {
    /// Not started yet.
    NotStarted,
    /// Uploading route file to Cloud Storage.
    UploadingRoute,
    /// Uploading workout summary to Firestore.
    UploadingSummary,
    /// Updating daily summary.
    UpdatingDailySummary,
    /// All steps complete.
    Complete,
    /// Failed at some step.
    Failed,
}

/// Represents the progress of a single workout's upload pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    pub workout_id: String,
    pub current_step: UploadStep,
    pub steps_completed: u8,
    pub total_steps: u8,
    pub error: Option<String>,
}

impl UploadProgress {
    pub fn new(workout_id: impl Into<String>) -> Self {
        Self {
            workout_id: workout_id.into(),
            current_step: UploadStep::NotStarted,
            steps_completed: 0,
            total_steps: 3, // route + summary + daily summary
            error: None,
        }
    }

    /// Fraction of steps completed, in [0.0, 1.0].
    pub fn fraction(&self) -> f64 {
        if self.total_steps == 0 {
            return 1.0;
        }
        self.steps_completed as f64 / self.total_steps as f64
    }

    /// Advances to the next step, incrementing the completed count.
    pub fn advance_to(&mut self, step: UploadStep) {
        if self.current_step != UploadStep::Failed {
            self.steps_completed =
                (self.steps_completed + 1).min(self.total_steps);
        }
        self.current_step = step;
    }

    /// Marks the upload as failed at the current step.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.current_step = UploadStep::Failed;
        self.error = Some(error.into());
    }

    /// Marks the upload as fully complete.
    pub fn complete(&mut self) {
        self.steps_completed = self.total_steps;
        self.current_step = UploadStep::Complete;
        self.error = None;
    }
}

// ---------------------------------------------------------------------------
// Tombstone (deletion propagation) lifecycle
// ---------------------------------------------------------------------------

/// State of a deletion tombstone in the pending_deletions queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TombstoneState {
    /// Deletion is queued but hasn't been attempted yet.
    Pending,
    /// Cloud deletion is in progress.
    Deleting,
    /// Cloud deletion succeeded — tombstone can be GC'd after age.
    Synced,
    /// Cloud deletion failed — will be retried.
    Failed,
}

/// A deletion tombstone record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionTombstone {
    pub workout_id: String,
    pub user_id: String,
    pub state: TombstoneState,
    pub retry_count: u32,
    pub queued_at: EpochMillis,
    pub last_attempt_at: Option<EpochMillis>,
    pub error_message: Option<String>,
}

/// Decides whether a tombstone should be retried, given its current state
/// and retry count.
pub fn tombstone_should_retry(
    state: TombstoneState,
    retry_count: u32,
) -> bool {
    match state {
        TombstoneState::Pending => true,
        TombstoneState::Failed => retry_count < MAX_RETRY_COUNT,
        TombstoneState::Deleting => false, // Already in progress.
        TombstoneState::Synced => false,   // Already done.
    }
}

/// Decides whether a synced tombstone is old enough to be garbage-collected.
pub fn tombstone_should_gc(
    state: TombstoneState,
    synced_at: EpochMillis,
    now_ms: EpochMillis,
) -> bool {
    if state != TombstoneState::Synced {
        return false;
    }
    (now_ms - synced_at) > TOMBSTONE_GC_AGE_MS
}

// ---------------------------------------------------------------------------
// Last-sync timestamp management
// ---------------------------------------------------------------------------

/// Tracks the last successful sync timestamp per category.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastSyncTimestamps {
    /// Last time workout summaries were successfully synced.
    pub summaries_last_synced_at: Option<EpochMillis>,
    /// Last time route files were successfully synced.
    pub routes_last_synced_at: Option<EpochMillis>,
    /// Last time deletions were propagated.
    pub deletions_last_synced_at: Option<EpochMillis>,
    /// Last time cloud-to-local download sync was performed.
    pub download_last_synced_at: Option<EpochMillis>,
}

impl LastSyncTimestamps {
    /// Updates the timestamp for the given operation type.
    pub fn mark_synced(
        &mut self,
        operation: SyncOperation,
        now_ms: EpochMillis,
    ) {
        match operation {
            SyncOperation::UploadSummary => {
                self.summaries_last_synced_at = Some(now_ms);
            }
            SyncOperation::UploadRoute => {
                self.routes_last_synced_at = Some(now_ms);
            }
            SyncOperation::DeleteWorkout => {
                self.deletions_last_synced_at = Some(now_ms);
            }
            SyncOperation::DownloadUpdates => {
                self.download_last_synced_at = Some(now_ms);
            }
        }
    }

    /// Returns the timestamp for the given operation type, if any.
    pub fn get(&self, operation: SyncOperation) -> Option<EpochMillis> {
        match operation {
            SyncOperation::UploadSummary => self.summaries_last_synced_at,
            SyncOperation::UploadRoute => self.routes_last_synced_at,
            SyncOperation::DeleteWorkout => self.deletions_last_synced_at,
            SyncOperation::DownloadUpdates => self.download_last_synced_at,
        }
    }

    /// Returns true if a download sync is due (hasn't been done in the
    /// given interval, or has never been done).
    pub fn download_is_due(&self, now_ms: EpochMillis, interval_ms: i64) -> bool {
        match self.download_last_synced_at {
            None => true,
            Some(last) => (now_ms - last) >= interval_ms,
        }
    }
}

// ---------------------------------------------------------------------------
// Device-to-device sync
// ---------------------------------------------------------------------------/// Identifies which device "owns" a workout (was the recording device).
/// Used in device-to-device sync to determine which device should upload
/// a given workout and which should only download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncState {
    /// The device that recorded the workout.
    pub recording_device_id: String,
    /// The current device's ID.
    pub current_device_id: String,
    /// Whether this workout has been uploaded to the cloud by any device.
    pub is_uploaded: bool,
    /// Whether this device has downloaded the latest cloud version.
    pub is_downloaded: bool,
}

/// Decides what action the current device should take for a workout in
/// device-to-device sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceSyncAction {
    /// This device recorded the workout and should upload it.
    Upload,
    /// This device should download the cloud version.
    Download,
    /// No action needed (already synced on this device).
    NoAction,
}

pub fn decide_device_sync_action(state: &DeviceSyncState) -> DeviceSyncAction {
    if state.current_device_id == state.recording_device_id {
        // This is the recording device.
        if state.is_uploaded {
            DeviceSyncAction::NoAction
        } else {
            DeviceSyncAction::Upload
        }
    } else {
        // This is a different device.
        if state.is_downloaded {
            DeviceSyncAction::NoAction
        } else {
            DeviceSyncAction::Download
        }
    }
}

// ---------------------------------------------------------------------------
// Sync coordinator: top-level decision for a sync pass
// ---------------------------------------------------------------------------

/// The overall state of the sync engine for a sync pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPassResult {
    /// Number of items successfully synced.
    pub synced_count: u32,
    /// Number of items that failed and will be retried.
    pub retried_count: u32,
    /// Number of items that permanently failed.
    pub permanently_failed_count: u32,
    /// Number of conflicts detected.
    pub conflict_count: u32,
    /// Number of duplicates skipped.
    pub skipped_duplicates: u32,
    /// Number of deletions propagated.
    pub deletions_propagated: u32,
    /// Whether a download sync was performed.
    pub download_performed: bool,
    /// Timestamp of this sync pass.
    pub sync_pass_at: EpochMillis,
}

impl SyncPassResult {
    pub fn new(now_ms: EpochMillis) -> Self {
        Self {
            synced_count: 0,
            retried_count: 0,
            permanently_failed_count: 0,
            conflict_count: 0,
            skipped_duplicates: 0,
            deletions_propagated: 0,
            download_performed: false,
            sync_pass_at: now_ms,
        }
    }

    /// Total number of items processed.
    pub fn total_processed(&self) -> u32 {
        self.synced_count
            + self.retried_count
            + self.permanently_failed_count
            + self.conflict_count
            + self.skipped_duplicates
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Backoff ---

    #[test]
    fn first_retry_is_immediate() {
        assert_eq!(compute_backoff_delay(0, 42), 0);
    }

    #[test]
    fn backoff_grows_exponentially() {
        let d1 = compute_backoff_delay(1, 0);
        let d2 = compute_backoff_delay(2, 0);
        let d3 = compute_backoff_delay(3, 0);
        // Each should be roughly double the previous (within jitter range).
        assert!(d2 > d1);
        assert!(d3 > d2);
    }

    #[test]
    fn backoff_is_capped() {
        // Very high attempt number should not exceed MAX_BACKOFF_MS
        // (adjusted for jitter, so allow some margin).
        let delay = compute_backoff_delay(20, 0);
        assert!(
            delay <= MAX_BACKOFF_MS + MAX_BACKOFF_MS / 4,
            "delay {} exceeded cap {}",
            delay,
            MAX_BACKOFF_MS
        );
    }

    #[test]
    fn backoff_jitter_varies_with_seed() {
        let d1 = compute_backoff_delay(3, 12345);
        let d2 = compute_backoff_delay(3, 67890);
        // Different seeds should produce different delays (very likely,
        // though not guaranteed — use widely-spaced seeds to make it
        // virtually certain).
        assert_ne!(d1, d2);
    }

    // --- Retry decision ---

    #[test]
    fn retryable_failure_within_max_retries_returns_delay() {
        let result = decide_retry(SyncAttemptResult::RetryableFailure, 2, 42);
        assert!(result.is_some());
        assert!(result.unwrap() >= 0);
    }

    #[test]
    fn retryable_failure_at_max_retries_returns_none() {
        let result =
            decide_retry(SyncAttemptResult::RetryableFailure, MAX_RETRY_COUNT, 42);
        assert!(result.is_none());
    }

    #[test]
    fn permanent_failure_returns_none() {
        let result = decide_retry(SyncAttemptResult::PermanentFailure, 0, 42);
        assert!(result.is_none());
    }

    #[test]
    fn success_returns_none() {
        let result = decide_retry(SyncAttemptResult::Success, 0, 42);
        assert!(result.is_none());
    }

    // --- Conflict resolution ---

    #[test]
    fn last_write_wins_local_newer() {
        let info = ConflictInfo {
            workout_id: "w1".into(),
            local_updated_at: 2000,
            cloud_updated_at: 1000,
            cloud_device_id: Some("device-b".into()),
            local_device_id: "device-a".into(),
        };
        let decision =
            resolve_conflict(&info, ConflictResolutionStrategy::LastWriteWins);
        assert_eq!(decision, ConflictDecision::KeepLocal);
    }

    #[test]
    fn last_write_wins_cloud_newer() {
        let info = ConflictInfo {
            workout_id: "w1".into(),
            local_updated_at: 1000,
            cloud_updated_at: 2000,
            cloud_device_id: Some("device-b".into()),
            local_device_id: "device-a".into(),
        };
        let decision =
            resolve_conflict(&info, ConflictResolutionStrategy::LastWriteWins);
        assert_eq!(decision, ConflictDecision::KeepCloud);
    }

    #[test]
    fn last_write_wins_within_tolerance_is_no_conflict() {
        let info = ConflictInfo {
            workout_id: "w1".into(),
            local_updated_at: 1000,
            cloud_updated_at: 1500, // 500ms diff, within 1s tolerance
            cloud_device_id: Some("device-b".into()),
            local_device_id: "device-a".into(),
        };
        let decision =
            resolve_conflict(&info, ConflictResolutionStrategy::LastWriteWins);
        assert_eq!(decision, ConflictDecision::NoConflict);
    }

    #[test]
    fn server_authoritative_always_keeps_cloud() {
        let info = ConflictInfo {
            workout_id: "w1".into(),
            local_updated_at: 9999,
            cloud_updated_at: 1,
            cloud_device_id: None,
            local_device_id: "device-a".into(),
        };
        let decision = resolve_conflict(
            &info,
            ConflictResolutionStrategy::ServerAuthoritative,
        );
        assert_eq!(decision, ConflictDecision::KeepCloud);
    }

    // --- Duplicate detection ---

    #[test]
    fn identical_workout_is_duplicate() {
        assert!(is_duplicate("w1", 1000, Some("w1"), Some(1000)));
    }

    #[test]
    fn different_timestamp_is_not_duplicate() {
        assert!(!is_duplicate("w1", 1000, Some("w1"), Some(2000)));
    }

    #[test]
    fn no_cloud_version_is_not_duplicate() {
        assert!(!is_duplicate("w1", 1000, None, None));
    }

    #[test]
    fn different_id_is_not_duplicate() {
        assert!(!is_duplicate("w1", 1000, Some("w2"), Some(1000)));
    }

    // --- Upsert decision ---

    #[test]
    fn upsert_insert_when_no_cloud_version() {
        assert_eq!(
            decide_upsert("w1", 1000, None, None),
            UpsertDecision::Insert
        );
    }

    #[test]
    fn upsert_skip_when_identical() {
        assert_eq!(
            decide_upsert("w1", 1000, Some("w1"), Some(1000)),
            UpsertDecision::Skip
        );
    }

    #[test]
    fn upsert_update_when_cloud_is_older() {
        assert_eq!(
            decide_upsert("w1", 2000, Some("w1"), Some(1000)),
            UpsertDecision::Update
        );
    }

    // --- Upload progress ---

    #[test]
    fn upload_progress_starts_at_zero() {
        let progress = UploadProgress::new("w1");
        assert_eq!(progress.steps_completed, 0);
        assert_eq!(progress.fraction(), 0.0);
        assert_eq!(progress.current_step, UploadStep::NotStarted);
    }

    #[test]
    fn upload_progress_advances_correctly() {
        let mut progress = UploadProgress::new("w1");
        progress.advance_to(UploadStep::UploadingRoute);
        assert_eq!(progress.steps_completed, 1);
        assert!((progress.fraction() - 0.333).abs() < 0.01);
        progress.advance_to(UploadStep::UploadingSummary);
        assert_eq!(progress.steps_completed, 2);
        progress.advance_to(UploadStep::UpdatingDailySummary);
        assert_eq!(progress.steps_completed, 3);
        progress.complete();
        assert_eq!(progress.current_step, UploadStep::Complete);
        assert_eq!(progress.fraction(), 1.0);
    }

    #[test]
    fn upload_progress_fail_sets_error() {
        let mut progress = UploadProgress::new("w1");
        progress.advance_to(UploadStep::UploadingRoute);
        progress.fail("network timeout");
        assert_eq!(progress.current_step, UploadStep::Failed);
        assert_eq!(progress.error, Some("network timeout".to_string()));
    }

    // --- Tombstone lifecycle ---

    #[test]
    fn pending_tombstone_should_retry() {
        assert!(tombstone_should_retry(TombstoneState::Pending, 0));
    }

    #[test]
    fn synced_tombstone_should_not_retry() {
        assert!(!tombstone_should_retry(TombstoneState::Synced, 0));
    }

    #[test]
    fn failed_tombstone_within_max_retries_should_retry() {
        assert!(tombstone_should_retry(TombstoneState::Failed, 3));
    }

    #[test]
    fn failed_tombstone_at_max_retries_should_not_retry() {
        assert!(!tombstone_should_retry(
            TombstoneState::Failed,
            MAX_RETRY_COUNT
        ));
    }

    #[test]
    fn synced_old_tombstone_should_be_gc() {
        let now = 10_000_000;
        let synced_at = now - TOMBSTONE_GC_AGE_MS - 1;
        assert!(tombstone_should_gc(
            TombstoneState::Synced,
            synced_at,
            now
        ));
    }

    #[test]
    fn synced_recent_tombstone_should_not_be_gc() {
        let now = 10_000_000;
        let synced_at = now - 1000;
        assert!(!tombstone_should_gc(
            TombstoneState::Synced,
            synced_at,
            now
        ));
    }

    // --- Last sync timestamps ---

    #[test]
    fn last_sync_timestamps_mark_and_get() {
        let mut ts = LastSyncTimestamps::default();
        ts.mark_synced(SyncOperation::UploadSummary, 5000);
        assert_eq!(ts.get(SyncOperation::UploadSummary), Some(5000));
        assert_eq!(ts.get(SyncOperation::UploadRoute), None);
    }

    #[test]
    fn download_is_due_when_never_synced() {
        let ts = LastSyncTimestamps::default();
        assert!(ts.download_is_due(10000, 5000));
    }

    #[test]
    fn download_is_due_when_interval_elapsed() {
        let mut ts = LastSyncTimestamps::default();
        ts.mark_synced(SyncOperation::DownloadUpdates, 10000);
        assert!(ts.download_is_due(20000, 5000));
    }

    #[test]
    fn download_not_due_within_interval() {
        let mut ts = LastSyncTimestamps::default();
        ts.mark_synced(SyncOperation::DownloadUpdates, 10000);
        assert!(!ts.download_is_due(12000, 5000));
    }

    // --- Device sync ---

    #[test]
    fn recording_device_should_upload_if_not_uploaded() {
        let state = DeviceSyncState {
            recording_device_id: "device-a".into(),
            current_device_id: "device-a".into(),
            is_uploaded: false,
            is_downloaded: false,
        };
        assert_eq!(decide_device_sync_action(&state), DeviceSyncAction::Upload);
    }

    #[test]
    fn recording_device_no_action_if_already_uploaded() {
        let state = DeviceSyncState {
            recording_device_id: "device-a".into(),
            current_device_id: "device-a".into(),
            is_uploaded: true,
            is_downloaded: true,
        };
        assert_eq!(
            decide_device_sync_action(&state),
            DeviceSyncAction::NoAction
        );
    }

    #[test]
    fn non_recording_device_should_download_if_not_downloaded() {
        let state = DeviceSyncState {
            recording_device_id: "device-a".into(),
            current_device_id: "device-b".into(),
            is_uploaded: true,
            is_downloaded: false,
        };
        assert_eq!(
            decide_device_sync_action(&state),
            DeviceSyncAction::Download
        );
    }

    #[test]
    fn non_recording_device_no_action_if_already_downloaded() {
        let state = DeviceSyncState {
            recording_device_id: "device-a".into(),
            current_device_id: "device-b".into(),
            is_uploaded: true,
            is_downloaded: true,
        };
        assert_eq!(
            decide_device_sync_action(&state),
            DeviceSyncAction::NoAction
        );
    }

    // --- Sync pass result ---

    #[test]
    fn sync_pass_result_counts_total_processed() {
        let mut result = SyncPassResult::new(1000);
        result.synced_count = 3;
        result.retried_count = 2;
        result.skipped_duplicates = 1;
        assert_eq!(result.total_processed(), 6);
    }
}
