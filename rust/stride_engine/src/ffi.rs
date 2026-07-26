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
use crate::engine::wearable::{
    self, FallbackDecision, HealthConnectConsentState, MetricType,
    SourceAvailability, SourceRevocationRecord, WearableConnectionState,
    WearableDeviceInfo, WearableStatus, WearableSyncConfig, WearableSyncStatus,
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
