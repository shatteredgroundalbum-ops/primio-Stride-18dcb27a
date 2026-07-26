//! C-ABI FFI boundary consumed by the Flutter app via `dart:ffi`.
//!
//! Design:
//!   - Every live workout gets a Rust-owned `WorkoutSessionController`
//!     stored in a process-wide registry, keyed by an opaque `i64` handle
//!     handed back to Dart. Dart never sees a raw pointer to Rust memory —
//!     just an integer it passes back on every subsequent call. This keeps
//!     the boundary simple and avoids any Dart-side unsafe pointer misuse.
//!   - All structured data crosses the boundary as UTF-8 JSON C strings
//!     (`*mut c_char`). This avoids needing to hand-maintain a parallel
//!     Dart struct layout for every Rust type — `serde_json` is the single
//!     source of truth for the wire format on both sides.
//!   - Every entry point is wrapped in `std::panic::catch_unwind` so a bug
//!     in the engine can never crash the whole Flutter/Dart host process;
//!     a panic is converted into a normal `{"ok": false, "error": ...}`
//!     JSON response instead.
//!   - Every JSON string returned to Dart is heap-allocated by Rust via
//!     `CString::into_raw`. Dart MUST call `stride_free_string` on every
//!     pointer it receives from this module once it's done reading it, or
//!     the memory leaks. Strings Dart passes *into* Rust (e.g. as
//!     `*const c_char` parameters) remain owned by Dart and are only
//!     borrowed here (read via `CStr`), never freed by Rust.
//!
//! Response envelope (always JSON):
//!   success: `{"ok": true, "data": <T>}`
//!   failure: `{"ok": false, "error": "<message>"}`

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::json;

use crate::engine::achievements::{self, AchievementHistory};
use crate::engine::account::{
    self, AccountStatus, AuthProvider, DeletionResult, DeletionScope,
    LoginContext, LoginHistory, PasswordValidationResult,
    SessionState, SensitiveAction, SuspiciousLoginResult,
    UserDataCategory,
};
use crate::engine::battery::{self, BatteryContext, WorkoutActivityLevel};
use crate::engine::background::{
    self, BackgroundExecutionContext, BackgroundExecutionDecision, BackgroundStatus,
    BackgroundTrackingPreference, CheckpointBatteryContext, CheckpointSchedule,
    ForegroundServiceCommand, ForegroundServiceState, ForegroundServiceTransition,
    InterruptionAction, InterruptionEvent, PowerMode, ProcessKillRecoveryDecision,
    WorkoutPhase,
};
use crate::engine::notifications::{
    self, CoachingAnnouncementKind, CoachingDeliveryMode, NotificationCategory,
    NotificationContent, NotificationDecision, NotificationDecisionContext,
    NotificationPermission, NotificationPreferences, NotificationPriority,
    NotificationStatus, NotificationType, QuietHoursConfig, TimezoneContext,
    VoiceCoachingConfig,
};
use crate::engine::error_states::{
    self, EngineHealthLabel, EngineHealthStatus, ErrorCategory, ErrorContext,
    ErrorRegistry, ErrorRegistryEntry, ErrorSeverity, ErrorState,
    ErrorStateTransition, RecoveryAction, RecoveryStrategy, RetryPolicy,
};
use crate::engine::calories::{self, CalorieEstimate, CalorieEstimateResult, CalorieInputs};
use crate::engine::controller::{SessionConfig, WorkoutSessionController};
use crate::engine::coaching_plan::{
    self, AiAvailability, AiOperation, DayPlan, EscalationReason,
    ExperienceLevel, PainType, PlanAdjustmentInput, UserFeedback,
    WeeklyPlan, WorkoutSummaryInput,
};
use crate::engine::permissions::{self, PermissionContext, PermissionState};
use crate::engine::personal_records::{self, PriorBests};
use crate::engine::route_format::{
    self, RouteFileFormat, RouteFormatInput, RouteSummary,
    RouteFileSyncState,
};
use crate::engine::security::{
    self, AccessContext, AccessDecision, AppCheckState, FieldRule,
    FieldValidationResult, FirestoreCollection, PlayIntegrityVerdict,
    RateLimitBucket, RateLimitCategory, RateLimitResult, FirebaseEnvironment,
};
use crate::engine::maps::{
    self, GpsAccuracyLevel, MapViewType, OfflineRegion, SavedRoute,
    TileCoord, TileProvider,
};
use crate::engine::sync::{
    self, ConflictInfo, ConflictResolutionStrategy, DeviceSyncState,
};
use crate::engine::units::{self, ConversionKind, DistanceUnit};
use crate::engine::validation::{self, ValidationInput};
use crate::engine::music::{
    self, AudioFocusEvent, AudioFocusState, CoachingInteropState, CoachingRequest,
    MusicMode, MusicSource, MusicStatus, NetworkLossDecision, NetworkState,
    PlaybackAction, PlaybackCommand, PlaybackState, Playlist, Track, TrackFeedback,
    FeedbackRecord, RemoteControlSource, TransitionResult,
};
use crate::engine::wearable::{
    self, FallbackDecision, HealthConnectConsentState, MetricType,
    SourceAvailability, SourceRevocationRecord, WearableConnectionState,
    WearableDeviceInfo, WearableStatus, WearableSyncConfig, WearableSyncStatus,
};
use crate::engine::testing::{
    self, TestCategory, TestConfig, TestEnvironment, TestLayer, TestReport,
    TestResult, TestSeverity, TestStatus, TestSuite, CoverageMetrics,
    DeviceProfile, GpsQuality, LocationType,
    RealDeviceScenario, ScenarioResult, TestRegistry,
};
use crate::engine::monitoring::{
    self, AiCostMetrics, AiOperationCount, AiOperationType, Alert, AlertSeverity,
    AlertThresholds, AlertType, CrashReport, CrashSeverity, LogBuffer, LogEntry,
    LogLevel, MetricPair, MonitoringCategory, MonitoringConfig,
    MonitoredService, PerformanceTrace, ReleaseHealthDashboard,
    ReleaseHealthStatus, ServiceStatus, SyncFailureMetrics, UptimeMonitor,
    KeyValuePair,
};
use crate::engine::backups::{
    self, BackupConfig, BackupFrequency, BackupManifest, BackupRecord, BackupStatus,
    BackupType, ExportDataCategory, ExportFormat, ExportRequest, ExportResult,
    ExportStatus, MigrationPlan, MigrationStep, MigrationStepType, MigrationStatus,
    RecoveryTestSchedule, RestoreRequest, RestoreResult, RestoreScope, RestoreStatus,
    RetentionDataType, RetentionPolicy, RetentionRule, RollbackPlan, RollbackStep,
    RollbackStatus, StorageClass,
};
use crate::engine::privacy::{
    self, PrivacyPolicy, PrivacyPolicyVersion, PrivacyPolicySection,
    TermsOfService, TermsOfServiceVersion, HealthDisclaimer,
    DisclaimerStatus, DisclosureType, DataDisclosure,
    PrivacyDataType, PrivacyRetentionRule, PrivacyRetentionPolicy,
    AccountDeletionPolicy, SupportContact, ConsentType, ConsentStatus,
    ConsentRecord, ConsentRegistry, PrivacyExportStatus,
    PrivacyExportRequest, PrivacyDeletionStatus, PrivacyDeletionRequest,
    AttributionType, AttributionEntry, OpenStreetMapAttribution,
    SdkCategory, SdkDataCollection, SdkDisclosure,
    DataSafetyCategory, DataSafetyPurpose, DataSharingStatus,
    DataSafetyEntry, DataSafetyForm, PrivacyComplianceStatus,
};
use crate::engine::release::{
    self, AppIdentity, AppNameStatus, SigningKeyConfig, SigningKeyStatus,
    BuildType, ReleaseBundle, ReleaseBundleStatus,
    VersioningPolicy, VersioningStrategy,
    StoreAsset, StoreAssetType, AssetStatus, StoreAssets,
    StoreListing, ContentRating, AppCategory,
    PermissionType, PermissionDeclaration, PermissionDeclarations,
    ReviewerAccess, TestingTrack, TrackStatus, ReleaseTrack,
    StagedRolloutPlan, StackComponent, StackComponentCategory,
    ProductionStack, ReleaseReadinessStatus,
};
use crate::models::{
    ActivityType, LocationSource, SensorSource, WorkoutCheckpoint, WorkoutGoal, WorkoutPoint,
    WorkoutSummary,
};

// ─── Global registry of live controllers ───────────────────────────────────

static REGISTRY: Lazy<Mutex<HashMap<i64, WorkoutSessionController>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT_HANDLE: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(1));

fn next_handle() -> i64 {
    let mut n = NEXT_HANDLE.lock().unwrap();
    let h = *n;
    *n += 1;
    h
}

// ─── JSON envelope helpers ──────────────────────────────────────────────────

fn ok_json<T: Serialize>(data: &T) -> CString {
    let body = json!({ "ok": true, "data": data });
    CString::new(body.to_string())
        .unwrap_or_else(|_| CString::new("{\"ok\":false,\"error\":\"encoding_error\"}").unwrap())
}

fn err_json(message: impl std::fmt::Display) -> CString {
    let body = json!({ "ok": false, "error": message.to_string() });
    CString::new(body.to_string())
        .unwrap_or_else(|_| CString::new("{\"ok\":false,\"error\":\"encoding_error\"}").unwrap())
}

/// Runs `f`, catching any Rust panic, and always returns an owned CString
/// pointer (never null) so the Dart side has a uniform contract.
fn guarded(f: impl FnOnce() -> CString + panic::UnwindSafe) -> *mut c_char {
    let result = panic::catch_unwind(f);
    let cstring = result.unwrap_or_else(|payload| {
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic in stride_engine".to_string()
        };
        err_json(format!("internal_panic: {msg}"))
    });
    cstring.into_raw()
}

/// Reads a `*const c_char` argument from Dart as a `&str`. Returns an
/// error string describing the problem rather than panicking on bad input.
unsafe fn read_str<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null_string_argument".to_string());
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|e| format!("invalid_utf8: {e}"))
}

// ─── Memory management ──────────────────────────────────────────────────────

/// Frees a C string previously returned by any `stride_*` function in this
/// module. Calling this on a pointer not obtained from this module (or
/// calling it twice on the same pointer) is undefined behavior — the Dart
/// binding layer must call this exactly once per returned pointer.
#[no_mangle]
pub extern "C" fn stride_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

// ─── Lifecycle ───────────────────────────────────────────────────────────────

/// Creates a new workout session controller.
///
/// `request_json` shape:
/// ```json
/// {
///   "user_id": "u1",
///   "activity_type": "walk" | "run" | "hike" | "auto_detect",
///   "weight_kg": 70.0,
///   "started_at": 1690000000000,
///   "config": { ... SessionConfig, optional, defaults applied ... }
/// }
/// ```
/// Returns `{"ok": true, "data": {"handle": <i64>, "workout_id": "..."}}`.
#[no_mangle]
pub extern "C" fn stride_create_session(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            user_id: String,
            activity_type: ActivityType,
            weight_kg: f64,
            started_at: i64,
            #[serde(default)]
            config: Option<SessionConfig>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let config = req.config.unwrap_or_else(default_session_config);

        let controller = WorkoutSessionController::new(
            req.user_id,
            req.activity_type,
            req.weight_kg,
            req.started_at,
            config,
        );
        let workout_id = controller.workout_id().to_string();
        let handle = next_handle();
        REGISTRY.lock().unwrap().insert(handle, controller);

        ok_json(&json!({ "handle": handle, "workout_id": workout_id }))
    })
}

fn default_session_config() -> SessionConfig {
    serde_json::from_str("{}").unwrap_or_else(|e| panic!("SessionConfig default failed: {e}"))
}

/// Destroys a session and frees its memory. Must be called once the
/// workout has been finished/discarded and the Dart side no longer needs
/// it, to avoid leaking controllers in the registry for the app's
/// lifetime.
#[no_mangle]
pub extern "C" fn stride_destroy_session(handle: i64) -> *mut c_char {
    guarded(move || {
        let removed = REGISTRY.lock().unwrap().remove(&handle).is_some();
        ok_json(&json!({ "removed": removed }))
    })
}

/// Runs `f` with mutable access to the controller for `handle`, returning
/// its JSON result, or an `{"ok": false, ...}` envelope if the handle is
/// unknown.
fn with_controller(
    handle: i64,
    f: impl FnOnce(&mut WorkoutSessionController) -> CString,
) -> CString {
    let mut registry = REGISTRY.lock().unwrap();
    match registry.get_mut(&handle) {
        Some(controller) => f(controller),
        None => err_json(format!("unknown_handle: {handle}")),
    }
}

// ─── State transitions ───────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn stride_start(handle: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| match c.start() {
            Ok(event) => ok_json(&event),
            Err(e) => err_json(e),
        })
    })
}

#[no_mangle]
pub extern "C" fn stride_pause(handle: i64, now_ms: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| match c.pause(now_ms) {
            Ok(()) => ok_json(&json!({})),
            Err(e) => err_json(e),
        })
    })
}

#[no_mangle]
pub extern "C" fn stride_resume(handle: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| match c.resume() {
            Ok(()) => ok_json(&json!({})),
            Err(e) => err_json(e),
        })
    })
}

#[no_mangle]
pub extern "C" fn stride_discard(handle: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| match c.discard() {
            Ok(()) => ok_json(&json!({})),
            Err(e) => err_json(e),
        })
    })
}

// ─── Goal ────────────────────────────────────────────────────────────────────

/// `goal_json` is a serialized `WorkoutGoal`, e.g.
/// `{"goal_type": "distance", "target_value": 5000.0}`.
#[no_mangle]
pub extern "C" fn stride_set_goal(handle: i64, goal_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(goal_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };
        let goal: WorkoutGoal = match serde_json::from_str(raw) {
            Ok(g) => g,
            Err(e) => return err_json(format!("invalid_goal: {e}")),
        };
        with_controller(handle, |c| {
            c.set_goal(goal);
            ok_json(&json!({}))
        })
    })
}

/// Supplies the user's prior personal bests (loaded by Dart from local or
/// cloud workout history right after `stride_create_session`) so the
/// controller can emit a live "personal record possible" coaching nudge.
/// Optional — if never called, no live nudge fires, but
/// `stride_detect_personal_records` post-finish still works normally.
///
/// `prior_bests_json` is a serialized `PriorBests`, e.g.
/// `{"longest_distance_meters": 8046.7, "highest_average_pace_sec_per_km": 300.0}`
/// (all fields optional).
#[no_mangle]
pub extern "C" fn stride_set_prior_bests(
    handle: i64,
    prior_bests_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(prior_bests_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };
        let prior: PriorBests = match serde_json::from_str(raw) {
            Ok(p) => p,
            Err(e) => return err_json(format!("invalid_prior_bests: {e}")),
        };
        with_controller(handle, |c| {
            c.set_prior_bests(prior);
            ok_json(&json!({}))
        })
    })
}

// ─── Live sensor ingestion ───────────────────────────────────────────────────

/// `point_json` is a serialized `WorkoutPoint` (see models/point.rs). The
/// engine fills in `workout_id`/`accepted`/`rejection_reason` itself, so
/// Dart only needs to supply lat/lon/altitude/accuracy/speed/bearing/
/// recorded_at/source/is_mock_location (point_id can be any placeholder).
/// Returns the full `LiveUpdate` snapshot as JSON.
#[no_mangle]
pub extern "C" fn stride_add_location_sample(
    handle: i64,
    point_json: *const c_char,
    now_ms: i64,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(point_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };
        let point: WorkoutPoint = match serde_json::from_str(raw) {
            Ok(p) => p,
            Err(e) => return err_json(format!("invalid_point: {e}")),
        };
        with_controller(handle, |c| {
            let update = c.add_location_sample(point, now_ms);
            ok_json(&update)
        })
    })
}

#[no_mangle]
pub extern "C" fn stride_add_heart_rate_sample(handle: i64, at_ms: i64, bpm: u16) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| {
            c.add_heart_rate_sample(at_ms, bpm);
            ok_json(&json!({}))
        })
    })
}

/// `source` is the snake_case `SensorSource` variant name, e.g.
/// `"phone_step_sensor"`, `"wear_os"`, `"health_connect"`.
#[no_mangle]
pub extern "C" fn stride_add_step_delta(
    handle: i64,
    at_ms: i64,
    delta: u32,
    source: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(source) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };
        let source: SensorSource = match serde_json::from_value(json!(raw)) {
            Ok(s) => s,
            Err(e) => return err_json(format!("invalid_source: {e}")),
        };
        with_controller(handle, |c| {
            c.add_step_delta(at_ms, delta, source);
            ok_json(&json!({}))
        })
    })
}

/// Periodic no-new-GPS-point tick (call roughly once per second while a
/// workout is active/paused) so elapsed/paused time and GPS-staleness
/// detection keep advancing even between fixes.
#[no_mangle]
pub extern "C" fn stride_tick(handle: i64, now_ms: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| {
            let update = c.tick(now_ms);
            ok_json(&update)
        })
    })
}

#[no_mangle]
pub extern "C" fn stride_manual_lap(handle: i64, now_ms: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| {
            let split = c.manual_lap(now_ms);
            ok_json(&split)
        })
    })
}

#[no_mangle]
pub extern "C" fn stride_build_checkpoint(handle: i64, now_ms: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| {
            let checkpoint = c.build_checkpoint(now_ms);
            ok_json(&checkpoint)
        })
    })
}

/// Finalizes the workout into an immutable `WorkoutSummary`. The
/// controller remains in the registry afterward (state becomes
/// `Completed`) — Dart should call `stride_destroy_session` once it has
/// persisted the summary locally.
#[no_mangle]
pub extern "C" fn stride_finish(handle: i64, now_ms: i64) -> *mut c_char {
    guarded(move || {
        with_controller(handle, |c| match c.finish(now_ms) {
            Ok(summary) => ok_json(&summary),
            Err(e) => err_json(e),
        })
    })
}

/// Returns the current live `WorkoutSession` snapshot without advancing
/// anything — useful for reattaching the UI after a hot-restart or
/// tab-away/tab-back.
#[no_mangle]
pub extern "C" fn stride_get_session(handle: i64) -> *mut c_char {
    guarded(move || with_controller(handle, |c| ok_json(c.session())))
}

/// Returns the crate version, useful for the Dart side to confirm the
/// native library loaded successfully and log which build is running.
#[no_mangle]
pub extern "C" fn stride_engine_version() -> *mut c_char {
    guarded(|| ok_json(&env!("CARGO_PKG_VERSION")))
}

// ─── Crash / restart recovery ───────────────────────────────────────────

/// Given a checkpoint previously returned by `stride_build_checkpoint` (and
/// persisted by Dart to SQLite), decides what recovery UI to present:
/// resume, finish-and-save, or discard. Pure decision logic — does not
/// create or touch any controller. Call this at app startup before
/// deciding whether to call `stride_restore_session`.
///
/// `checkpoint_json` is a serialized `WorkoutCheckpoint`. Returns a
/// `RecoveryDecision` JSON: `{"offered_options": [...], "recommended_option":
/// "...", "reason": "..."}`.
#[no_mangle]
pub extern "C" fn stride_evaluate_recovery(
    checkpoint_json: *const c_char,
    now_ms: i64,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(checkpoint_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };
        let checkpoint: WorkoutCheckpoint = match serde_json::from_str(raw) {
            Ok(c) => c,
            Err(e) => return err_json(format!("invalid_checkpoint: {e}")),
        };
        let decision = crate::engine::recovery::evaluate_checkpoint(&checkpoint, now_ms);
        ok_json(&decision)
    })
}

/// Rebuilds a live `WorkoutSessionController` from a persisted checkpoint
/// after the app was killed/crashed/restarted mid-workout, registers it
/// under a new handle, and returns that handle so Dart can resume issuing
/// normal lifecycle/sample calls against it.
///
/// The restored controller always lands in the `Paused` state — the user
/// must explicitly resume to continue live GPS tracking.
///
/// `request_json` shape:
/// ```json
/// {
///   "checkpoint": { ... WorkoutCheckpoint ... },
///   "user_id": "u1",
///   "activity_type": "walk",
///   "weight_kg": 70.0,
///   "config": { ... SessionConfig, optional ... }
/// }
/// ```
/// Returns `{"ok": true, "data": {"handle": <i64>, "workout_id": "..."}}`.
#[no_mangle]
pub extern "C" fn stride_restore_session(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            checkpoint: WorkoutCheckpoint,
            user_id: String,
            activity_type: ActivityType,
            weight_kg: f64,
            #[serde(default)]
            config: Option<SessionConfig>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let config = req.config.unwrap_or_else(default_session_config);
        let controller = WorkoutSessionController::restore_from_checkpoint(
            &req.checkpoint,
            req.user_id,
            req.activity_type,
            req.weight_kg,
            config,
        );
        let workout_id = controller.workout_id().to_string();
        let handle = next_handle();
        REGISTRY.lock().unwrap().insert(handle, controller);

        ok_json(&json!({ "handle": handle, "workout_id": workout_id }))
    })
}

// ─── Stateless decision-logic entry points ─────────────────────────────
//
// The functions below wrap pure logic modules that do not need a live
// session handle at all: validation, personal records, permissions, and
// battery/sampling decisions. They exist so Dart can invoke this logic
// directly (e.g. permission/battery decisions happen before a workout
// session is even created) rather than only through the controller.

/// Re-runs full workout validation (spec section 26) given a
/// `ValidationInput` JSON payload. `stride_finish` already runs this
/// internally against the data the engine tracked live, but it cannot know
/// about duplicate-workout detection (which requires querying local
/// storage) — Dart should call this again after `stride_finish` with
/// `is_duplicate_of_existing_workout` filled in from its own history
/// lookup, and OR the two `is_blocked` results together.
#[no_mangle]
pub extern "C" fn stride_validate(input_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(input_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };
        let input: ValidationInput = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => return err_json(format!("invalid_validation_input: {e}")),
        };
        ok_json(&validation::validate(&input))
    })
}

/// Detects personal records (spec section 28) for a finalized workout
/// summary against the caller-supplied prior bests. Dart is responsible
/// for loading `PriorBests` from local/cloud workout history before
/// calling this — the Rust engine has no storage access of its own.
///
/// `request_json` shape:
/// ```json
/// { "summary": { ... WorkoutSummary, e.g. from stride_finish ... },
///   "prior_bests": { "fastest_mile_sec_per_km": 300.0, ... (all optional) } }
/// ```
#[no_mangle]
pub extern "C" fn stride_detect_personal_records(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            summary: WorkoutSummary,
            #[serde(default)]
            prior_bests: PriorBests,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let records = personal_records::detect_records(&req.summary, &req.prior_bests);
        ok_json(&records)
    })
}

/// Permissions decision logic (spec section 30): given the current
/// `PermissionState` and `PermissionContext`, returns the `RequiredAction`
/// the app should take. Pure/stateless — safe to call at any time,
/// including before any workout session exists.
///
/// `request_json` shape: `{"state": "granted", "context": "initial"}`.
#[no_mangle]
pub extern "C" fn stride_required_permission_action(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            state: PermissionState,
            context: PermissionContext,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let action = permissions::required_action(req.state, req.context);
        ok_json(&action)
    })
}

/// Decides whether a workout can start in full-featured, foreground-only,
/// or blocked mode, given precise/background location permission states.
///
/// `request_json` shape:
/// `{"precise_location": "granted", "background_location": "denied"}`.
#[no_mangle]
pub extern "C" fn stride_evaluate_start_capability(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            precise_location: PermissionState,
            background_location: PermissionState,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let cap =
            permissions::evaluate_start_capability(req.precise_location, req.background_location);
        ok_json(&cap)
    })
}

/// Battery/sampling manager (spec section 32): chooses a `SamplingProfile`
/// (GPS interval, min-distance filter, accuracy mode, sensor interval,
/// DB batch-flush interval) given the current workout activity level and
/// battery context. Pure/stateless — Dart should call this whenever
/// activity level or battery state changes and reconfigure its GPS/sensor
/// listeners accordingly.
///
/// `request_json` shape:
/// ```json
/// { "activity": "active",
///   "battery": {"battery_percent": 42, "battery_saver_enabled": false, "is_charging": false} }
/// ```
#[no_mangle]
pub extern "C" fn stride_choose_sampling_profile(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            activity: WorkoutActivityLevel,
            battery: BatteryContext,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let profile = battery::choose_sampling_profile(req.activity, req.battery);
        ok_json(&profile)
    })
}

/// Unit system (spec section 34): applies a single named conversion to a
/// value. Dart may keep doing its own simple `*3.6`-style conversions if
/// it prefers, but this exists so the canonical formulas defined once in
/// Rust are also reachable from Dart without duplicating constants.
///
/// `request_json` shape: `{"kind": "meters_to_miles", "value": 5000.0}`.
#[no_mangle]
pub extern "C" fn stride_convert_unit(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            kind: ConversionKind,
            value: f64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = units::convert(req.kind, req.value);
        ok_json(&json!({ "value": result }))
    })
}

/// Formats a canonical pace (seconds per km) as "M:SS" for the requested
/// display unit, e.g. `{"sec_per_km": 330.0, "unit": "kilometers"}` ->
/// `{"formatted": "5:30"}`.
#[no_mangle]
pub extern "C" fn stride_format_pace(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            sec_per_km: f64,
            unit: DistanceUnit,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let formatted = units::format_pace(req.sec_per_km, req.unit);
        ok_json(&json!({ "formatted": formatted }))
    })
}

/// Achievement/badge engine (spec section 29): decides which one-time
/// badges a completed, validated workout newly unlocks. Dart owns loading
/// `AchievementHistory` from storage beforehand and persisting any
/// newly-returned awards afterward so they are never re-issued — this
/// module is stateless between calls.
///
/// `request_json` shape:
/// ```json
/// { "summary": { ... WorkoutSummary ... },
///   "history": { ... AchievementHistory ... },
///   "weekly_goal_completed_by_this_workout": false,
///   "new_pace_record_set": false }
/// ```
#[no_mangle]
pub extern "C" fn stride_detect_achievements(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            summary: WorkoutSummary,
            #[serde(default)]
            history: AchievementHistory,
            #[serde(default)]
            weekly_goal_completed_by_this_workout: bool,
            #[serde(default)]
            new_pace_record_set: bool,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let events = achievements::detect_achievements(
            &req.summary,
            &req.history,
            req.weekly_goal_completed_by_this_workout,
            req.new_pace_record_set,
        );
        ok_json(&events)
    })
}

// ──────────────────────────────────────────────────────────────────────
// Cloud synchronization engine (spec section 3)
// ──────────────────────────────────────────────────────────────────────

/// Computes the retry delay (in milliseconds) for a sync attempt using
/// exponential backoff with jitter.
///
/// `request_json` shape: `{"attempt": 2, "jitter_seed": 12345}`.
/// Returns `{"delay_ms": 4000}`.
#[no_mangle]
pub extern "C" fn stride_compute_sync_backoff(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            attempt: u32,
            #[serde(default)]
            jitter_seed: u64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let delay_ms = sync::compute_backoff_delay(req.attempt, req.jitter_seed);
        ok_json(&json!({ "delay_ms": delay_ms }))
    })
}

/// Decides whether a failed sync attempt should be retried, and if so,
/// after how long.
///
/// `request_json` shape:
/// `{"result": "retryable_failure", "current_retry_count": 2, "jitter_seed": 42}`.
/// Returns `{"retry": true, "delay_ms": 4000}` or `{"retry": false}`.
#[no_mangle]
pub extern "C" fn stride_decide_sync_retry(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            result: sync::SyncAttemptResult,
            current_retry_count: u32,
            #[serde(default)]
            jitter_seed: u64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match sync::decide_retry(req.result, req.current_retry_count, req.jitter_seed) {
            Some(delay_ms) => ok_json(&json!({ "retry": true, "delay_ms": delay_ms })),
            None => ok_json(&json!({ "retry": false })),
        }
    })
}

/// Resolves a sync conflict between local and cloud versions.
///
/// `request_json` shape:
/// ```json
/// { "workout_id": "w1",
///   "local_updated_at": 2000,
///   "cloud_updated_at": 1000,
///   "cloud_device_id": "device-b",
///   "local_device_id": "device-a",
///   "strategy": "last_write_wins" }
/// ```
/// Returns `{"decision": "keep_local"}`.
#[no_mangle]
pub extern "C" fn stride_resolve_sync_conflict(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            #[serde(flatten)]
            info: ConflictInfo,
            #[serde(default = "default_conflict_strategy")]
            strategy: ConflictResolutionStrategy,
        }

        fn default_conflict_strategy() -> ConflictResolutionStrategy {
            ConflictResolutionStrategy::LastWriteWins
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision = sync::resolve_conflict(&req.info, req.strategy);
        ok_json(&json!({ "decision": decision }))
    })
}

/// Checks whether a local workout is a duplicate of an existing cloud
/// workout.
///
/// `request_json` shape:
/// ```json
/// { "local_workout_id": "w1",
///   "local_updated_at": 1000,
///   "cloud_workout_id": "w1",
///   "cloud_updated_at": 1000 }
/// ```
/// Returns `{"is_duplicate": true}`.
#[no_mangle]
pub extern "C" fn stride_detect_sync_duplicate(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            local_workout_id: String,
            local_updated_at: i64,
            cloud_workout_id: Option<String>,
            cloud_updated_at: Option<i64>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let is_dup = sync::is_duplicate(
            &req.local_workout_id,
            req.local_updated_at,
            req.cloud_workout_id.as_deref(),
            req.cloud_updated_at,
        );
        ok_json(&json!({ "is_duplicate": is_dup }))
    })
}

/// Decides whether to insert, update, or skip a workout upsert.
///
/// `request_json` shape (same as detect_sync_duplicate).
/// Returns `{"decision": "insert"}`.
#[no_mangle]
pub extern "C" fn stride_decide_sync_upsert(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            local_workout_id: String,
            local_updated_at: i64,
            cloud_workout_id: Option<String>,
            cloud_updated_at: Option<i64>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision = sync::decide_upsert(
            &req.local_workout_id,
            req.local_updated_at,
            req.cloud_workout_id.as_deref(),
            req.cloud_updated_at,
        );
        ok_json(&json!({ "decision": decision }))
    })
}

/// Decides what action the current device should take for a workout in
/// device-to-device sync.
///
/// `request_json` shape:
/// ```json
/// { "recording_device_id": "device-a",
///   "current_device_id": "device-a",
///   "is_uploaded": false,
///   "is_downloaded": false }
/// ```
/// Returns `{"action": "upload"}`.
#[no_mangle]
pub extern "C" fn stride_decide_device_sync(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            #[serde(flatten)]
            state: DeviceSyncState,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let action = sync::decide_device_sync_action(&req.state);
        ok_json(&json!({ "action": action }))
    })
}

/// Decides whether a deletion tombstone should be retried.
///
/// `request_json` shape: `{"state": "failed", "retry_count": 3}`.
/// Returns `{"should_retry": true}`.
#[no_mangle]
pub extern "C" fn stride_tombstone_should_retry(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            state: sync::TombstoneState,
            retry_count: u32,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let should = sync::tombstone_should_retry(req.state, req.retry_count);
        ok_json(&json!({ "should_retry": should }))
    })
}

/// Decides whether a synced tombstone is old enough to be garbage-collected.
///
/// `request_json` shape: `{"state": "synced", "synced_at": 1000, "now_ms": 999999}`.
/// Returns `{"should_gc": true}`.
#[no_mangle]
pub extern "C" fn stride_tombstone_should_gc(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            state: sync::TombstoneState,
            synced_at: i64,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let should = sync::tombstone_should_gc(req.state, req.synced_at, req.now_ms);
        ok_json(&json!({ "should_gc": should }))
    })
}

// ════════════════════════════════════════════════════════════════════
// Route file format & storage layout (spec section 4)
// ════════════════════════════════════════════════════════════════════

/// Decides which route file format to use for a workout.
///
/// `request_json` shape:
/// `{"point_count": 1000, "is_offline": false, "wants_gpx_export": false}`.
/// Returns `{"format": "gpx" | "compressed_binary" | "polyline_only"}`.
#[no_mangle]
pub extern "C" fn stride_decide_route_format(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let req: RouteFormatInput = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let format = route_format::decide_route_format(&req);
        ok_json(&json!({ "format": format }))
    })
}

/// Generates the Cloud Storage object key for a route file.
///
/// `request_json` shape:
/// `{"user_id": "u1", "workout_id": "wk1", "format": "gpx"}`.
/// Returns `{"path": "routes/u1/wk1.gpx"}` or `{"path": null}` for
/// polyline-only routes (no file is uploaded).
#[no_mangle]
pub extern "C" fn stride_generate_route_file_path(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            user_id: String,
            workout_id: String,
            format: RouteFileFormat,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let path = route_format::generate_route_file_path(&req.user_id, &req.workout_id, req.format);
        ok_json(&json!({ "path": path }))
    })
}

/// Estimates the byte size of a route file before it is serialized.
///
/// `request_json` shape: `{"format": "gpx", "point_count": 1000}`.
/// Returns `{"size_bytes": 180512}`.
#[no_mangle]
pub extern "C" fn stride_estimate_route_file_size(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            format: RouteFileFormat,
            point_count: usize,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let size = route_format::estimate_route_file_size(req.format, req.point_count);
        ok_json(&json!({ "size_bytes": size }))
    })
}

/// Decides whether a route file upload should wait for Wi-Fi.
///
/// `request_json` shape:
/// `{"format": "gpx", "point_count": 4000, "is_on_wifi": false}`.
/// Returns `{"should_prefer_wifi": true}`.
#[no_mangle]
pub extern "C" fn stride_should_prefer_wifi_for_upload(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            format: RouteFileFormat,
            point_count: usize,
            is_on_wifi: bool,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let prefer = route_format::should_prefer_wifi_for_upload(
            req.format,
            req.point_count,
            req.is_on_wifi,
        );
        ok_json(&json!({ "should_prefer_wifi": prefer }))
    })
}

/// Serializes a slice of WorkoutPoints into a GPX 1.1 XML document.
///
/// `request_json` shape:
/// `{"workout_id": "wk1", "started_at_ms": 1000, "points": [{...}, ...]}`.
/// Returns `{"gpx": "<gpx ...>...</gpx>"}`.
///
/// Only accepted points are emitted. This is the full-resolution route
/// export for Cloud Storage upload or Strava/Garmin sharing.
#[no_mangle]
pub extern "C" fn stride_serialize_gpx(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            workout_id: String,
            started_at_ms: i64,
            points: Vec<WorkoutPoint>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let gpx = route_format::serialize_gpx(&req.workout_id, req.started_at_ms, &req.points);
        ok_json(&json!({ "gpx": gpx }))
    })
}

/// Builds route file metadata from controller inputs at workout-finish time.
///
/// `request_json` shape:
/// `{"user_id": "u1", "workout_id": "wk1", "points": [...],
///  "device_source": "pixel-8", "is_offline": false,
///  "wants_gpx_export": false, "finished_at": 1900000}`.
/// Returns the full `RouteFileMetadata` JSON object.
#[no_mangle]
pub extern "C" fn stride_build_route_file_metadata(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            user_id: String,
            workout_id: String,
            points: Vec<WorkoutPoint>,
            device_source: String,
            is_offline: bool,
            wants_gpx_export: bool,
            finished_at: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let meta = route_format::build_route_file_metadata(
            &req.user_id,
            &req.workout_id,
            &req.points,
            &req.device_source,
            req.is_offline,
            req.wants_gpx_export,
            req.finished_at,
        );
        ok_json(&meta)
    })
}

/// Checks whether a route file sync state is terminal (no further
/// automatic action will be taken).
///
/// `request_json` shape: `{"state": "uploaded"}`.
/// Returns `{"is_terminal": true}`.
#[no_mangle]
pub extern "C" fn stride_is_route_sync_terminal(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            state: RouteFileSyncState,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let is_terminal = route_format::is_route_sync_terminal(req.state);
        ok_json(&json!({ "is_terminal": is_terminal }))
    })
}

/// Validates that a RouteSummary is safe to write to Firestore.
///
/// `request_json` shape: the full `RouteSummary` JSON object.
/// Returns `{"valid": true}` or `{"valid": false, "error": "..."}`.
///
/// The Dart layer calls this *before* writing to Firestore so a bug
/// never produces an invalid or oversized document (e.g. a route file
/// path that doesn't match the user_id prefix, or a polyline that
/// exceeds the 1 MiB document limit).
#[no_mangle]
pub extern "C" fn stride_validate_route_summary(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let summary: RouteSummary = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match route_format::validate_route_summary(&summary) {
            Ok(()) => ok_json(&json!({ "valid": true })),
            Err(msg) => ok_json(&json!({ "valid": false, "error": msg })),
        }
    })
}

/// Determines the dominant LocationSource of a route (the source that
/// contributed the most accepted points).
///
/// `request_json` shape: `{"points": [{...}, ...]}`.
/// Returns `{"source": "phone_gps"}` or `{"source": null}` if no
/// accepted points exist.
#[no_mangle]
pub extern "C" fn stride_dominant_location_source(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            points: Vec<WorkoutPoint>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let source = route_format::dominant_location_source(&req.points);
        ok_json(&json!({ "source": source }))
    })
}

/// Returns the human-readable label for a LocationSource.
///
/// `request_json` shape: `{"source": "phone_gps"}`.
/// Returns `{"label": "Phone GPS"}`.
#[no_mangle]
pub extern "C" fn stride_location_source_label(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            source: LocationSource,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let label = route_format::location_source_label(req.source);
        ok_json(&json!({ "label": label }))
    })
}

// ────────────────────────────────────────────────────────────────────────────
// §5 Maps and location services
// ────────────────────────────────────────────────────────────────────────────

/// Determines the tile provider for a given map view type.
///
/// `request_json` shape: `{"view": "standard"}`.
/// Returns `{"provider": "open_street_map"}`.
#[no_mangle]
pub extern "C" fn stride_provider_for_view(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            view: MapViewType,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let provider = maps::provider_for_view(req.view);
        ok_json(&json!({ "provider": provider }))
    })
}

/// Returns the attribution text for a tile provider.
///
/// `request_json` shape: `{"provider": "open_street_map"}`.
/// Returns `{"attribution": "© OpenStreetMap contributors"}`.
#[no_mangle]
pub extern "C" fn stride_attribution_for_provider(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            provider: TileProvider,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let attribution = maps::attribution_for_provider(req.provider);
        ok_json(&json!({ "attribution": attribution }))
    })
}

/// Returns the attribution text for a map view type (convenience
/// wrapper that resolves the view → provider → attribution chain).
///
/// `request_json` shape: `{"view": "standard"}`.
/// Returns `{"attribution": "© OpenStreetMap contributors"}`.
#[no_mangle]
pub extern "C" fn stride_attribution_for_view(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            view: MapViewType,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let attribution = maps::attribution_for_view(req.view);
        ok_json(&json!({ "attribution": attribution }))
    })
}

/// Returns the tile URL template for a provider.
///
/// `request_json` shape: `{"provider": "open_street_map"}`.
/// Returns `{"url_template": "https://tile.openstreetmap.org/{z}/{x}/{y}.png"}`.
#[no_mangle]
pub extern "C" fn stride_tile_url_template(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            provider: TileProvider,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let template = maps::tile_url_template(req.provider);
        ok_json(&json!({ "url_template": template }))
    })
}

/// Classifies a GPS accuracy value (in meters) into a quality level.
///
/// `request_json` shape: `{"accuracy_meters": 4.5}` or `{"accuracy_meters": null}`.
/// Returns `{"level": "excellent"}`.
#[no_mangle]
pub extern "C" fn stride_classify_gps_accuracy(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            accuracy_meters: Option<f64>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let level = maps::classify_gps_accuracy(req.accuracy_meters);
        ok_json(&json!({ "level": level }))
    })
}

/// Returns the human-readable description for a GPS accuracy level.
///
/// `request_json` shape: `{"level": "good"}`.
/// Returns `{"description": "Good GPS (5–10 m)"}`.
#[no_mangle]
pub extern "C" fn stride_gps_accuracy_description(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            level: GpsAccuracyLevel,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let description = maps::gps_accuracy_description(req.level);
        ok_json(&json!({ "description": description }))
    })
}

/// Returns the hex color for the GPS accuracy indicator dot.
///
/// `request_json` shape: `{"level": "excellent"}`.
/// Returns `{"color": "#4CAF50"}`.
#[no_mangle]
pub extern "C" fn stride_gps_accuracy_color(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            level: GpsAccuracyLevel,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let color = maps::gps_accuracy_color(req.level);
        ok_json(&json!({ "color": color }))
    })
}

/// Converts a (lat, lon) pair to a tile coordinate at a given zoom.
///
/// `request_json` shape: `{"lat": 51.5, "lon": -0.1, "zoom": 15}`.
/// Returns `{"tile": {"z": 15, "x": 16384, "y": 10904}}` or
/// `{"tile": null}` if out of range.
#[no_mangle]
pub extern "C" fn stride_lat_lon_to_tile(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            lat: f64,
            lon: f64,
            zoom: u8,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let tile = maps::lat_lon_to_tile(req.lat, req.lon, req.zoom);
        ok_json(&json!({ "tile": tile }))
    })
}

/// Counts the total number of tiles needed to cover a bounding box
/// across a range of zoom levels. Used to estimate offline download
/// size before starting a download.
///
/// `request_json` shape: `{"min_lat": 51.4, "min_lon": -0.2,
/// "max_lat": 51.6, "max_lon": 0.0, "min_zoom": 10, "max_zoom": 16}`.
/// Returns `{"tile_count": 1234}`.
#[no_mangle]
pub extern "C" fn stride_count_tiles_in_region(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            min_lat: f64,
            min_lon: f64,
            max_lat: f64,
            max_lon: f64,
            min_zoom: u8,
            max_zoom: u8,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let count = maps::count_tiles_in_region(
            req.min_lat, req.min_lon, req.max_lat, req.max_lon,
            req.min_zoom, req.max_zoom,
        );
        ok_json(&json!({ "tile_count": count }))
    })
}

/// Builds an offline region manifest from the user's download request.
/// Computes tile count and estimated size.
///
/// `request_json` shape: `{"region_id": "uuid", "name": "Home",
/// "min_lat": ..., "min_lon": ..., "max_lat": ..., "max_lon": ...,
/// "min_zoom": 10, "max_zoom": 16, "provider": "open_street_map",
/// "now_ms": 1700000000000}`.
/// Returns the full `OfflineRegion` as JSON.
#[no_mangle]
pub extern "C" fn stride_build_offline_region(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            region_id: String,
            name: String,
            min_lat: f64,
            min_lon: f64,
            max_lat: f64,
            max_lon: f64,
            min_zoom: u8,
            max_zoom: u8,
            provider: TileProvider,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let region = maps::build_offline_region(
            &req.region_id, &req.name,
            req.min_lat, req.min_lon, req.max_lat, req.max_lon,
            req.min_zoom, req.max_zoom, req.provider, req.now_ms,
        );
        ok_json(&region)
    })
}

/// Checks whether the device has enough free storage to download a
/// new offline region.
///
/// `request_json` shape: `{"new_region_size_bytes": 50000000,
/// "current_total_cache_bytes": 200000000, "device_free_bytes": 1000000000}`.
/// Returns `{"ok": true}` or `{"ok": false, "error": "Not enough..."}`.
#[no_mangle]
pub extern "C" fn stride_check_storage_availability(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            new_region_size_bytes: u64,
            current_total_cache_bytes: u64,
            device_free_bytes: u64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match maps::check_storage_availability(
            req.new_region_size_bytes,
            req.current_total_cache_bytes,
            req.device_free_bytes,
        ) {
            Ok(()) => ok_json(&json!({ "available": true })),
            Err(msg) => ok_json(&json!({ "available": false, "error": msg })),
        }
    })
}

/// Checks whether the user can save a new offline region (count limit).
///
/// `request_json` shape: `{"current_region_count": 15}`.
/// Returns `{"can_add": true}`.
#[no_mangle]
pub extern "C" fn stride_can_add_region(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_region_count: usize,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let can_add = maps::can_add_region(req.current_region_count);
        ok_json(&json!({ "can_add": can_add }))
    })
}

/// Selects the LRU eviction candidate from existing offline regions.
///
/// `request_json` shape: `{"regions": [...], "bytes_needed": 50000000}`.
/// Returns `{"evict_index": 2}` or `{"evict_index": null}`.
#[no_mangle]
pub extern "C" fn stride_select_eviction_candidate(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            regions: Vec<OfflineRegion>,
            bytes_needed: u64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let index = maps::select_eviction_candidate(&req.regions, req.bytes_needed);
        ok_json(&json!({ "evict_index": index }))
    })
}

/// Checks whether a downloaded region is stale (not accessed in 90 days).
///
/// `request_json` shape: `{"region": {...}, "now_ms": 1700000000000}`.
/// Returns `{"is_stale": true}`.
#[no_mangle]
pub extern "C" fn stride_is_region_stale(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            region: OfflineRegion,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let stale = maps::is_region_stale(&req.region, req.now_ms);
        ok_json(&json!({ "is_stale": stale }))
    })
}

/// Updates a region's last_accessed_at timestamp (for LRU tracking).
///
/// `request_json` shape: `{"region": {...}, "now_ms": 1700000000000}`.
/// Returns the updated `OfflineRegion` as JSON.
#[no_mangle]
pub extern "C" fn stride_touch_region(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            region: OfflineRegion,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let touched = maps::touch_region(req.region, req.now_ms);
        ok_json(&touched)
    })
}

/// Selects the Douglas-Peucker simplification epsilon for a zoom level.
///
/// `request_json` shape: `{"zoom": 15}`.
/// Returns `{"epsilon": 0.00003}`.
#[no_mangle]
pub extern "C" fn stride_simplification_epsilon_for_zoom(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            zoom: u8,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let epsilon = maps::simplification_epsilon_for_zoom(req.zoom);
        ok_json(&json!({ "epsilon": epsilon }))
    })
}

/// Whether the current zoom is high enough to show full-resolution route.
///
/// `request_json` shape: `{"zoom": 15}`.
/// Returns `{"show_full_resolution": true}`.
#[no_mangle]
pub extern "C" fn stride_should_show_full_resolution_route(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            zoom: u8,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let show = maps::should_show_full_resolution_route(req.zoom);
        ok_json(&json!({ "show_full_resolution": show }))
    })
}

/// Whether the recenter button should be visible.
///
/// `request_json` shape: `{"distance_from_center_meters": 75.0}`.
/// Returns `{"show_recenter": true}`.
#[no_mangle]
pub extern "C" fn stride_should_show_recenter_button(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            distance_from_center_meters: f64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let show = maps::should_show_recenter_button(req.distance_from_center_meters);
        ok_json(&json!({ "show_recenter": show }))
    })
}

/// Whether the map should rotate with the user's heading (compass mode).
///
/// `request_json` shape: `{"speed_mps": 2.0, "is_heading_valid": true}`.
/// Returns `{"rotate_with_heading": true}`.
#[no_mangle]
pub extern "C" fn stride_should_rotate_with_heading(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            speed_mps: f64,
            is_heading_valid: bool,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let rotate = maps::should_rotate_with_heading(req.speed_mps, req.is_heading_valid);
        ok_json(&json!({ "rotate_with_heading": rotate }))
    })
}

/// Validates a saved route before persisting it.
///
/// `request_json` shape: the full `SavedRoute` JSON.
/// Returns `{"valid": true}` or `{"valid": false, "error": "..."}`.
#[no_mangle]
pub extern "C" fn stride_validate_saved_route(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let route: SavedRoute = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match maps::validate_saved_route(&route) {
            Ok(()) => ok_json(&json!({ "valid": true })),
            Err(msg) => ok_json(&json!({ "valid": false, "error": msg })),
        }
    })
}

/// Builds the cache key for a tile in the on-device tile cache.
///
/// `request_json` shape: `{"provider": "open_street_map",
/// "tile": {"z": 15, "x": 16384, "y": 10904}}`.
/// Returns `{"cache_key": "tiles/osm/15/16384/10904"}`.
#[no_mangle]
pub extern "C" fn stride_tile_cache_key(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            provider: TileProvider,
            tile: TileCoord,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let key = maps::tile_cache_key(req.provider, req.tile);
        ok_json(&json!({ "cache_key": key }))
    })
}

/// Returns the short filesystem slug for a tile provider.
///
/// `request_json` shape: `{"provider": "open_street_map"}`.
/// Returns `{"slug": "osm"}`.
#[no_mangle]
pub extern "C" fn stride_tile_provider_slug(request_json: *const c_char) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            provider: TileProvider,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let slug = maps::tile_provider_slug(req.provider);
        ok_json(&json!({ "slug": slug }))
    })
}

// ===========================================================================
// §6 — Authentication / account lifecycle
// ===========================================================================

pub extern "C" fn stride_auth_provider_label(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            provider: AuthProvider,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let label = account::auth_provider_label(req.provider);
        ok_json(&json!({ "label": label }))
    })
}

pub extern "C" fn stride_classify_session_state(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            token_issued_at_ms: i64,
            token_expires_at_ms: i64,
            now_ms: i64,
            is_revoked: bool,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let state =
            account::classify_session_state(
                req.token_issued_at_ms,
                req.token_expires_at_ms,
                req.now_ms,
                req.is_revoked,
            );
        ok_json(&json!({ "session_state": state }))
    })
}

pub extern "C" fn stride_needs_token_refresh(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            session_state: SessionState,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let needs = account::needs_token_refresh(req.session_state);
        ok_json(&json!({ "needs_refresh": needs }))
    })
}

pub extern "C" fn stride_requires_relogin(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            session_state: SessionState,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let needs = account::requires_relogin(req.session_state);
        ok_json(&json!({ "needs_relogin": needs }))
    })
}

pub extern "C" fn stride_requires_reauthentication(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            action: SensitiveAction,
            last_auth_at_ms: i64,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let requires = account::requires_reauthentication(
            req.action,
            req.last_auth_at_ms,
            req.now_ms,
        );
        ok_json(&json!({ "requires_reauth": requires }))
    })
}

pub extern "C" fn stride_reauth_threshold_ms(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            action: SensitiveAction,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let threshold = account::reauth_threshold_ms(req.action);
        ok_json(&json!({ "threshold_ms": threshold }))
    })
}

pub extern "C" fn stride_reauth_reason(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            action: SensitiveAction,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let reason = account::reauth_reason(req.action);
        ok_json(&json!({ "reason": reason }))
    })
}

pub extern "C" fn stride_decide_verification_action(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            provider: AuthProvider,
            is_verified: bool,
            verification_sent_at_ms: i64,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let action = account::decide_verification_action(
            req.provider,
            req.is_verified,
            req.verification_sent_at_ms,
            req.now_ms,
        );
        ok_json(&json!({ "verification_action": action }))
    })
}

pub extern "C" fn stride_can_resend_verification(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            verification_sent_at_ms: i64,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let can = account::can_resend_verification(
            req.verification_sent_at_ms,
            req.now_ms,
        );
        ok_json(&json!({ "can_resend": can }))
    })
}

pub extern "C" fn stride_validate_password(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            password: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result: PasswordValidationResult =
            account::validate_password(&req.password);
        ok_json(&result)
    })
}

pub extern "C" fn stride_password_strength_score(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            password: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let score = account::password_strength_score(&req.password);
        ok_json(&json!({ "score": score }))
    })
}

pub extern "C" fn stride_password_strength_label(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            score: u8,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let label = account::password_strength_label(req.score);
        ok_json(&json!({ "label": label }))
    })
}

pub extern "C" fn stride_validate_email(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            email: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match account::validate_email(&req.email) {
            Ok(()) => ok_json(&json!({ "is_valid": true, "error": null })),
            Err(msg) => ok_json(&json!({ "is_valid": false, "error": msg })),
        }
    })
}

pub extern "C" fn stride_account_status_message(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            status: AccountStatus,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let message = account::account_status_message(req.status);
        ok_json(&json!({ "message": message }))
    })
}

pub extern "C" fn stride_can_sign_in(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            status: AccountStatus,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let can = account::can_sign_in(req.status);
        ok_json(&json!({ "can_sign_in": can }))
    })
}

pub extern "C" fn stride_analyze_login_attempt(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            context: LoginContext,
            history: LoginHistory,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result: SuspiciousLoginResult =
            account::analyze_login_attempt(&req.context, &req.history);
        ok_json(&result)
    })
}

pub extern "C" fn stride_enumerate_user_data_categories(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let categories = account::enumerate_user_data_categories();
        ok_json(&json!({ "categories": categories }))
    })
}

pub extern "C" fn stride_build_deletion_plan(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            scope: DeletionScope,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let plan = account::build_deletion_plan(req.scope);
        ok_json(&json!({ "categories": plan }))
    })
}

pub extern "C" fn stride_build_deletion_result(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct FailedItem {
            category: UserDataCategory,
            error: String,
        }

        #[derive(serde::Deserialize)]
        struct Req {
            deleted: Vec<UserDataCategory>,
            failed: Vec<FailedItem>,
            auth_account_deleted: bool,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let failed: Vec<(UserDataCategory, String)> = req
            .failed
            .into_iter()
            .map(|f| (f.category, f.error))
            .collect();

        let result: DeletionResult = account::build_deletion_result(
            req.deleted,
            failed,
            req.auth_account_deleted,
        );
        ok_json(&result)
    })
}

pub extern "C" fn stride_should_auto_signout(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            session_state: SessionState,
            account_status: AccountStatus,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let should = account::should_auto_signout(
            req.session_state,
            req.account_status,
        );
        ok_json(&json!({ "should_signout": should }))
    })
}

pub extern "C" fn stride_can_upgrade_anonymous(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_provider: AuthProvider,
            target_provider: AuthProvider,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let can = account::can_upgrade_anonymous(
            req.current_provider,
            req.target_provider,
        );
        ok_json(&json!({ "can_upgrade": can }))
    })
}

// ===========================================================================
// §7 — Secure Firebase: security validation FFI entry points
// ===========================================================================

pub extern "C" fn stride_security_collection_path_template(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            collection: FirestoreCollection,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let path = req.collection.path_template();
        ok_json(&json!({ "path_template": path }))
    })
}

pub extern "C" fn stride_security_collection_is_admin_only(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            collection: FirestoreCollection,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let is_admin = req.collection.is_admin_only();
        ok_json(&json!({ "is_admin_only": is_admin }))
    })
}

pub extern "C" fn stride_security_collection_is_user_scoped(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            collection: FirestoreCollection,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let is_user_scoped = req.collection.is_user_scoped();
        ok_json(&json!({ "is_user_scoped": is_user_scoped }))
    })
}

pub extern "C" fn stride_security_check_access(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let ctx: AccessContext = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision: AccessDecision = security::check_access(&ctx);
        ok_json(&decision)
    })
}

pub extern "C" fn stride_security_validate_path_ownership(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            path: String,
            user_id: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match security::validate_path_ownership(&req.path, &req.user_id) {
            Ok(()) => ok_json(&json!({ "valid": true })),
            Err(e) => ok_json(&json!({ "valid": false, "error": e })),
        }
    })
}

pub extern "C" fn stride_security_field_rules_for_collection(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            collection: FirestoreCollection,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let rules: Vec<FieldRule> = security::field_rules_for_collection(req.collection);
        ok_json(&json!({ "rules": rules }))
    })
}

pub extern "C" fn stride_security_allowed_fields_for_collection(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            collection: FirestoreCollection,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let fields: Vec<String> = security::allowed_fields_for_collection(req.collection);
        ok_json(&json!({ "allowed_fields": fields }))
    })
}

pub extern "C" fn stride_security_validate_document(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            collection: FirestoreCollection,
            doc: serde_json::Value,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result: FieldValidationResult =
            security::validate_document(req.collection, &req.doc);
        ok_json(&result)
    })
}

pub extern "C" fn stride_security_sanitize_string(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            input: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let sanitized = security::sanitize_string(&req.input);
        ok_json(&json!({ "sanitized": sanitized }))
    })
}

pub extern "C" fn stride_security_detect_injection(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            input: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let detected = security::detect_injection(&req.input);
        ok_json(&json!({ "detected": detected }))
    })
}

pub extern "C" fn stride_security_is_safe_string(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            input: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let is_safe = security::is_safe_string(&req.input);
        ok_json(&json!({ "is_safe": is_safe }))
    })
}

pub extern "C" fn stride_security_validate_storage_path(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            path: String,
            user_id: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match security::validate_storage_path(&req.path, &req.user_id) {
            Ok(()) => ok_json(&json!({ "valid": true })),
            Err(e) => ok_json(&json!({ "valid": false, "error": e })),
        }
    })
}

pub extern "C" fn stride_security_app_check_decision(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            state: AppCheckState,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision: AccessDecision = security::app_check_decision(req.state);
        ok_json(&decision)
    })
}

pub extern "C" fn stride_security_play_integrity_decision(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            verdict: PlayIntegrityVerdict,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision: AccessDecision = security::play_integrity_decision(req.verdict);
        ok_json(&decision)
    })
}

pub extern "C" fn stride_security_check_rate_limit(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            bucket: RateLimitBucket,
            now_ms: i64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mut bucket = req.bucket;
        let result: RateLimitResult = security::check_rate_limit(&mut bucket, req.now_ms);
        ok_json(&json!({ "result": result, "bucket": bucket }))
    })
}

pub extern "C" fn stride_security_rate_limit_config(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            category: RateLimitCategory,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let (capacity, refill_rate) = security::rate_limit_config(req.category);
        ok_json(&json!({ "capacity": capacity, "refill_rate": refill_rate }))
    })
}

pub extern "C" fn stride_security_check_for_secrets(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            config: serde_json::Value,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let config = match req.config.as_object() {
            Some(m) => m,
            None => return err_json("config must be a JSON object".to_string()),
        };

        let found = security::check_for_secrets(config);
        ok_json(&json!({ "found_secrets": found }))
    })
}

pub extern "C" fn stride_security_is_secret_free(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            config: serde_json::Value,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let config = match req.config.as_object() {
            Some(m) => m,
            None => return err_json("config must be a JSON object".to_string()),
        };

        let is_free = security::is_secret_free(config);
        ok_json(&json!({ "is_secret_free": is_free }))
    })
}

pub extern "C" fn stride_security_validate_project_id(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            project_id: String,
            expected: FirebaseEnvironment,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        match security::validate_project_id(&req.project_id, req.expected) {
            Ok(()) => ok_json(&json!({ "valid": true })),
            Err(e) => ok_json(&json!({ "valid": false, "error": e })),
        }
    })
}

pub extern "C" fn stride_security_environment_from_project_id(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            project_id: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let env = security::environment_from_project_id(&req.project_id);
        ok_json(&json!({ "environment": env }))
    })
}

pub extern "C" fn stride_security_generate_firestore_rules(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let rules = security::generate_firestore_rules();
        ok_json(&json!({ "rules": rules }))
    })
}

pub extern "C" fn stride_security_generate_storage_rules(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let rules = security::generate_storage_rules();
        ok_json(&json!({ "rules": rules }))
    })
}
// ===========================================================================
// §8 — AI coaching backend FFI entry points
// ===========================================================================

/// Returns the experience-level caps (distance, duration, rest days).
pub extern "C" fn stride_coaching_plan_experience_caps(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            experience_level: ExperienceLevel,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let level = req.experience_level;
        ok_json(&json!({
            "experience_level": level,
            "label": level.label(),
            "max_single_session_distance_m": level.max_single_session_distance_m(),
            "max_single_session_duration_s": level.max_single_session_duration_s(),
            "max_weekly_distance_m": level.max_weekly_distance_m(),
            "recommended_rest_days_per_week": level.recommended_rest_days_per_week(),
        }))
    })
}

/// Validates a single day plan.
pub extern "C" fn stride_coaching_plan_validate_day_plan(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            day: DayPlan,
            experience_level: ExperienceLevel,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let issues = coaching_plan::validate_day_plan(&req.day, req.experience_level);
        ok_json(&json!({ "issues": issues }))
    })
}

/// Validates a weekly plan.
pub extern "C" fn stride_coaching_plan_validate_weekly_plan(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            plan: WeeklyPlan,
            previous_weekly_distance_m: Option<f64>,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = coaching_plan::validate_weekly_plan(&req.plan, req.previous_weekly_distance_m);
        ok_json(&result)
    })
}

/// Generates a fallback weekly plan.
pub extern "C" fn stride_coaching_plan_generate_fallback(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            experience_level: ExperienceLevel,
            current_weekly_distance_m: f64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let plan = coaching_plan::generate_fallback_plan(req.experience_level, req.current_weekly_distance_m);
        ok_json(&plan)
    })
}

/// Responds to user-reported pain.
pub extern "C" fn stride_coaching_plan_respond_to_pain(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            pain: PainType,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let response = coaching_plan::respond_to_pain(req.pain);
        ok_json(&response)
    })
}

/// Checks if text contains medical diagnosis language.
pub extern "C" fn stride_coaching_plan_contains_diagnosis(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            text: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let contains = coaching_plan::contains_diagnosis(&req.text);
        ok_json(&json!({ "contains_diagnosis": contains }))
    })
}

/// Checks if text contains weight-loss promise language.
pub extern "C" fn stride_coaching_plan_contains_weight_loss_promise(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            text: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let contains = coaching_plan::contains_weight_loss_promise(&req.text);
        ok_json(&json!({ "contains_weight_loss_promise": contains }))
    })
}

/// Validates AI-generated coaching text against content guards.
pub extern "C" fn stride_coaching_plan_validate_coaching_text(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            text: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = coaching_plan::validate_coaching_text(&req.text);
        ok_json(&result)
    })
}

/// Generates an escalation message for concerning symptoms.
pub extern "C" fn stride_coaching_plan_escalation_message(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            reason: EscalationReason,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let message = coaching_plan::escalation_message(req.reason);
        ok_json(&message)
    })
}

/// Processes user feedback on a plan.
pub extern "C" fn stride_coaching_plan_process_user_feedback(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            feedback: UserFeedback,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = coaching_plan::process_user_feedback(req.feedback);
        ok_json(&result)
    })
}

/// Decides whether the AI should be called or the fallback used.
pub extern "C" fn stride_coaching_plan_decide_ai_availability(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            is_service_up: bool,
            remaining_quota: Option<u32>,
            estimated_cost_cents: u64,
            cost_budget_cents: Option<u64>,
            is_enabled: bool,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let availability = coaching_plan::decide_ai_availability(
            req.is_service_up,
            req.remaining_quota,
            req.estimated_cost_cents,
            req.cost_budget_cents,
            req.is_enabled,
        );
        ok_json(&availability)
    })
}

/// Whether the fallback should be used instead of calling the AI.
pub extern "C" fn stride_coaching_plan_should_use_fallback(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            availability: AiAvailability,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let should = coaching_plan::should_use_fallback(req.availability);
        ok_json(&json!({ "should_use_fallback": should }))
    })
}

/// Estimates the cost in cents for an AI operation.
pub extern "C" fn stride_coaching_plan_estimate_ai_cost(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            operation: AiOperation,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let cost = coaching_plan::estimate_ai_cost_cents(req.operation);
        ok_json(&json!({ "cost_cents": cost }))
    })
}

/// Generates a deterministic cache key for an AI request.
pub extern "C" fn stride_coaching_plan_cache_key(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            operation: AiOperation,
            user_input: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let key = coaching_plan::cache_key(req.operation, &req.user_input);
        ok_json(&json!({ "cache_key": key }))
    })
}

/// Summarizes a completed workout (fallback when AI unavailable).
pub extern "C" fn stride_coaching_plan_summarize_workout(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let input: WorkoutSummaryInput = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let summary = coaching_plan::summarize_workout(&input);
        ok_json(&json!({ "summary": summary }))
    })
}

/// Generates a rule-based encouragement message.
pub extern "C" fn stride_coaching_plan_generate_encouragement(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            days_active_last_week: u32,
            total_distance_last_week_m: f64,
            goal_distance_m: f64,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let message = coaching_plan::generate_encouragement(
            req.days_active_last_week,
            req.total_distance_last_week_m,
            req.goal_distance_m,
        );
        ok_json(&json!({ "message": message }))
    })
}

/// Adjusts a plan based on user feedback (fallback).
pub extern "C" fn stride_coaching_plan_adjust_plan(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let input: PlanAdjustmentInput = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = coaching_plan::adjust_plan(&input);
        ok_json(&result)
    })
}

/// Recommends a realistic progression for the next week.
pub extern "C" fn stride_coaching_plan_recommend_progression(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_weekly_distance_m: f64,
            experience_level: ExperienceLevel,
            weeks_at_current_level: u32,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let recommended = coaching_plan::recommend_progression(
            req.current_weekly_distance_m,
            req.experience_level,
            req.weeks_at_current_level,
        );
        ok_json(&json!({ "recommended_weekly_distance_m": recommended }))
    })
}

/// Moderates a user request for safety.
pub extern "C" fn stride_coaching_plan_moderate_request(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            request: String,
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = coaching_plan::moderate_request(&req.request);
        ok_json(&result)
    })
}

// ─── §9 — Calorie/fitness calculations ─────────────────────────────

/// Produces a full calorie estimate (with method, version, labels, and
/// source-priority) for the given inputs, following the documented
/// source-priority chain: wearable > heart_rate > met > distance_weight.
/// The result always includes the calculation version and a display
/// label, per spec section 9.
#[no_mangle]
pub extern "C" fn stride_calorie_estimate(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let inputs: CalorieInputs = match serde_json::from_str(&raw) {
            Ok(i) => i,
            Err(e) => return err_json(format!("invalid_calorie_inputs: {e}")),
        };

        let result: CalorieEstimateResult = calories::estimate_with_details(&inputs);
        ok_json(&result)
    })
}

/// Validates and clamps a calorie value to a plausible range (0–10,000
/// kcal), returning the clamped value along with whether it was modified
/// from the original input.
#[no_mangle]
pub extern "C" fn stride_calorie_clamp(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            kcal: f64,
        }
        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let clamped = calories::clamp_to_plausible(req.kcal);
        let was_modified = (clamped - req.kcal).abs() > f64::EPSILON
            || !req.kcal.is_finite();

        #[derive(serde::Serialize)]
        struct Resp {
            original: f64,
            clamped: f64,
            was_modified: bool,
            is_plausible: bool,
        }
        let resp = Resp {
            original: if req.kcal.is_finite() { req.kcal } else { 0.0 },
            clamped,
            was_modified,
            is_plausible: calories::is_plausible_kcal(clamped),
        };
        ok_json(&resp)
    })
}

/// Returns the source-priority order for calorie estimation methods, as a
/// list of (method_str, rank) pairs. This documents the fallback chain
/// for the UI and debugging.
#[no_mangle]
pub extern "C" fn stride_calorie_source_priority() -> *mut c_char {
    guarded(move || {
        #[derive(serde::Serialize)]
        struct Entry {
            method: String,
            rank: u8,
            label: String,
        }
        let entries: Vec<Entry> = calories::source_priority_order()
            .into_iter()
            .map(|(m, r)| Entry {
                method: m.as_str().to_string(),
                rank: r,
                label: m.label().to_string(),
            })
            .collect();
        ok_json(&entries)
    })
}

// ─── §10 — Wearable / Health Connect ──────────────────────────────

/// Decides the appropriate operating mode (phone-only, watch, or Health
/// Connect) given the current source availability. Returns a full
/// `FallbackDecision` with which sources to use for steps, distance, and
/// heart rate, plus a human-readable message.
#[no_mangle]
pub extern "C" fn stride_wearable_decide_fallback(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let availability: SourceAvailability = match serde_json::from_str(&raw) {
            Ok(a) => a,
            Err(e) => return err_json(format!("invalid_source_availability: {e}")),
        };

        let decision: FallbackDecision = wearable::decide_fallback(&availability);
        ok_json(&decision)
    })
}

/// Builds a full wearable status snapshot, combining the connection
/// state, consent state, sync status, fallback decision, revocation
/// log, and device info into a single `WearableStatus` for the UI.
#[no_mangle]
pub extern "C" fn stride_wearable_build_status(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            availability: SourceAvailability,
            consent_state: HealthConnectConsentState,
            last_sync_ms: Option<i64>,
            now_ms: i64,
            sync_config: WearableSyncConfig,
            revocations: Vec<SourceRevocationRecord>,
            device: Option<WearableDeviceInfo>,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let status: WearableStatus = wearable::build_status(
            &req.availability,
            req.consent_state,
            req.last_sync_ms,
            req.now_ms,
            &req.sync_config,
            &req.revocations,
            req.device,
        );
        ok_json(&status)
    })
}

/// Evaluates the current sync status of a wearable, given the
/// connection state, the time of the last successful sync, and the
/// current time. Returns the `WearableSyncStatus`.
#[no_mangle]
pub extern "C" fn stride_wearable_sync_status(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            connection_state: WearableConnectionState,
            last_sync_ms: Option<i64>,
            now_ms: i64,
            stale_threshold_ms: i64,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let config = WearableSyncConfig {
            stale_threshold_ms: req.stale_threshold_ms,
        };
        let status: WearableSyncStatus = wearable::evaluate_sync_status(
            req.connection_state,
            req.last_sync_ms,
            req.now_ms,
            &config,
        );

        #[derive(serde::Serialize)]
        struct Resp {
            sync_status: WearableSyncStatus,
            label: String,
            is_current: bool,
        }
        let resp = Resp {
            is_current: status.is_current(),
            label: status.label().to_string(),
            sync_status: status,
        };
        ok_json(&resp)
    })
}

/// Decides which source wins when the same metric arrives from two
/// sources at approximately the same time (duplicate-record
/// prevention). Returns the winning `SensorSource`.
#[no_mangle]
pub extern "C" fn stride_wearable_deduplicate_source(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            source_a: crate::models::SensorSource,
            source_b: crate::models::SensorSource,
            metric: MetricType,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let winner = wearable::deduplicate_source(req.source_a, req.source_b, req.metric);

        #[derive(serde::Serialize)]
        struct Resp {
            winner: crate::models::SensorSource,
            winner_priority: u8,
        }
        let resp = Resp {
            winner_priority: winner.priority(),
            winner,
        };
        ok_json(&resp)
    })
}

/// Processes a Health Connect consent request result, transitioning
/// the consent state. Returns the new `HealthConnectConsentState`.
#[no_mangle]
pub extern "C" fn stride_wearable_consent_result(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_state: HealthConnectConsentState,
            granted: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let new_state = req.current_state.on_request_result(req.granted);

        #[derive(serde::Serialize)]
        struct Resp {
            consent_state: HealthConnectConsentState,
            label: String,
            can_read: bool,
        }
        let resp = Resp {
            can_read: new_state.can_read(),
            label: new_state.label().to_string(),
            consent_state: new_state,
        };
        ok_json(&resp)
    })
}

// ── §11 — Music system ────────────────────────────────────────────────────

/// Processes a playback state transition. Given the current playback
/// state and a command, returns a `TransitionResult` with the new state
/// (or unchanged state if the transition is invalid) and a message.
#[no_mangle]
pub extern "C" fn stride_music_transition(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_state: PlaybackState,
            command: PlaybackCommand,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result: TransitionResult =
            music::transition(req.current_state, req.command);
        ok_json(&result)
    })
}

/// Handles an audio focus event. Given the current audio focus state and
/// the event type, returns the new focus state, recommended playback
/// action, and metadata.
#[no_mangle]
pub extern "C" fn stride_music_audio_focus(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_focus: AudioFocusState,
            event: AudioFocusEvent,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let (new_focus, action) =
            music::handle_audio_focus_event(req.current_focus, req.event);

        #[derive(serde::Serialize)]
        struct Resp {
            new_focus: AudioFocusState,
            action: PlaybackAction,
            can_play: bool,
            volume_multiplier: f64,
            focus_label: String,
        }
        let resp = Resp {
            can_play: new_focus.can_play(),
            volume_multiplier: new_focus.volume_multiplier(),
            focus_label: new_focus.label().to_string(),
            new_focus,
            action,
        };
        ok_json(&resp)
    })
}

/// Coordinates music with a coaching prompt. Given the current coaching-
/// interop state, the request type, and whether music is playing, returns
/// a `CoachingInteropResult` with the music action to take.
#[no_mangle]
pub extern "C" fn stride_music_coaching_interop(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_state: CoachingInteropState,
            request: CoachingRequest,
            music_is_playing: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = music::handle_coaching_request(
            req.current_state,
            req.request,
            req.music_is_playing,
        );
        ok_json(&result)
    })
}

/// Decides what to do when the network is lost during music streaming.
/// Given the network state, current source, local-media availability, and
/// buffer health, returns a `NetworkLossDecision`.
#[no_mangle]
pub extern "C" fn stride_music_network_loss(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            network: NetworkState,
            current_source: MusicSource,
            has_local_media: bool,
            buffer_health_ms: i64,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision: NetworkLossDecision = music::handle_network_loss(
            req.network,
            req.current_source,
            req.has_local_media,
            req.buffer_health_ms,
        );
        ok_json(&decision)
    })
}

/// Filters a playlist, removing tracks by blocked artists or genres.
/// Returns the filtered playlist and a count of removed tracks.
#[no_mangle]
pub extern "C" fn stride_music_filter_blocked(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            playlist: Playlist,
            blocked_artists: Vec<String>,
            blocked_genres: Vec<String>,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let (filtered, removed) = music::filter_blocked_content(
            &req.playlist,
            &req.blocked_artists,
            &req.blocked_genres,
        );

        #[derive(serde::Serialize)]
        struct Resp {
            filtered_playlist: Playlist,
            removed_count: usize,
        }
        let resp = Resp {
            filtered_playlist: filtered,
            removed_count: removed,
        };
        ok_json(&resp)
    })
}

/// Decides whether a track should be recommended again based on the
/// user's feedback history. Returns a boolean indicating whether the
/// track should be recommended.
#[no_mangle]
pub extern "C" fn stride_music_should_recommend(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            feedback_history: Vec<FeedbackRecord>,
            track_id: String,
            now_ms: i64,
            skip_cooldown_ms: i64,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let should = music::should_recommend_track(
            &req.feedback_history,
            &req.track_id,
            req.now_ms,
            req.skip_cooldown_ms,
        );

        #[derive(serde::Serialize)]
        struct Resp {
            should_recommend: bool,
        }
        let resp = Resp {
            should_recommend: should,
        };
        ok_json(&resp)
    })
}

/// Builds a full music status snapshot from the current state. Given
/// the playback state, audio focus, coaching-interop state, music
/// source, mode, network state, current track index, and an optional
/// playlist, returns a `MusicStatus` suitable for the UI.
#[no_mangle]
pub extern "C" fn stride_music_build_status(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            playback: PlaybackState,
            focus: AudioFocusState,
            coaching: CoachingInteropState,
            source: MusicSource,
            mode: MusicMode,
            network: NetworkState,
            current_track_index: Option<usize>,
            playlist: Option<Playlist>,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let status: MusicStatus = music::build_status(
            req.playback,
            req.focus,
            req.coaching,
            req.source,
            req.mode,
            req.network,
            req.current_track_index,
            req.playlist.as_ref(),
        );
        ok_json(&status)
    })
}

/// Processes a remote control command (from a lock screen, Bluetooth
/// headset, Wear OS, or notification). Returns the resulting
/// `TransitionResult`.
#[no_mangle]
pub extern "C" fn stride_music_remote_control(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current: PlaybackState,
            command: PlaybackCommand,
            source: RemoteControlSource,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result: TransitionResult = music::handle_remote_control(
            req.current,
            req.command,
            req.source,
        );
        ok_json(&result)
    })
}

// ─── §12 — Background execution ────────────────────────────────────

/// Transitions the foreground service state machine. Given the current
/// service state and a command, returns a `ForegroundServiceTransition`
/// with the new state, notification text, and whether to keep the
/// service alive.
#[no_mangle]
pub extern "C" fn stride_background_service_transition(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            current_state: ForegroundServiceState,
            command: ForegroundServiceCommand,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result: ForegroundServiceTransition =
            background::transition_service(req.current_state, req.command);
        ok_json(&result)
    })
}

/// Decides the checkpoint write interval given the workout phase and
/// battery state. Returns a `CheckpointSchedule` with the interval,
/// max acceptable data loss, and a reason string.
#[no_mangle]
pub extern "C" fn stride_background_checkpoint_interval(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            phase: WorkoutPhase,
            battery: CheckpointBatteryContext,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let schedule: CheckpointSchedule =
            background::decide_checkpoint_interval(req.phase, req.battery);
        ok_json(&schedule)
    })
}

/// Evaluates whether background execution is permitted right now, given
/// the user's preference, service state, permissions, workout state, and
/// battery. Returns a `BackgroundExecutionDecision`.
#[no_mangle]
pub extern "C" fn stride_background_evaluate(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let ctx: BackgroundExecutionContext = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision: BackgroundExecutionDecision =
            background::evaluate_background_execution(&ctx);
        ok_json(&decision)
    })
}

/// Evaluates whether a workout can be resumed after a process kill or
/// device restart. Given the checkpoint age in milliseconds and whether
/// the workout was active, returns a `ProcessKillRecoveryDecision`.
#[no_mangle]
pub extern "C" fn stride_background_process_kill(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            checkpoint_age_ms: i64,
            was_active: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision: ProcessKillRecoveryDecision =
            background::evaluate_process_kill_recovery(
                req.checkpoint_age_ms,
                req.was_active,
            );
        ok_json(&decision)
    })
}

/// Decides the power mode and notification update interval based on the
/// battery state. Returns the power mode, notification interval, and
/// checkpoint interval for the given battery context.
#[no_mangle]
pub extern "C" fn stride_background_power_mode(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let battery: CheckpointBatteryContext = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mode: PowerMode = background::decide_power_mode(&battery);
        let notif_interval = background::notification_update_interval_ms(mode);

        #[derive(serde::Serialize)]
        struct Resp {
            power_mode: PowerMode,
            notification_interval_ms: u64,
        }
        let resp = Resp {
            power_mode: mode,
            notification_interval_ms: notif_interval,
        };
        ok_json(&resp)
    })
}

/// Handles an interruption event (screen off, incoming call, low memory,
/// device shutdown, etc.) during a background workout. Returns the
/// recommended `InterruptionAction`.
#[no_mangle]
pub extern "C" fn stride_background_interruption(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            event: InterruptionEvent,
            service_state: ForegroundServiceState,
            background_allowed: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let action: InterruptionAction = background::handle_interruption(
            req.event,
            req.service_state,
            req.background_allowed,
        );
        ok_json(&action)
    })
}

/// Assesses the current battery usage and whether it's acceptable. Given
/// the GPS interval, sensor interval, battery percentage, and charging
/// status, returns a `BatteryUseAssessment`.
#[no_mangle]
pub extern "C" fn stride_background_battery_assessment(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            gps_interval_ms: u32,
            sensor_interval_ms: u32,
            battery_percent: u8,
            is_charging: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let assessment = background::assess_battery_use(
            req.gps_interval_ms,
            req.sensor_interval_ms,
            req.battery_percent,
            req.is_charging,
        );
        ok_json(&assessment)
    })
}

/// Builds a full background status snapshot for the UI / diagnostics.
/// Given the service state, battery, preference, permission, workout
/// state, and sampling intervals, returns a `BackgroundStatus`.
#[no_mangle]
pub extern "C" fn stride_background_build_status(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            service_state: ForegroundServiceState,
            battery: CheckpointBatteryContext,
            preference: BackgroundTrackingPreference,
            background_location_granted: bool,
            workout_active: bool,
            gps_interval_ms: u32,
            sensor_interval_ms: u32,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let status: BackgroundStatus = background::build_background_status(
            req.service_state,
            &req.battery,
            req.preference,
            req.background_location_granted,
            req.workout_active,
            req.gps_interval_ms,
            req.sensor_interval_ms,
        );
        ok_json(&status)
    })
}

/// Returns the full background location explanation text for the
/// Google Play Store data safety form and in-app rationale dialog.
#[no_mangle]
pub extern "C" fn stride_background_explanation(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            short: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(_) => Req { short: false },
        };

        let text = if req.short {
            background::background_location_short_explanation()
        } else {
            background::background_location_explanation()
        };

        ok_json(&text)
    })
}

// ──────────────────────────────────────────────────────────────────────
// §13 — Notifications
// ──────────────────────────────────────────────────────────────────────

/// Makes a complete notification decision: should the notification be
/// delivered, when, and with what content? Given the notification type,
/// preferences, permission, quiet hours, current time, and timezone,
/// returns a `NotificationDecision`.
#[no_mangle]
pub extern "C" fn stride_notification_decide(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        let ctx: NotificationDecisionContext = match serde_json::from_str(&raw) {
            Ok(c) => c,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision: NotificationDecision = notifications::decide_notification(&ctx);
        ok_json(&decision)
    })
}

/// Builds the notification content (title, body, action label) for a
/// given notification type and optional context params. Returns a
/// `NotificationContent`.
#[no_mangle]
pub extern "C" fn stride_notification_content(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            notif_type: NotificationType,
            context_params: notifications::NotificationContextParams,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let content: NotificationContent =
            notifications::build_notification_content(req.notif_type, &req.context_params);
        ok_json(&content)
    })
}

/// Evaluates whether a notification should be delivered now given the
/// quiet hours configuration. Returns a `QuietHoursDecision`-like
/// struct with `is_quiet`, `should_deliver_now`, `reschedule_to_minute`,
/// and `reason`.
#[no_mangle]
pub extern "C" fn stride_notification_quiet_hours(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            quiet_hours: QuietHoursConfig,
            notif_type: NotificationType,
            local_minute: u32,
            critical_bypass: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let decision = notifications::evaluate_quiet_hours(
            req.quiet_hours,
            req.notif_type,
            req.local_minute,
            req.critical_bypass,
        );
        ok_json(&decision)
    })
}

/// Evaluates whether a voice coaching announcement should be made now.
/// Given the coaching config, quiet hours, announcement kind, last
/// announcement distance/time, and local minute, returns a boolean
/// decision.
#[no_mangle]
pub extern "C" fn stride_notification_coaching(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            voice_config: VoiceCoachingConfig,
            quiet_hours: QuietHoursConfig,
            kind: CoachingAnnouncementKind,
            last_distance_m: f64,
            last_time_s: u64,
            local_minute: u32,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let should = notifications::should_announce(
            &req.voice_config,
            req.quiet_hours,
            req.kind,
            req.last_distance_m,
            req.last_time_s,
            req.local_minute,
        );
        ok_json(&should)
    })
}

/// Builds a full notification status snapshot for the UI / diagnostics.
/// Given the permission, preferences, quiet hours, voice coaching
/// config, and timezone, returns a `NotificationStatus`.
#[no_mangle]
pub extern "C" fn stride_notification_status(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            permission: NotificationPermission,
            preferences: NotificationPreferences,
            quiet_hours: QuietHoursConfig,
            voice_config: VoiceCoachingConfig,
            timezone: TimezoneContext,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let status: NotificationStatus = notifications::build_notification_status(
            req.permission,
            req.preferences,
            req.quiet_hours,
            &req.voice_config,
            req.timezone,
        );
        ok_json(&status)
    })
}

/// Computes the UTC timestamp (epoch milliseconds) for the next daily
/// reminder at a given local-time minute. Given the current UTC time,
/// local minute, and timezone, returns the UTC ms timestamp.
#[no_mangle]
pub extern "C" fn stride_notification_next_reminder(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            now_utc_ms: i64,
            local_minute: u32,
            timezone: TimezoneContext,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let utc_ms = notifications::next_daily_reminder_utc_ms(
            req.now_utc_ms,
            req.local_minute,
            req.timezone,
        );
        ok_json(&utc_ms)
    })
}

// ---------------------------------------------------------------------------
// §14 Error / Recovery States
// ---------------------------------------------------------------------------

/// Decides the recovery action for a given error context and retry policy.
///
/// `request_json` shape:
/// ```json
/// {
///   "error": { "category": "gps", "severity": "high", "code": "gps_no_fix",
///              "message": "...", "subsystem": "gps",
///              "timestamp_ms": 0, "retry_count": 2,
///              "metadata": [["key","value"]] },
///   "policy": { "max_attempts": 5, "base_delay_ms": 1000,
///               "max_delay_ms": 600000, "backoff_multiplier": 2.0,
///               "jitter_fraction": 0.25 }
/// }
/// ```
/// Returns `RecoveryAction` as JSON.
#[no_mangle]
pub extern "C" fn stride_error_decide_recovery(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            error: ErrorContext,
            policy: RetryPolicy,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let action = error_states::decide_recovery(&req.error, &req.policy);
        ok_json(&action)
    })
}

/// Computes the retry delay in milliseconds for a given attempt number
/// and retry policy using exponential backoff + jitter.
///
/// `request_json` shape:
/// ```json
/// { "attempt": 2, "policy": { ... RetryPolicy ... } }
/// ```
/// Returns the delay in milliseconds as a JSON integer.
#[no_mangle]
pub extern "C" fn stride_error_retry_delay(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            attempt: u32,
            policy: RetryPolicy,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let delay_ms = error_states::compute_retry_delay(req.attempt, &req.policy);
        ok_json(&delay_ms)
    })
}

/// Validates and performs an error state transition.
///
/// `request_json` shape:
/// ```json
/// { "from": "detected", "to": "recovering" }
/// ```
/// Returns `ErrorStateTransition` as JSON with `is_valid` and `message`.
#[no_mangle]
pub extern "C" fn stride_error_transition(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            from: ErrorState,
            to: ErrorState,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let transition = error_states::transition_error_state(req.from, req.to);
        ok_json(&transition)
    })
}

/// Builds an overall engine health status from an error registry.
///
/// `request_json` shape:
/// ```json
/// { "registry": { "entries": [...], "max_entries": 100 } }
/// ```
/// Returns `EngineHealthStatus` as JSON.
#[no_mangle]
pub extern "C" fn stride_error_health_status(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            registry: ErrorRegistry,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let status = error_states::build_health_status(&req.registry);
        ok_json(&status)
    })
}

/// Returns whether errors of a given category are retryable.
///
/// `request_json` shape:
/// ```json
/// { "category": "gps" }
/// ```
/// Returns a JSON boolean.
#[no_mangle]
pub extern "C" fn stride_error_is_recoverable(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            category: ErrorCategory,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let retryable = error_states::is_retryable(req.category);
        ok_json(&retryable)
    })
}

// ===========================================================================
// §15 — Testing
// ===========================================================================

/// Runs a test suite (simulated) and returns the suite with results.
///
/// `request_json` shape:
/// ```json
/// { "suite": { "name": "...", "layer": "unit", "configs": [...], "results": [] } }
/// ```
/// Returns a `TestSuite` as JSON with results filled in (all passed,
/// zero duration — simulated run).
#[no_mangle]
pub extern "C" fn stride_testing_run_suite(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            suite: TestSuite,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = testing::run_suite(&req.suite);
        ok_json(&result)
    })
}

/// Runs a real-device scenario (simulated) and returns the result.
///
/// `request_json` shape:
/// ```json
/// { "scenario": { "id": "...", "name": "...", ... } }
/// ```
/// Returns a `ScenarioResult` as JSON (all behaviors passed,
/// zero duration — simulated run).
#[no_mangle]
pub extern "C" fn stride_testing_run_scenario(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            scenario: RealDeviceScenario,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = testing::run_scenario(&req.scenario);
        ok_json(&result)
    })
}

/// Builds a test report from a test registry.
///
/// `request_json` shape:
/// ```json
/// { "name": "...", "registry": { "suites": [...], "scenarios": [...], "scenario_results": [] } }
/// ```
/// Returns a `TestReport` as JSON with aggregated stats and coverage.
#[no_mangle]
pub extern "C" fn stride_testing_build_report(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            name: String,
            registry: TestRegistry,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let report = testing::build_report(req.name, &req.registry);
        ok_json(&report)
    })
}

/// Looks up a test config by its id from the full test registry.
///
/// `request_json` shape:
/// ```json
/// { "test_id": "unit_distance_haversine_known" }
/// ```
/// Returns a `TestConfig` as JSON, or an error if not found.
#[no_mangle]
pub extern "C" fn stride_testing_get_config(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            test_id: String,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let registry = testing::build_full_registry();
        for suite in &registry.suites {
            for config in &suite.configs {
                if config.id == req.test_id {
                    return ok_json(config);
                }
            }
        }
        err_json(format!("test_not_found: {}", req.test_id))
    })
}

/// Returns the list of all standard test suite names.
///
/// `request_json` shape: `{ }` (empty JSON object)
/// Returns a `Vec<String>` as JSON — the names of all suites in the
/// full registry.
#[no_mangle]
pub extern "C" fn stride_testing_list_suites(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let registry = testing::build_full_registry();
        let names = registry.suite_names();
        ok_json(&names)
    })
}

// ─── §16 Monitoring ────────────────────────────────────────────────────

/// Checks all alert thresholds against the provided metric values and
/// returns every alert that was triggered.
///
/// `request_json` shape:
/// ```json
/// {
///   "thresholds": { … AlertThresholds … },
///   "timestamp_ms": 1700000000000,
///   "cloud_function_error_rate": 0.08,
///   "cloud_function_latency_ms": 6000,
///   "firestore_reads_per_day": 55000,
///   "firestore_writes_per_day": 12000,
///   "firestore_deletes_per_day": 3000,
///   "storage_usage_bytes": 9000000000,
///   "storage_bandwidth_bytes": 5000000000,
///   "billing_budget_cents": 5000,
///   "ai_cost_per_day_cents": 200,
///   "ai_error_rate": 0.15,
///   "sync_failure_rate": 0.10,
///   "uptime": 0.98,
///   "crash_rate_per_1000": 5.0
/// }
/// ```
/// Returns a `Vec<Alert>` as JSON — all alerts that exceeded their
/// thresholds.
#[no_mangle]
pub extern "C" fn stride_monitoring_check_alerts(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            thresholds: AlertThresholds,
            timestamp_ms: i64,
            cloud_function_error_rate: f64,
            cloud_function_latency_ms: u64,
            firestore_reads_per_day: u64,
            firestore_writes_per_day: u64,
            firestore_deletes_per_day: u64,
            storage_usage_bytes: u64,
            storage_bandwidth_bytes: u64,
            billing_budget_cents: u64,
            ai_cost_per_day_cents: u64,
            ai_error_rate: f64,
            sync_failure_rate: f64,
            uptime: f64,
            crash_rate_per_1000: f64,
        }

        impl Default for Req {
            fn default() -> Self {
                Self {
                    thresholds: AlertThresholds::default(),
                    timestamp_ms: 0,
                    cloud_function_error_rate: 0.0,
                    cloud_function_latency_ms: 0,
                    firestore_reads_per_day: 0,
                    firestore_writes_per_day: 0,
                    firestore_deletes_per_day: 0,
                    storage_usage_bytes: 0,
                    storage_bandwidth_bytes: 0,
                    billing_budget_cents: 0,
                    ai_cost_per_day_cents: 0,
                    ai_error_rate: 0.0,
                    sync_failure_rate: 0.0,
                    uptime: 1.0,
                    crash_rate_per_1000: 0.0,
                }
            }
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let t = &req.thresholds;
        let ts = req.timestamp_ms;
        let mut alerts: Vec<Alert> = Vec::new();

        if let Some(a) = t.check_cloud_function_error_rate(ts, req.cloud_function_error_rate) {
            alerts.push(a);
        }
        if let Some(a) = t.check_cloud_function_latency(ts, req.cloud_function_latency_ms) {
            alerts.push(a);
        }
        if let Some(a) = t.check_firestore_reads(ts, req.firestore_reads_per_day) {
            alerts.push(a);
        }
        if let Some(a) = t.check_firestore_writes(ts, req.firestore_writes_per_day) {
            alerts.push(a);
        }
        if let Some(a) = t.check_storage_usage(ts, req.storage_usage_bytes) {
            alerts.push(a);
        }
        if let Some(a) = t.check_ai_cost(ts, req.ai_cost_per_day_cents) {
            alerts.push(a);
        }
        if let Some(a) = t.check_ai_error_rate(ts, req.ai_error_rate) {
            alerts.push(a);
        }
        if let Some(a) = t.check_sync_failure_rate(ts, req.sync_failure_rate) {
            alerts.push(a);
        }
        if let Some(a) = t.check_uptime(ts, req.uptime) {
            alerts.push(a);
        }
        if let Some(a) = t.check_billing_budget(ts, req.billing_budget_cents) {
            alerts.push(a);
        }
        if let Some(a) = t.check_crash_rate(ts, req.crash_rate_per_1000) {
            alerts.push(a);
        }

        ok_json(&alerts)
    })
}

/// Builds a release-health dashboard from the provided metric values.
///
/// `request_json` shape:
/// ```json
/// {
///   "app_version": "1.2.0",
///   "generated_at_ms": 1700000000000,
///   "crash_free_rate": 0.98,
///   "active_users_24h": 1500,
///   "workouts_24h": 300,
///   "sync_failure_rate": 0.05,
///   "ai_error_rate": 0.03,
///   "overall_uptime": 0.99,
///   "ai_cost_24h_cents": 200,
///   "firestore_reads_24h": 5000,
///   "storage_usage_bytes": 1000000000,
///   "alerts": [ … Alert … ]
/// }
/// ```
/// Returns a `ReleaseHealthDashboard` as JSON.
#[no_mangle]
pub extern "C" fn stride_monitoring_build_dashboard(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            app_version: String,
            generated_at_ms: i64,
            crash_free_rate: f64,
            active_users_24h: u32,
            workouts_24h: u32,
            sync_failure_rate: f64,
            ai_error_rate: f64,
            overall_uptime: f64,
            ai_cost_24h_cents: u64,
            firestore_reads_24h: u64,
            storage_usage_bytes: u64,
            alerts: Vec<Alert>,
        }

        impl Default for Req {
            fn default() -> Self {
                Self {
                    app_version: String::new(),
                    generated_at_ms: 0,
                    crash_free_rate: 1.0,
                    active_users_24h: 0,
                    workouts_24h: 0,
                    sync_failure_rate: 0.0,
                    ai_error_rate: 0.0,
                    overall_uptime: 1.0,
                    ai_cost_24h_cents: 0,
                    firestore_reads_24h: 0,
                    storage_usage_bytes: 0,
                    alerts: Vec::new(),
                }
            }
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mut dash = ReleaseHealthDashboard::new(req.app_version, req.generated_at_ms);
        dash.crash_free_rate = req.crash_free_rate;
        dash.active_users_24h = req.active_users_24h;
        dash.workouts_24h = req.workouts_24h;
        dash.sync_failure_rate = req.sync_failure_rate;
        dash.ai_error_rate = req.ai_error_rate;
        dash.overall_uptime = req.overall_uptime;
        dash.ai_cost_24h_cents = req.ai_cost_24h_cents;
        dash.firestore_reads_24h = req.firestore_reads_24h;
        dash.storage_usage_bytes = req.storage_usage_bytes;
        for a in req.alerts {
            dash.add_alert(a);
        }
        dash.recompute_status();
        ok_json(&dash)
    })
}

/// Creates a structured log entry.
///
/// `request_json` shape:
/// ```json
/// {
///   "timestamp_ms": 1700000000000,
///   "level": "info",
///   "category": "workout_engine",
///   "message": "Workout started",
///   "context": [{"key": "session_id", "value": "abc123"}],
///   "session_id": "abc123",
///   "user_id": "user456"
/// }
/// ```
/// Returns a `LogEntry` as JSON.
#[no_mangle]
pub extern "C" fn stride_monitoring_log_entry(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            timestamp_ms: i64,
            level: LogLevel,
            category: MonitoringCategory,
            message: String,
            #[serde(default)]
            context: Vec<KeyValuePair>,
            #[serde(default)]
            session_id: Option<String>,
            #[serde(default)]
            user_id: Option<String>,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mut entry = LogEntry::new(req.timestamp_ms, req.level, req.category, req.message);
        for pair in req.context {
            entry = entry.with_context(pair.key, pair.value);
        }
        if let Some(sid) = req.session_id {
            entry = entry.with_session(sid);
        }
        if let Some(uid) = req.user_id {
            entry = entry.with_user(uid);
        }
        ok_json(&entry)
    })
}

/// Creates a crash report.
///
/// `request_json` shape:
/// ```json
/// {
///   "timestamp_ms": 1700000000000,
///   "severity": "non_fatal",
///   "exception_type": "NullPointerException",
///   "message": "Failed to update UI",
///   "stack_trace": "at com.example...",
///   "breadcrumbs": [ … LogEntry … ],
///   "app_version": "1.2.0",
///   "device_model": "Pixel 7",
///   "os_version": "Android 14",
///   "during_workout": true,
///   "session_id": "abc123"
/// }
/// ```
/// Returns a `CrashReport` as JSON.
#[no_mangle]
pub extern "C" fn stride_monitoring_crash_report(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            timestamp_ms: i64,
            severity: CrashSeverity,
            exception_type: String,
            message: String,
            #[serde(default)]
            stack_trace: Option<String>,
            #[serde(default)]
            breadcrumbs: Vec<LogEntry>,
            #[serde(default)]
            app_version: String,
            #[serde(default)]
            device_model: String,
            #[serde(default)]
            os_version: String,
            #[serde(default)]
            during_workout: bool,
            #[serde(default)]
            session_id: Option<String>,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mut report = CrashReport::new(
            req.timestamp_ms,
            req.severity,
            req.exception_type,
            req.message,
        );
        if let Some(trace) = req.stack_trace {
            report = report.with_stack_trace(trace);
        }
        for crumb in req.breadcrumbs {
            report.add_breadcrumb(crumb);
        }
        if !req.app_version.is_empty() {
            report = report.with_app_version(req.app_version);
        }
        if !req.device_model.is_empty() {
            report = report.with_device_model(req.device_model);
        }
        if !req.os_version.is_empty() {
            report = report.with_os_version(req.os_version);
        }
        report = report.with_during_workout(req.during_workout);
        if let Some(sid) = req.session_id {
            report = report.with_session(sid);
        }
        ok_json(&report)
    })
}

/// Returns the standard uptime monitor with all monitored services.
///
/// `request_json` shape: `{ }` (empty JSON object)
/// Returns a `UptimeMonitor` as JSON — initialized with the six standard
/// services (auth, firestore, cloud_storage, cloud_functions, ai_backend,
/// push_notifications), all in `Unknown` status.
#[no_mangle]
pub extern "C" fn stride_monitoring_uptime(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let services = monitoring::build_monitored_services();
        let mut monitor = UptimeMonitor::new();
        for svc in services {
            monitor.add_service(svc);
        }
        monitor.recompute();
        ok_json(&monitor)
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// §17 — Backups and disaster recovery
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the default backup schedule configuration.
///
/// `request_json` shape: `{ }` (empty JSON object)
/// Returns a `BackupConfig` as JSON — the standard default schedule
/// (weekly, Sunday 02:00 UTC, 30-day retention, all collections, includes
/// Cloud Storage, bucket `stride-backups`, region `us-central1`).
#[no_mangle]
pub extern "C" fn stride_backup_get_config(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let config = backups::build_default_backup_schedule();
        ok_json(&config)
    })
}

/// Builds a backup manifest from a list of backup records.
///
/// `request_json` shape:
/// ```json
/// {
///   "config": { … BackupConfig … },
///   "generated_at_ms": 1700000000000,
///   "records": [ … BackupRecord … ]
/// }
/// ```
/// Returns a `BackupManifest` as JSON, assembled from the given records and
/// config, with recomputed aggregate fields (total size, success/fail counts).
#[no_mangle]
pub extern "C" fn stride_backup_manifest(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            config: BackupConfig,
            generated_at_ms: i64,
            records: Vec<BackupRecord>,
        }

        impl Default for Req {
            fn default() -> Self {
                Self {
                    config: backups::build_default_backup_schedule(),
                    generated_at_ms: 0,
                    records: Vec::new(),
                }
            }
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mut manifest = BackupManifest::new(req.config, req.generated_at_ms);
        for record in req.records {
            manifest.add_record(record);
        }
        ok_json(&manifest)
    })
}

/// Executes a restore operation from a restore request.
///
/// `request_json` shape:
/// ```json
/// {
///   "request": { … RestoreRequest … },
///   "completed_at_ms": 1700000005000,
///   "documents_restored": 5000,
///   "files_restored": 120,
///   "size_restored_bytes": 536870912,
///   "validation_passed": true
/// }
/// ```
/// Returns a `RestoreResult` as JSON — the restore is marked `Completed`
/// with the given counts and validation flag.
#[no_mangle]
pub extern "C" fn stride_backup_restore(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            request: RestoreRequest,
            completed_at_ms: i64,
            documents_restored: u64,
            files_restored: u64,
            size_restored_bytes: u64,
            validation_passed: bool,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = RestoreResult::new(req.request).complete(
            req.completed_at_ms,
            req.documents_restored,
            req.files_restored,
            req.size_restored_bytes,
            req.validation_passed,
        );
        ok_json(&result)
    })
}

/// Runs a migration plan, executing all steps and returning the completed plan.
///
/// `request_json` shape:
/// ```json
/// {
///   "created_at_ms": 1700000000000,
///   "executed_at_ms": 1700000005000
/// }
/// ```
/// Returns a `MigrationPlan` as JSON — the default v1→v2 migration plan,
/// with all steps executed (marked `Completed`) and the plan itself marked
/// `Completed` at the given timestamp.
#[no_mangle]
pub extern "C" fn stride_backup_migrate(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            created_at_ms: i64,
            executed_at_ms: i64,
        }

        impl Default for Req {
            fn default() -> Self {
                Self {
                    created_at_ms: 0,
                    executed_at_ms: 0,
                }
            }
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mut plan = backups::build_default_migration_plan(req.created_at_ms);
        let steps: Vec<MigrationStep> = plan
            .steps
            .iter()
            .map(|s| {
                let mut clone = s.clone();
                clone = clone.execute(req.executed_at_ms);
                clone
            })
            .collect();
        plan.steps = steps;
        plan = plan.complete(req.executed_at_ms);
        ok_json(&plan)
    })
}

/// Creates a user data export result from an export request.
///
/// `request_json` shape:
/// ```json
/// {
///   "request": { … ExportRequest … },
///   "completed_at_ms": 1700000005000,
///   "size_bytes": 10485760,
///   "workout_count": 42,
///   "route_count": 42,
///   "download_url": "https://storage.googleapis.com/stride-exports/..."
/// }
/// ```
/// Returns an `ExportResult` as JSON — the export is marked `Completed`
/// with the given size, counts, and download URL.
#[no_mangle]
pub extern "C" fn stride_backup_export(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        struct Req {
            request: ExportRequest,
            completed_at_ms: i64,
            size_bytes: u64,
            workout_count: u32,
            route_count: u32,
            download_url: String,
        }

        let req: Req = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let result = ExportResult::new(req.request).complete(
            req.completed_at_ms,
            req.size_bytes,
            req.workout_count,
            req.route_count,
            req.download_url,
        );
        ok_json(&result)
    })
}


// ═══════════════════════════════════════════════════════════════════════
// §18 — Privacy, legal, and safety
// ═══════════════════════════════════════════════════════════════════════

/// Returns the default privacy policy, terms of service, health
/// disclaimer, and retention policy as a combined JSON object.
///
/// `request_json` shape:
/// ```json
/// { "last_updated_ms": 1700000000000 }
/// ```
#[no_mangle]
pub extern "C" fn stride_privacy_get_policy(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            last_updated_ms: i64,
        }

        impl Default for Req {
            fn default() -> Self {
                Self { last_updated_ms: 0 }
            }
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let policy = privacy::build_default_privacy_policy(req.last_updated_ms);
        let tos = privacy::build_default_terms_of_service(req.last_updated_ms);
        let health = HealthDisclaimer::new();
        let retention = privacy::build_default_privacy_retention_policy(req.last_updated_ms);

        let combined = json!({
            "privacy_policy": policy,
            "terms_of_service": tos,
            "health_disclaimer": health,
            "retention_policy": retention,
        });
        ok_json(&combined)
    })
}

/// Returns the default consent registry — a record of every consent
/// type the app tracks, with default (not-yet-granted) status.
///
/// `request_json` shape:
/// ```json
/// { "last_updated_ms": 1700000000000 }
/// ```
#[no_mangle]
pub extern "C" fn stride_privacy_consent(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            last_updated_ms: i64,
        }

        impl Default for Req {
            fn default() -> Self {
                Self { last_updated_ms: 0 }
            }
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let registry = privacy::build_default_consent_registry(req.last_updated_ms);
        ok_json(&registry)
    })
}

/// Returns all data disclosures (location, wearable, AI, music,
/// analytics, advertising).
#[no_mangle]
pub extern "C" fn stride_privacy_disclosures(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let disclosures = privacy::build_all_disclosures();
        ok_json(&disclosures)
    })
}

/// Computes and returns the privacy compliance status by checking all
/// the privacy components.
///
/// `request_json` shape:
/// ```json
/// { "last_updated_ms": 1700000000000, "health_acknowledged": true }
/// ```
#[no_mangle]
pub extern "C" fn stride_privacy_compliance(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            last_updated_ms: i64,
            health_acknowledged: bool,
        }

        impl Default for Req {
            fn default() -> Self {
                Self {
                    last_updated_ms: 0,
                    health_acknowledged: false,
                }
            }
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let mut health = HealthDisclaimer::new();
        if req.health_acknowledged {
            health = health.acknowledge(req.last_updated_ms);
        }
        let disclosures = privacy::build_all_disclosures();
        let retention = privacy::build_default_privacy_retention_policy(req.last_updated_ms);
        let consent = privacy::build_default_consent_registry(req.last_updated_ms);
        let osm = OpenStreetMapAttribution::new();
        let sdks = privacy::build_all_sdk_disclosures();
        let form = privacy::build_default_data_safety_form(req.last_updated_ms);

        let status = privacy::build_privacy_compliance_status(
            &health, &disclosures, &retention, &consent, &osm, &sdks, &form,
        );
        ok_json(&status)
    })
}

/// Returns the Google Play Data Safety form — the matrix of data
/// types × purposes × sharing status.
///
/// `request_json` shape:
/// ```json
/// { "last_updated_ms": 1700000000000 }
/// ```
#[no_mangle]
pub extern "C" fn stride_privacy_data_safety(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            last_updated_ms: i64,
        }

        impl Default for Req {
            fn default() -> Self {
                Self { last_updated_ms: 0 }
            }
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let form = privacy::build_default_data_safety_form(req.last_updated_ms);
        ok_json(&form)
    })
}


// ═══════════════════════════════════════════════════════════════════════════
// §19 — Store / Release readiness
// ═══════════════════════════════════════════════════════════════════════════

/// Returns the app identity — the immutable store-level identity
/// (app name, package ID, version, SDK levels).
///
/// `request_json` shape:
/// ```json
/// {}
/// ```
#[no_mangle]
pub extern "C" fn stride_release_identity(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let identity = release::AppIdentity::new();
        ok_json(&identity)
    })
}

/// Computes and returns the release readiness status from all
/// components (identity, signing, bundle, versioning, assets,
/// listing, permissions, reviewer, stack, data safety).
///
/// `request_json` shape:
/// ```json
/// { "data_safety_submitted": true }
/// ```
#[no_mangle]
pub extern "C" fn stride_release_readiness(
    request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let raw = match unsafe { read_str(request_json) } {
            Ok(s) => s,
            Err(e) => return err_json(e),
        };

        #[derive(serde::Deserialize)]
        #[serde(default)]
        struct Req {
            data_safety_submitted: bool,
        }

        impl Default for Req {
            fn default() -> Self {
                Self { data_safety_submitted: false }
            }
        }

        let req: Req = match serde_json::from_str(raw) {
            Ok(r) => r,
            Err(e) => return err_json(format!("invalid_request: {e}")),
        };

        let identity = release::AppIdentity::new();
        let signing = release::SigningKeyConfig::new();
        let bundle = release::ReleaseBundle::new();
        let versioning = release::VersioningPolicy::new();
        let assets = release::StoreAssets::new();
        let listing = release::StoreListing::new();
        let permissions = release::PermissionDeclarations::new();
        let reviewer = release::ReviewerAccess::new();
        let stack = release::build_default_production_stack();

        let status = release::build_release_readiness_status(
            &identity, &signing, &bundle, &versioning,
            &assets, &listing, &permissions, &reviewer,
            &stack, req.data_safety_submitted,
        );
        ok_json(&status)
    })
}

/// Returns the permission declarations for the app, including the
/// background-location justification text required by Google Play.
///
/// `request_json` shape:
/// ```json
/// {}
/// ```
#[no_mangle]
pub extern "C" fn stride_release_permissions(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let permissions = release::PermissionDeclarations::new();
        ok_json(&permissions)
    })
}

/// Returns the staged rollout plan — the internal, closed, and
/// production testing track configuration with rollout percentages.
///
/// `request_json` shape:
/// ```json
/// {}
/// ```
#[no_mangle]
pub extern "C" fn stride_release_rollout(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let plan = release::StagedRolloutPlan::new();
        ok_json(&plan)
    })
}

/// Returns the production stack inventory — the list of backend
/// services, SDKs, and infrastructure components used by the app.
///
/// `request_json` shape:
/// ```json
/// {}
/// ```
#[no_mangle]
pub extern "C" fn stride_release_stack(
    _request_json: *const c_char,
) -> *mut c_char {
    guarded(move || {
        let stack = release::build_default_production_stack();
        ok_json(&stack)
    })
}
