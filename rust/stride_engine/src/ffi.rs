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
use crate::engine::units::{self, ConversionKind, DistanceUnit};
use crate::engine::validation::{self, ValidationInput};
use crate::models::{
    ActivityType, SensorSource, WorkoutCheckpoint, WorkoutGoal, WorkoutPoint, WorkoutSummary,
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
