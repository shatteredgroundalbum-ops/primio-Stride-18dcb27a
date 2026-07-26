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
use crate::engine::battery::{self, BatteryContext, WorkoutActivityLevel};
use crate::engine::controller::{SessionConfig, WorkoutSessionController};
use crate::engine::permissions::{self, PermissionContext, PermissionState};
use crate::engine::personal_records::{self, PriorBests};
use crate::engine::route_format::{
    self, RouteFileFormat, RouteFormatInput, RouteSummary,
    RouteFileSyncState,
};
use crate::engine::sync::{
    self, ConflictInfo, ConflictResolutionStrategy, DeviceSyncState,
};
use crate::engine::units::{self, ConversionKind, DistanceUnit};
use crate::engine::validation::{self, ValidationInput};
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
