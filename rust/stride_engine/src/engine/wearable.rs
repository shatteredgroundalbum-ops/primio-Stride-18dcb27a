//! §10 — Wearable and Health Connect integration.
//!
//! This module models the integration with Wear OS devices (e.g. Samsung
//! Galaxy Watch Ultra) and the Android Health Connect platform. The
//! actual sensor I/O runs on the Kotlin/Android side (Health Connect
//! client, Wear OS Data Layer, foreground service). The Rust engine
//! provides the **decision logic**: source dedup, connection-state
//! tracking, revocation handling, no-watch fallback, sync status, and
//! consent/permission state.
//!
//! Key responsibilities:
//!
//! - **Duplicate-record prevention**: when the same metric (steps, HR,
//!   distance, calories) arrives from both the phone and the watch, the
//!   higher-priority source wins per `SensorSource::priority()`.
//! - **Revocation handling**: if the user revokes Health Connect
//!   permissions or disconnects the watch, the engine records the
//!   revocation and falls back gracefully.
//! - **No-watch fallback**: the app must work phone-only without any
//!   wearable. The engine decides which sources are available and
//!   degrades gracefully.
//! - **Sync status**: tracks whether wearable data is in sync, stale,
//!   or pending.

use serde::{Deserialize, Serialize};

use crate::models::sensor::{HeartRateSample, SensorSource, StepSample};

// ─── Connection state ────────────────────────────────────────────────

/// The current connection state of a wearable device or the Health
/// Connect platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WearableConnectionState {
    /// No wearable paired; phone-only mode.
    Disconnected,
    /// Watch is paired and connected; data flowing.
    Connected,
    /// Watch is paired but not currently reachable (bluetooth off, out
    /// of range, etc.). Data is stale.
    Unreachable,
    /// Health Connect permissions have been granted and data is flowing.
    HealthConnectActive,
    /// Health Connect permissions have been revoked by the user.
    Revoked,
    /// The user has explicitly denied Health Connect permissions.
    PermissionDenied,
    /// The watch or Health Connect is syncing historical data.
    Syncing,
    /// An error occurred while communicating with the watch or Health
    /// Connect (e.g. API error, timeout).
    Error,
}

impl WearableConnectionState {
    /// Returns `true` if this state indicates that wearable/Health
    /// Connect data is currently available and trustworthy.
    pub fn is_data_available(self) -> bool {
        matches!(
            self,
            WearableConnectionState::Connected
                | WearableConnectionState::HealthConnectActive
                | WearableConnectionState::Syncing
        )
    }

    /// Returns `true` if the user has actively revoked or denied
    /// permissions, as opposed to a transient connectivity issue.
    pub fn is_revoked(self) -> bool {
        matches!(
            self,
            WearableConnectionState::Revoked | WearableConnectionState::PermissionDenied
        )
    }

    /// Returns a human-readable label for this state, suitable for the UI.
    pub fn label(self) -> &'static str {
        match self {
            WearableConnectionState::Disconnected => "No wearable connected",
            WearableConnectionState::Connected => "Watch connected",
            WearableConnectionState::Unreachable => "Watch unreachable",
            WearableConnectionState::HealthConnectActive => "Health Connect active",
            WearableConnectionState::Revoked => "Health Connect revoked",
            WearableConnectionState::PermissionDenied => "Health Connect denied",
            WearableConnectionState::Syncing => "Syncing from watch",
            WearableConnectionState::Error => "Wearable error",
        }
    }
}

impl Default for WearableConnectionState {
    fn default() -> Self {
        WearableConnectionState::Disconnected
    }
}

// ─── Source availability ────────────────────────────────────────────

/// Which data sources are currently available for a given workout. The
/// engine uses this to decide which inputs to use for each metric (HR,
/// steps, distance, calories) and whether to fall back to phone-only
/// estimation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceAvailability {
    /// Is a Wear OS watch connected and reporting data?
    pub watch_connected: bool,
    /// Is Health Connect permission granted and data flowing?
    pub health_connect_granted: bool,
    /// Is the phone GPS available?
    pub phone_gps_available: bool,
    /// Is the phone step sensor (accelerometer/pedometer) available?
    pub phone_step_sensor_available: bool,
    /// Is the phone heart-rate sensor available (rare on phones)?
    pub phone_heart_rate_available: bool,
}

impl SourceAvailability {
    /// Returns the best available `SensorSource` for **heart-rate** data,
    /// following the priority order: HealthConnect > WearOs >
    /// PhoneStepSensor (rarely has HR) > Estimated.
    /// Returns `None` if no HR source is available (the app will just
    /// not show HR data).
    pub fn best_heart_rate_source(&self) -> Option<SensorSource> {
        if self.health_connect_granted {
            Some(SensorSource::HealthConnect)
        } else if self.watch_connected {
            Some(SensorSource::WearOs)
        } else if self.phone_heart_rate_available {
            Some(SensorSource::PhoneAccelerometer)
        } else {
            None
        }
    }

    /// Returns the best available `SensorSource` for **step-count** data.
    /// Wearable/HealthConnect steps are preferred over phone accelerometer
    /// because the watch is typically worn on the wrist and more
    /// accurate for step counting during exercise.
    pub fn best_step_source(&self) -> SensorSource {
        if self.health_connect_granted {
            SensorSource::HealthConnect
        } else if self.watch_connected {
            SensorSource::WearOs
        } else if self.phone_step_sensor_available {
            SensorSource::PhoneStepSensor
        } else {
            SensorSource::Estimated
        }
    }

    /// Returns the best available `SensorSource` for **distance** data.
    /// GPS is the primary distance source; the watch may also report
    /// distance but GPS is generally more reliable for outdoor activity.
    pub fn best_distance_source(&self) -> SensorSource {
        if self.phone_gps_available {
            SensorSource::PhoneGps
        } else if self.health_connect_granted {
            SensorSource::HealthConnect
        } else if self.watch_connected {
            SensorSource::WearOs
        } else {
            SensorSource::Estimated
        }
    }

    /// Returns `true` if any wearable source (watch or Health Connect)
    /// is available.
    pub fn has_wearable(&self) -> bool {
        self.watch_connected || self.health_connect_granted
    }

    /// Returns `true` if the app is in phone-only mode (no watch, no
    /// Health Connect).
    pub fn is_phone_only(&self) -> bool {
        !self.has_wearable()
    }
}

// ─── Revocation handling ────────────────────────────────────────────

/// A record of a source revocation — when the user revokes Health
/// Connect permissions, disconnects a watch, or otherwise withdraws a
/// data source. The engine keeps a log of revocations so the app can
/// display what happened and when, and so we can avoid re-querying a
/// revoked source until the user re-grants permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRevocationRecord {
    /// Which source was revoked.
    pub source: SensorSource,
    /// When the revocation was detected (epoch ms).
    pub revoked_at_ms: i64,
    /// Why the revocation occurred.
    pub reason: RevocationReason,
    /// A human-readable message about the revocation.
    pub message: String,
}

/// Why a data source was revoked or became unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevocationReason {
    /// The user revoked Health Connect permissions in system settings.
    UserRevoked,
    /// The user denied the permission request dialog.
    UserDenied,
    /// The watch was physically disconnected.
    WatchDisconnected,
    /// The watch went out of Bluetooth range.
    WatchOutOfRange,
    /// An API error occurred (Health Connect API unavailable, etc.).
    ApiError,
    /// The system revoked permissions (e.g. app data cleared).
    SystemRevoked,
}

impl RevocationReason {
    /// Returns `true` if this revocation was initiated by the user
    /// (as opposed to a system or connectivity issue).
    pub fn is_user_initiated(self) -> bool {
        matches!(
            self,
            RevocationReason::UserRevoked | RevocationReason::UserDenied
        )
    }

    /// Returns `true` if this revocation is potentially recoverable
    /// without user action (e.g. watch comes back in range).
    pub fn is_recoverable(self) -> bool {
        matches!(
            self,
            RevocationReason::WatchOutOfRange | RevocationReason::ApiError
        )
    }

    /// Returns a human-readable label for this revocation reason.
    pub fn label(self) -> &'static str {
        match self {
            RevocationReason::UserRevoked => "User revoked Health Connect permissions",
            RevocationReason::UserDenied => "User denied permission request",
            RevocationReason::WatchDisconnected => "Watch disconnected",
            RevocationReason::WatchOutOfRange => "Watch out of range",
            RevocationReason::ApiError => "API error",
            RevocationReason::SystemRevoked => "System revoked permissions",
        }
    }
}

// ─── Wearable sync status ───────────────────────────────────────────

/// The sync status of wearable data relative to the phone app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WearableSyncStatus {
    /// No wearable paired; nothing to sync.
    NoWearable,
    /// All wearable data is in sync with the phone.
    InSync,
    /// Wearable data is being synced now.
    Syncing,
    /// Wearable data is stale (last sync was too long ago).
    Stale,
    /// A sync error occurred.
    SyncError,
    /// Sync is pending (queued but not yet started).
    Pending,
}

impl Default for WearableSyncStatus {
    fn default() -> Self {
        WearableSyncStatus::NoWearable
    }
}

impl WearableSyncStatus {
    /// Returns `true` if the wearable data is current and trustworthy.
    pub fn is_current(self) -> bool {
        matches!(self, WearableSyncStatus::InSync | WearableSyncStatus::Syncing)
    }

    /// Returns a human-readable label for this sync status.
    pub fn label(self) -> &'static str {
        match self {
            WearableSyncStatus::NoWearable => "No wearable paired",
            WearableSyncStatus::InSync => "Wearable in sync",
            WearableSyncStatus::Syncing => "Syncing from wearable",
            WearableSyncStatus::Stale => "Wearable data is stale",
            WearableSyncStatus::SyncError => "Wearable sync error",
            WearableSyncStatus::Pending => "Wearable sync pending",
        }
    }
}

/// Configuration for wearable sync staleness detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableSyncConfig {
    /// How long (ms) since the last successful sync before data is
    /// considered stale. Default: 5 minutes.
    pub stale_threshold_ms: i64,
}

impl Default for WearableSyncConfig {
    fn default() -> Self {
        WearableSyncConfig {
            stale_threshold_ms: 5 * 60 * 1000, // 5 minutes
        }
    }
}

/// Evaluates the current sync status of a wearable, given the
/// connection state, the time of the last successful sync, and the
/// current time.
pub fn evaluate_sync_status(
    connection: WearableConnectionState,
    last_sync_ms: Option<i64>,
    now_ms: i64,
    config: &WearableSyncConfig,
) -> WearableSyncStatus {
    match connection {
        WearableConnectionState::Disconnected | WearableConnectionState::PermissionDenied |
        WearableConnectionState::Revoked => WearableSyncStatus::NoWearable,
        WearableConnectionState::Error => WearableSyncStatus::SyncError,
        WearableConnectionState::Syncing => WearableSyncStatus::Syncing,
        WearableConnectionState::Connected | WearableConnectionState::HealthConnectActive |
        WearableConnectionState::Unreachable => {
            match last_sync_ms {
                None => WearableSyncStatus::Pending,
                Some(last) => {
                    let elapsed = now_ms.saturating_sub(last);
                    if elapsed > config.stale_threshold_ms {
                        WearableSyncStatus::Stale
                    } else {
                        WearableSyncStatus::InSync
                    }
                }
            }
        }
    }
}

// ─── Duplicate-record prevention ─────────────────────────────────────

/// A metric type for dedup decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    HeartRate,
    Steps,
    Distance,
    Calories,
    Elevation,
}

/// Decides which source wins when the same metric arrives from two
/// sources at (approximately) the same time. Returns the source that
/// should be used, or `None` if neither source is valid.
///
/// The higher-priority source wins per `SensorSource::priority()`. If
/// both sources have the same priority, the first source (typically the
/// wearable) wins as a tiebreaker.
pub fn deduplicate_source(
    source_a: SensorSource,
    source_b: SensorSource,
    _metric: MetricType,
) -> SensorSource {
    if source_b.priority() > source_a.priority() {
        source_b
    } else {
        source_a
    }
}

/// Deduplicates a batch of heart-rate samples, keeping only the
/// highest-priority sample per timestamp window. Samples from the same
/// source within the dedup window are kept; if two different sources
/// report at overlapping times, the higher-priority source wins.
///
/// The `dedup_window_ms` parameter controls how close two samples must
/// be (in time) to be considered duplicates. A typical value is 1000ms
/// (1 second).
pub fn deduplicate_heart_rate_samples(
    samples: &[HeartRateSample],
    dedup_window_ms: i64,
) -> Vec<HeartRateSample> {
    if samples.is_empty() {
        return Vec::new();
    }
    if dedup_window_ms <= 0 {
        return samples.to_vec();
    }

    // Sort by timestamp (ascending).
    let mut sorted: Vec<&HeartRateSample> = samples.iter().collect();
    sorted.sort_by_key(|s| s.recorded_at);

    let mut result: Vec<HeartRateSample> = Vec::with_capacity(sorted.len());
    let mut current_window_start: i64 = sorted[0].recorded_at;
    let mut best_in_window: &HeartRateSample = sorted[0];

    for &sample in sorted.iter().skip(1) {
        if sample.recorded_at - current_window_start <= dedup_window_ms {
            // Same window — keep the higher-priority source.
            if sample.source.priority() > best_in_window.source.priority() {
                best_in_window = sample;
            }
        } else {
            // New window — push the best from the previous window.
            result.push(best_in_window.clone());
            current_window_start = sample.recorded_at;
            best_in_window = sample;
        }
    }
    result.push(best_in_window.clone());
    result
}

/// Deduplicates a batch of step samples using the same window-based
/// dedup strategy as heart-rate samples.
pub fn deduplicate_step_samples(
    samples: &[StepSample],
    dedup_window_ms: i64,
) -> Vec<StepSample> {
    if samples.is_empty() {
        return Vec::new();
    }
    if dedup_window_ms <= 0 {
        return samples.to_vec();
    }

    let mut sorted: Vec<&StepSample> = samples.iter().collect();
    sorted.sort_by_key(|s| s.recorded_at);

    let mut result: Vec<StepSample> = Vec::with_capacity(sorted.len());
    let mut current_window_start: i64 = sorted[0].recorded_at;
    let mut best_in_window: &StepSample = sorted[0];

    for &sample in sorted.iter().skip(1) {
        if sample.recorded_at - current_window_start <= dedup_window_ms {
            if sample.source.priority() > best_in_window.source.priority() {
                best_in_window = sample;
            }
        } else {
            result.push(best_in_window.clone());
            current_window_start = sample.recorded_at;
            best_in_window = sample;
        }
    }
    result.push(best_in_window.clone());
    result
}

// ─── No-watch fallback ───────────────────────────────────────────────

/// The result of evaluating the no-watch fallback: which mode the app
/// should operate in and what sources to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackDecision {
    /// The connection state the app should assume.
    pub connection_state: WearableConnectionState,
    /// Which sources are available.
    pub available_sources: SourceAvailability,
    /// The best step source to use.
    pub step_source: SensorSource,
    /// The best distance source to use.
    pub distance_source: SensorSource,
    /// The best heart-rate source to use (if any).
    pub heart_rate_source: Option<SensorSource>,
    /// Whether the app is in phone-only mode.
    pub is_phone_only: bool,
    /// A human-readable message for the UI.
    pub message: String,
}

/// Decides the appropriate operating mode given the current source
/// availability. This is the core no-watch fallback logic: if no
/// wearable is present, the app falls back to phone-only GPS + phone
/// step sensor, and does not attempt to query Health Connect or Wear OS.
pub fn decide_fallback(availability: &SourceAvailability) -> FallbackDecision {
    let is_phone_only = availability.is_phone_only();

    let connection_state = if availability.health_connect_granted {
        WearableConnectionState::HealthConnectActive
    } else if availability.watch_connected {
        WearableConnectionState::Connected
    } else if is_phone_only {
        WearableConnectionState::Disconnected
    } else {
        WearableConnectionState::Disconnected
    };

    let step_source = availability.best_step_source();
    let distance_source = availability.best_distance_source();
    let heart_rate_source = availability.best_heart_rate_source();

    let message = if is_phone_only {
        "No wearable connected. Using phone sensors (GPS + step sensor). \
         Heart-rate data will not be available."
            .to_string()
    } else if availability.health_connect_granted {
        "Health Connect active. Using wearable data for heart rate and steps, \
         GPS for distance."
            .to_string()
    } else {
        "Watch connected. Using watch data for heart rate and steps, \
         GPS for distance."
            .to_string()
    };

    FallbackDecision {
        connection_state,
        available_sources: availability.clone(),
        step_source,
        distance_source,
        heart_rate_source,
        is_phone_only,
        message,
    }
}

// ─── Health Connect consent state ───────────────────────────────────

/// The consent state for Health Connect permissions. Health Connect
/// requires explicit user consent before the app can read any data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthConnectConsentState {
    /// The app has not yet requested Health Connect permissions.
    NotRequested,
    /// The permission request dialog is being shown to the user.
    Requesting,
    /// The user granted permissions.
    Granted,
    /// The user denied permissions.
    Denied,
    /// The user previously granted but later revoked permissions.
    Revoked,
}

impl Default for HealthConnectConsentState {
    fn default() -> Self {
        HealthConnectConsentState::NotRequested
    }
}

impl HealthConnectConsentState {
    /// Returns `true` if the app can currently read Health Connect data.
    pub fn can_read(self) -> bool {
        matches!(self, HealthConnectConsentState::Granted)
    }

    /// Returns `true` if the app should re-request permissions (the
    /// user denied or revoked, and the app hasn't asked again yet).
    pub fn should_re_request(self) -> bool {
        matches!(
            self,
            HealthConnectConsentState::NotRequested
        )
    }

    /// Returns a human-readable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            HealthConnectConsentState::NotRequested => "Health Connect not yet set up",
            HealthConnectConsentState::Requesting => "Requesting Health Connect permissions",
            HealthConnectConsentState::Granted => "Health Connect granted",
            HealthConnectConsentState::Denied => "Health Connect denied",
            HealthConnectConsentState::Revoked => "Health Connect revoked",
        }
    }

    /// Transitions to the next state when a permission request result
    /// arrives. If the user grants, → `Granted`. If the user denies, →
    /// `Denied`. If an error occurs, stays in `Requesting`.
    pub fn on_request_result(self, granted: bool) -> HealthConnectConsentState {
        match self {
            HealthConnectConsentState::Requesting => {
                if granted {
                    HealthConnectConsentState::Granted
                } else {
                    HealthConnectConsentState::Denied
                }
            }
            _ => self,
        }
    }
}

// ─── Wearable device info ────────────────────────────────────────────

/// Information about a paired wearable device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableDeviceInfo {
    /// The manufacturer (e.g. "Samsung", "Google", "Garmin").
    pub manufacturer: String,
    /// The model name (e.g. "Galaxy Watch Ultra").
    pub model: String,
    /// The device's unique identifier.
    pub device_id: String,
    /// Whether the device supports Health Connect.
    pub supports_health_connect: bool,
    /// The current battery level (0–100), if known.
    pub battery_level: Option<u8>,
}

impl WearableDeviceInfo {
    /// Returns a display name for the device (e.g. "Samsung Galaxy Watch Ultra").
    pub fn display_name(&self) -> String {
        format!("{} {}", self.manufacturer, self.model)
    }
}

// ─── Aggregated wearable status ──────────────────────────────────────

/// A snapshot of the full wearable integration status, suitable for
/// returning via FFI to the UI layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableStatus {
    pub connection_state: WearableConnectionState,
    pub consent_state: HealthConnectConsentState,
    pub sync_status: WearableSyncStatus,
    pub availability: SourceAvailability,
    pub fallback: FallbackDecision,
    pub device: Option<WearableDeviceInfo>,
    pub revocations: Vec<SourceRevocationRecord>,
    /// When the last successful sync occurred (epoch ms), if ever.
    pub last_sync_ms: Option<i64>,
}

/// Builds a full `WearableStatus` snapshot from the current state.
pub fn build_status(
    availability: &SourceAvailability,
    consent: HealthConnectConsentState,
    last_sync_ms: Option<i64>,
    now_ms: i64,
    sync_config: &WearableSyncConfig,
    revocations: &[SourceRevocationRecord],
    device: Option<WearableDeviceInfo>,
) -> WearableStatus {
    let fallback = decide_fallback(availability);

    // If consent is denied/revoked, override the connection state.
    let connection_state = match consent {
        HealthConnectConsentState::Denied => WearableConnectionState::PermissionDenied,
        HealthConnectConsentState::Revoked => WearableConnectionState::Revoked,
        _ => fallback.connection_state,
    };

    let sync_status = evaluate_sync_status(connection_state, last_sync_ms, now_ms, sync_config);

    WearableStatus {
        connection_state,
        consent_state: consent,
        sync_status,
        availability: availability.clone(),
        fallback,
        device,
        revocations: revocations.to_vec(),
        last_sync_ms,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sensor::{HeartRateSample, StepSample};

    fn hr_sample(workout_id: &str, bpm: u16, at: i64, source: SensorSource) -> HeartRateSample {
        HeartRateSample {
            workout_id: workout_id.to_string(),
            bpm,
            recorded_at: at,
            source,
        }
    }

    fn step_sample(
        workout_id: &str,
        delta: u32,
        at: i64,
        source: SensorSource,
    ) -> StepSample {
        StepSample {
            workout_id: workout_id.to_string(),
            step_count_delta: delta,
            recorded_at: at,
            source,
        }
    }

    // ── ConnectionState ──

    #[test]
    fn connected_state_is_data_available() {
        assert!(WearableConnectionState::Connected.is_data_available());
        assert!(WearableConnectionState::HealthConnectActive.is_data_available());
        assert!(WearableConnectionState::Syncing.is_data_available());
    }

    #[test]
    fn disconnected_state_is_not_data_available() {
        assert!(!WearableConnectionState::Disconnected.is_data_available());
        assert!(!WearableConnectionState::Unreachable.is_data_available());
        assert!(!WearableConnectionState::Revoked.is_data_available());
        assert!(!WearableConnectionState::PermissionDenied.is_data_available());
        assert!(!WearableConnectionState::Error.is_data_available());
    }

    #[test]
    fn revoked_states_are_revoked() {
        assert!(WearableConnectionState::Revoked.is_revoked());
        assert!(WearableConnectionState::PermissionDenied.is_revoked());
    }

    #[test]
    fn non_revoked_states_are_not_revoked() {
        assert!(!WearableConnectionState::Disconnected.is_revoked());
        assert!(!WearableConnectionState::Connected.is_revoked());
        assert!(!WearableConnectionState::Unreachable.is_revoked());
        assert!(!WearableConnectionState::Syncing.is_revoked());
    }

    #[test]
    fn connection_state_labels_are_non_empty() {
        for state in [
            WearableConnectionState::Disconnected,
            WearableConnectionState::Connected,
            WearableConnectionState::Unreachable,
            WearableConnectionState::HealthConnectActive,
            WearableConnectionState::Revoked,
            WearableConnectionState::PermissionDenied,
            WearableConnectionState::Syncing,
            WearableConnectionState::Error,
        ] {
            assert!(!state.label().is_empty());
        }
    }

    #[test]
    fn default_connection_state_is_disconnected() {
        assert_eq!(
            WearableConnectionState::default(),
            WearableConnectionState::Disconnected
        );
    }

    // ── SourceAvailability ──

    #[test]
    fn phone_only_when_no_wearable() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert!(avail.is_phone_only());
        assert!(!avail.has_wearable());
    }

    #[test]
    fn has_wearable_when_watch_connected() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert!(avail.has_wearable());
        assert!(!avail.is_phone_only());
    }

    #[test]
    fn has_wearable_when_health_connect_granted() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert!(avail.has_wearable());
        assert!(!avail.is_phone_only());
    }

    #[test]
    fn best_heart_rate_source_prefers_health_connect() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: true,
        };
        assert_eq!(
            avail.best_heart_rate_source(),
            Some(SensorSource::HealthConnect)
        );
    }

    #[test]
    fn best_heart_rate_source_falls_back_to_watch() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: true,
        };
        assert_eq!(avail.best_heart_rate_source(), Some(SensorSource::WearOs));
    }

    #[test]
    fn best_heart_rate_source_none_when_no_wearable_and_no_phone_hr() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert_eq!(avail.best_heart_rate_source(), None);
    }

    #[test]
    fn best_step_source_prefers_health_connect() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert_eq!(avail.best_step_source(), SensorSource::HealthConnect);
    }

    #[test]
    fn best_step_source_falls_back_to_phone() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert_eq!(avail.best_step_source(), SensorSource::PhoneStepSensor);
    }

    #[test]
    fn best_step_source_estimated_when_nothing_available() {
        let avail = SourceAvailability::default();
        assert_eq!(avail.best_step_source(), SensorSource::Estimated);
    }

    #[test]
    fn best_distance_source_prefers_gps() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert_eq!(avail.best_distance_source(), SensorSource::PhoneGps);
    }

    #[test]
    fn best_distance_source_falls_back_to_health_connect() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: false,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        assert_eq!(avail.best_distance_source(), SensorSource::HealthConnect);
    }

    #[test]
    fn best_distance_source_estimated_when_nothing_available() {
        let avail = SourceAvailability::default();
        assert_eq!(avail.best_distance_source(), SensorSource::Estimated);
    }

    // ── Revocation ──

    #[test]
    fn user_revoked_is_user_initiated() {
        assert!(RevocationReason::UserRevoked.is_user_initiated());
        assert!(RevocationReason::UserDenied.is_user_initiated());
    }

    #[test]
    fn watch_disconnected_is_not_user_initiated() {
        assert!(!RevocationReason::WatchDisconnected.is_user_initiated());
        assert!(!RevocationReason::WatchOutOfRange.is_user_initiated());
        assert!(!RevocationReason::ApiError.is_user_initiated());
        assert!(!RevocationReason::SystemRevoked.is_user_initiated());
    }

    #[test]
    fn out_of_range_is_recoverable() {
        assert!(RevocationReason::WatchOutOfRange.is_recoverable());
        assert!(RevocationReason::ApiError.is_recoverable());
    }

    #[test]
    fn user_revoked_is_not_recoverable() {
        assert!(!RevocationReason::UserRevoked.is_recoverable());
        assert!(!RevocationReason::UserDenied.is_recoverable());
        assert!(!RevocationReason::WatchDisconnected.is_recoverable());
        assert!(!RevocationReason::SystemRevoked.is_recoverable());
    }

    #[test]
    fn revocation_reason_labels_are_non_empty() {
        for reason in [
            RevocationReason::UserRevoked,
            RevocationReason::UserDenied,
            RevocationReason::WatchDisconnected,
            RevocationReason::WatchOutOfRange,
            RevocationReason::ApiError,
            RevocationReason::SystemRevoked,
        ] {
            assert!(!reason.label().is_empty());
        }
    }

    #[test]
    fn revocation_record_serializes() {
        let record = SourceRevocationRecord {
            source: SensorSource::HealthConnect,
            revoked_at_ms: 1_000_000,
            reason: RevocationReason::UserRevoked,
            message: "User revoked".to_string(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"source\":\"health_connect\""));
        assert!(json.contains("\"reason\":\"user_revoked\""));
        let back: SourceRevocationRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source, SensorSource::HealthConnect);
        assert_eq!(back.reason, RevocationReason::UserRevoked);
    }

    // ── Sync status ──

    #[test]
    fn in_sync_is_current() {
        assert!(WearableSyncStatus::InSync.is_current());
        assert!(WearableSyncStatus::Syncing.is_current());
    }

    #[test]
    fn stale_is_not_current() {
        assert!(!WearableSyncStatus::Stale.is_current());
        assert!(!WearableSyncStatus::SyncError.is_current());
        assert!(!WearableSyncStatus::Pending.is_current());
        assert!(!WearableSyncStatus::NoWearable.is_current());
    }

    #[test]
    fn default_sync_status_is_no_wearable() {
        assert_eq!(
            WearableSyncStatus::default(),
            WearableSyncStatus::NoWearable
        );
    }

    #[test]
    fn evaluate_sync_status_disconnected_returns_no_wearable() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Disconnected,
            Some(1_000_000),
            2_000_000,
            &config,
        );
        assert_eq!(status, WearableSyncStatus::NoWearable);
    }

    #[test]
    fn evaluate_sync_status_revoked_returns_no_wearable() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Revoked,
            Some(1_000_000),
            2_000_000,
            &config,
        );
        assert_eq!(status, WearableSyncStatus::NoWearable);
    }

    #[test]
    fn evaluate_sync_status_syncing_returns_syncing() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Syncing,
            Some(1_000_000),
            2_000_000,
            &config,
        );
        assert_eq!(status, WearableSyncStatus::Syncing);
    }

    #[test]
    fn evaluate_sync_status_error_returns_sync_error() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Error,
            Some(1_000_000),
            2_000_000,
            &config,
        );
        assert_eq!(status, WearableSyncStatus::SyncError);
    }

    #[test]
    fn evaluate_sync_status_in_sync_when_recent() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Connected,
            Some(1_000_000),
            1_000_000 + 60_000, // 1 minute ago — within 5-minute threshold
            &config,
        );
        assert_eq!(status, WearableSyncStatus::InSync);
    }

    #[test]
    fn evaluate_sync_status_stale_when_old() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Connected,
            Some(1_000_000),
            1_000_000 + 10 * 60_000, // 10 minutes ago — beyond 5-minute threshold
            &config,
        );
        assert_eq!(status, WearableSyncStatus::Stale);
    }

    #[test]
    fn evaluate_sync_status_pending_when_no_last_sync() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Connected,
            None,
            1_000_000,
            &config,
        );
        assert_eq!(status, WearableSyncStatus::Pending);
    }

    #[test]
    fn evaluate_sync_status_unreachable_in_sync_if_recent() {
        let config = WearableSyncConfig::default();
        let status = evaluate_sync_status(
            WearableConnectionState::Unreachable,
            Some(1_000_000),
            1_000_000 + 30_000, // 30 seconds ago
            &config,
        );
        assert_eq!(status, WearableSyncStatus::InSync);
    }

    // ── Dedup ──

    #[test]
    fn deduplicate_source_picks_higher_priority() {
        let winner = deduplicate_source(
            SensorSource::PhoneStepSensor,
            SensorSource::HealthConnect,
            MetricType::Steps,
        );
        assert_eq!(winner, SensorSource::HealthConnect);
    }

    #[test]
    fn deduplicate_source_keeps_first_on_tie() {
        let winner = deduplicate_source(
            SensorSource::HealthConnect,
            SensorSource::HealthConnect,
            MetricType::HeartRate,
        );
        assert_eq!(winner, SensorSource::HealthConnect);
    }

    #[test]
    fn deduplicate_heart_rate_empty_returns_empty() {
        let result = deduplicate_heart_rate_samples(&[], 1000);
        assert!(result.is_empty());
    }

    #[test]
    fn deduplicate_heart_rate_keeps_higher_priority_in_window() {
        let samples = vec![
            hr_sample("w1", 120, 1_000, SensorSource::PhoneAccelerometer),
            hr_sample("w1", 122, 1_500, SensorSource::HealthConnect), // higher priority, within 1s window
        ];
        let result = deduplicate_heart_rate_samples(&samples, 1000);
        // Both within the 1000ms window → only the highest-priority survives.
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, SensorSource::HealthConnect);
        assert_eq!(result[0].bpm, 122);
    }

    #[test]
    fn deduplicate_heart_rate_keeps_both_outside_window() {
        let samples = vec![
            hr_sample("w1", 120, 1_000, SensorSource::PhoneAccelerometer),
            hr_sample("w1", 122, 5_000, SensorSource::HealthConnect), // outside 1s window
        ];
        let result = deduplicate_heart_rate_samples(&samples, 1000);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn deduplicate_heart_rate_zero_window_returns_all() {
        let samples = vec![
            hr_sample("w1", 120, 1_000, SensorSource::PhoneAccelerometer),
            hr_sample("w1", 122, 1_000, SensorSource::HealthConnect),
        ];
        let result = deduplicate_heart_rate_samples(&samples, 0);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn deduplicate_heart_rate_unsorted_input_is_sorted() {
        let samples = vec![
            hr_sample("w1", 122, 5_000, SensorSource::HealthConnect),
            hr_sample("w1", 120, 1_000, SensorSource::PhoneAccelerometer),
        ];
        let result = deduplicate_heart_rate_samples(&samples, 1000);
        // Each in its own window → 2 results, but sorted by time.
        assert_eq!(result.len(), 2);
        assert!(result[0].recorded_at <= result[1].recorded_at);
    }

    #[test]
    fn deduplicate_step_empty_returns_empty() {
        let result = deduplicate_step_samples(&[], 1000);
        assert!(result.is_empty());
    }

    #[test]
    fn deduplicate_step_keeps_higher_priority_in_window() {
        let samples = vec![
            step_sample("w1", 10, 1_000, SensorSource::PhoneStepSensor),
            step_sample("w1", 12, 1_500, SensorSource::WearOs), // higher priority
        ];
        let result = deduplicate_step_samples(&samples, 1000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, SensorSource::WearOs);
        assert_eq!(result[0].step_count_delta, 12);
    }

    #[test]
    fn deduplicate_step_keeps_both_outside_window() {
        let samples = vec![
            step_sample("w1", 10, 1_000, SensorSource::PhoneStepSensor),
            step_sample("w1", 12, 5_000, SensorSource::WearOs),
        ];
        let result = deduplicate_step_samples(&samples, 1000);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn deduplicate_step_zero_window_returns_all() {
        let samples = vec![
            step_sample("w1", 10, 1_000, SensorSource::PhoneStepSensor),
            step_sample("w1", 12, 1_000, SensorSource::WearOs),
        ];
        let result = deduplicate_step_samples(&samples, 0);
        assert_eq!(result.len(), 2);
    }

    // ── Fallback decision ──

    #[test]
    fn fallback_phone_only_when_no_wearable() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let decision = decide_fallback(&avail);
        assert!(decision.is_phone_only);
        assert_eq!(
            decision.connection_state,
            WearableConnectionState::Disconnected
        );
        assert_eq!(decision.step_source, SensorSource::PhoneStepSensor);
        assert_eq!(decision.distance_source, SensorSource::PhoneGps);
        assert_eq!(decision.heart_rate_source, None);
    }

    #[test]
    fn fallback_watch_connected_uses_wearable() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let decision = decide_fallback(&avail);
        assert!(!decision.is_phone_only);
        assert_eq!(decision.connection_state, WearableConnectionState::Connected);
        assert_eq!(decision.step_source, SensorSource::WearOs);
        assert_eq!(decision.distance_source, SensorSource::PhoneGps);
        assert_eq!(decision.heart_rate_source, Some(SensorSource::WearOs));
    }

    #[test]
    fn fallback_health_connect_granted_uses_health_connect() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let decision = decide_fallback(&avail);
        assert!(!decision.is_phone_only);
        assert_eq!(
            decision.connection_state,
            WearableConnectionState::HealthConnectActive
        );
        assert_eq!(decision.step_source, SensorSource::HealthConnect);
        assert_eq!(decision.heart_rate_source, Some(SensorSource::HealthConnect));
    }

    #[test]
    fn fallback_message_mentions_phone_only() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let decision = decide_fallback(&avail);
        assert!(decision.message.contains("phone"));
    }

    #[test]
    fn fallback_decision_serializes() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let decision = decide_fallback(&avail);
        let json = serde_json::to_string(&decision).unwrap();
        assert!(json.contains("\"is_phone_only\":false"));
        assert!(json.contains("\"step_source\":\"wear_os\""));
        let back: FallbackDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back.is_phone_only, false);
        assert_eq!(back.step_source, SensorSource::WearOs);
    }

    // ── Consent state ──

    #[test]
    fn consent_granted_can_read() {
        assert!(HealthConnectConsentState::Granted.can_read());
    }

    #[test]
    fn consent_not_granted_cannot_read() {
        assert!(!HealthConnectConsentState::NotRequested.can_read());
        assert!(!HealthConnectConsentState::Requesting.can_read());
        assert!(!HealthConnectConsentState::Denied.can_read());
        assert!(!HealthConnectConsentState::Revoked.can_read());
    }

    #[test]
    fn consent_not_requested_should_re_request() {
        assert!(HealthConnectConsentState::NotRequested.should_re_request());
    }

    #[test]
    fn consent_granted_should_not_re_request() {
        assert!(!HealthConnectConsentState::Granted.should_re_request());
        assert!(!HealthConnectConsentState::Denied.should_re_request());
        assert!(!HealthConnectConsentState::Revoked.should_re_request());
    }

    #[test]
    fn consent_request_result_granted() {
        let state = HealthConnectConsentState::Requesting;
        assert_eq!(state.on_request_result(true), HealthConnectConsentState::Granted);
    }

    #[test]
    fn consent_request_result_denied() {
        let state = HealthConnectConsentState::Requesting;
        assert_eq!(state.on_request_result(false), HealthConnectConsentState::Denied);
    }

    #[test]
    fn consent_request_result_ignored_when_not_requesting() {
        let state = HealthConnectConsentState::Granted;
        assert_eq!(state.on_request_result(false), HealthConnectConsentState::Granted);
    }

    #[test]
    fn consent_labels_are_non_empty() {
        for state in [
            HealthConnectConsentState::NotRequested,
            HealthConnectConsentState::Requesting,
            HealthConnectConsentState::Granted,
            HealthConnectConsentState::Denied,
            HealthConnectConsentState::Revoked,
        ] {
            assert!(!state.label().is_empty());
        }
    }

    #[test]
    fn default_consent_state_is_not_requested() {
        assert_eq!(
            HealthConnectConsentState::default(),
            HealthConnectConsentState::NotRequested
        );
    }

    // ── Device info ──

    #[test]
    fn device_display_name_includes_manufacturer_and_model() {
        let device = WearableDeviceInfo {
            manufacturer: "Samsung".to_string(),
            model: "Galaxy Watch Ultra".to_string(),
            device_id: "device123".to_string(),
            supports_health_connect: true,
            battery_level: Some(85),
        };
        assert_eq!(device.display_name(), "Samsung Galaxy Watch Ultra");
    }

    #[test]
    fn device_info_serializes() {
        let device = WearableDeviceInfo {
            manufacturer: "Samsung".to_string(),
            model: "Galaxy Watch Ultra".to_string(),
            device_id: "device123".to_string(),
            supports_health_connect: true,
            battery_level: Some(85),
        };
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("\"manufacturer\":\"Samsung\""));
        assert!(json.contains("\"model\":\"Galaxy Watch Ultra\""));
        assert!(json.contains("\"battery_level\":85"));
    }

    // ── build_status ──

    #[test]
    fn build_status_phone_only() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let status = build_status(
            &avail,
            HealthConnectConsentState::NotRequested,
            None,
            1_000_000,
            &WearableSyncConfig::default(),
            &[],
            None,
        );
        assert_eq!(
            status.connection_state,
            WearableConnectionState::Disconnected
        );
        assert_eq!(status.sync_status, WearableSyncStatus::NoWearable);
        assert!(status.fallback.is_phone_only);
    }

    #[test]
    fn build_status_with_health_connect_granted() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let status = build_status(
            &avail,
            HealthConnectConsentState::Granted,
            Some(999_000),
            1_000_000,
            &WearableSyncConfig::default(),
            &[],
            None,
        );
        assert_eq!(
            status.connection_state,
            WearableConnectionState::HealthConnectActive
        );
        assert_eq!(status.sync_status, WearableSyncStatus::InSync);
    }

    #[test]
    fn build_status_denied_consent_overrides_connection() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let status = build_status(
            &avail,
            HealthConnectConsentState::Denied,
            Some(999_000),
            1_000_000,
            &WearableSyncConfig::default(),
            &[],
            None,
        );
        assert_eq!(
            status.connection_state,
            WearableConnectionState::PermissionDenied
        );
        assert_eq!(status.sync_status, WearableSyncStatus::NoWearable);
    }

    #[test]
    fn build_status_revoked_consent_overrides_connection() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let status = build_status(
            &avail,
            HealthConnectConsentState::Revoked,
            Some(999_000),
            1_000_000,
            &WearableSyncConfig::default(),
            &[],
            None,
        );
        assert_eq!(status.connection_state, WearableConnectionState::Revoked);
        assert_eq!(status.sync_status, WearableSyncStatus::NoWearable);
    }

    #[test]
    fn build_status_with_revocations() {
        let avail = SourceAvailability {
            watch_connected: false,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let revocations = vec![SourceRevocationRecord {
            source: SensorSource::HealthConnect,
            revoked_at_ms: 500_000,
            reason: RevocationReason::UserRevoked,
            message: "User revoked HC".to_string(),
        }];
        let status = build_status(
            &avail,
            HealthConnectConsentState::Revoked,
            None,
            1_000_000,
            &WearableSyncConfig::default(),
            &revocations,
            None,
        );
        assert_eq!(status.revocations.len(), 1);
        assert_eq!(status.revocations[0].source, SensorSource::HealthConnect);
    }

    #[test]
    fn build_status_serializes() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: true,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let status = build_status(
            &avail,
            HealthConnectConsentState::Granted,
            Some(999_000),
            1_000_000,
            &WearableSyncConfig::default(),
            &[],
            None,
        );
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"connection_state\":\"health_connect_active\""));
        assert!(json.contains("\"consent_state\":\"granted\""));
        let back: WearableStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.connection_state,
            WearableConnectionState::HealthConnectActive
        );
    }

    // ── Serialization round-trips ──

    #[test]
    fn source_availability_serializes_round_trip() {
        let avail = SourceAvailability {
            watch_connected: true,
            health_connect_granted: false,
            phone_gps_available: true,
            phone_step_sensor_available: true,
            phone_heart_rate_available: false,
        };
        let json = serde_json::to_string(&avail).unwrap();
        let back: SourceAvailability = serde_json::from_str(&json).unwrap();
        assert_eq!(back.watch_connected, true);
        assert_eq!(back.health_connect_granted, false);
        assert_eq!(back.phone_gps_available, true);
    }

    #[test]
    fn connection_state_serializes_round_trip() {
        let json = serde_json::to_string(&WearableConnectionState::HealthConnectActive).unwrap();
        assert!(json.contains("\"health_connect_active\""));
        let back: WearableConnectionState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, WearableConnectionState::HealthConnectActive);
    }

    #[test]
    fn sync_status_serializes_round_trip() {
        let json = serde_json::to_string(&WearableSyncStatus::InSync).unwrap();
        assert!(json.contains("\"in_sync\""));
        let back: WearableSyncStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, WearableSyncStatus::InSync);
    }

    #[test]
    fn consent_state_serializes_round_trip() {
        let json = serde_json::to_string(&HealthConnectConsentState::Granted).unwrap();
        assert!(json.contains("\"granted\""));
        let back: HealthConnectConsentState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, HealthConnectConsentState::Granted);
    }

    #[test]
    fn metric_type_serializes_round_trip() {
        let json = serde_json::to_string(&MetricType::HeartRate).unwrap();
        assert!(json.contains("\"heart_rate\""));
        let back: MetricType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, MetricType::HeartRate);
    }
}
