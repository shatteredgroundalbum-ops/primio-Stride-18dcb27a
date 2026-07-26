import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'stride_native_library.dart';

// ─── Native C function signatures ───────────────────────────────────────────
// These mirror the `extern "C"` functions in
// `rust/stride_engine/src/ffi.rs` one-to-one. Every function that returns
// `*mut c_char` hands Dart an owned, heap-allocated JSON string that MUST
// be freed via `stride_free_string` exactly once.

typedef _StrideEngineVersionNative = Pointer<Utf8> Function();
typedef _StrideEngineVersionDart = Pointer<Utf8> Function();

typedef _StrideCreateSessionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCreateSessionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDestroySessionNative = Pointer<Utf8> Function(Int64);
typedef _StrideDestroySessionDart = Pointer<Utf8> Function(int);

typedef _StrideStartNative = Pointer<Utf8> Function(Int64);
typedef _StrideStartDart = Pointer<Utf8> Function(int);

typedef _StridePauseNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StridePauseDart = Pointer<Utf8> Function(int, int);

typedef _StrideResumeNative = Pointer<Utf8> Function(Int64);
typedef _StrideResumeDart = Pointer<Utf8> Function(int);

typedef _StrideDiscardNative = Pointer<Utf8> Function(Int64);
typedef _StrideDiscardDart = Pointer<Utf8> Function(int);

typedef _StrideSetGoalNative = Pointer<Utf8> Function(Int64, Pointer<Utf8>);
typedef _StrideSetGoalDart = Pointer<Utf8> Function(int, Pointer<Utf8>);

typedef _StrideAddLocationSampleNative = Pointer<Utf8> Function(
    Int64, Pointer<Utf8>, Int64);
typedef _StrideAddLocationSampleDart = Pointer<Utf8> Function(
    int, Pointer<Utf8>, int);

typedef _StrideAddHeartRateSampleNative = Pointer<Utf8> Function(
    Int64, Int64, Uint16);
typedef _StrideAddHeartRateSampleDart = Pointer<Utf8> Function(
    int, int, int);

typedef _StrideAddStepDeltaNative = Pointer<Utf8> Function(
    Int64, Int64, Uint32, Pointer<Utf8>);
typedef _StrideAddStepDeltaDart = Pointer<Utf8> Function(
    int, int, int, Pointer<Utf8>);

typedef _StrideTickNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideTickDart = Pointer<Utf8> Function(int, int);

typedef _StrideManualLapNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideManualLapDart = Pointer<Utf8> Function(int, int);

typedef _StrideBuildCheckpointNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideBuildCheckpointDart = Pointer<Utf8> Function(int, int);

typedef _StrideFinishNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideFinishDart = Pointer<Utf8> Function(int, int);

typedef _StrideGetSessionNative = Pointer<Utf8> Function(Int64);
typedef _StrideGetSessionDart = Pointer<Utf8> Function(int);

typedef _StrideEvaluateRecoveryNative = Pointer<Utf8> Function(
    Pointer<Utf8>, Int64);
typedef _StrideEvaluateRecoveryDart = Pointer<Utf8> Function(
    Pointer<Utf8>, int);

typedef _StrideRestoreSessionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideRestoreSessionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideSetPriorBestsNative = Pointer<Utf8> Function(
    Int64, Pointer<Utf8>);
typedef _StrideSetPriorBestsDart = Pointer<Utf8> Function(
    int, Pointer<Utf8>);

// Stateless decision-logic entry points (no session handle required).
typedef _StrideValidateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideValidateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDetectPersonalRecordsNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideDetectPersonalRecordsDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideRequiredPermissionActionNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideRequiredPermissionActionDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideEvaluateStartCapabilityNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideEvaluateStartCapabilityDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideChooseSamplingProfileNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideChooseSamplingProfileDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideConvertUnitNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideConvertUnitDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideFormatPaceNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideFormatPaceDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDetectAchievementsNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideDetectAchievementsDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideFreeStringNative = Void Function(Pointer<Utf8>);
typedef _StrideFreeStringDart = void Function(Pointer<Utf8>);

/// Thin, allocation-safe wrapper around the raw native symbols. All public
/// methods return decoded `Map<String, dynamic>` (the `{"ok":..,"data"/
/// "error":..}` envelope) so callers never touch native memory directly —
/// every native string is read and freed inside this class.
class StrideEngineBindings {
  StrideEngineBindings._(DynamicLibrary lib) : _lib = lib {
    _version = _lib
        .lookupFunction<_StrideEngineVersionNative, _StrideEngineVersionDart>(
            'stride_engine_version');
    _createSession = _lib.lookupFunction<_StrideCreateSessionNative,
        _StrideCreateSessionDart>('stride_create_session');
    _destroySession = _lib.lookupFunction<_StrideDestroySessionNative,
        _StrideDestroySessionDart>('stride_destroy_session');
    _start =
        _lib.lookupFunction<_StrideStartNative, _StrideStartDart>('stride_start');
    _pause =
        _lib.lookupFunction<_StridePauseNative, _StridePauseDart>('stride_pause');
    _resume = _lib
        .lookupFunction<_StrideResumeNative, _StrideResumeDart>('stride_resume');
    _discard = _lib.lookupFunction<_StrideDiscardNative, _StrideDiscardDart>(
        'stride_discard');
    _setGoal = _lib.lookupFunction<_StrideSetGoalNative, _StrideSetGoalDart>(
        'stride_set_goal');
    _addLocationSample = _lib.lookupFunction<_StrideAddLocationSampleNative,
        _StrideAddLocationSampleDart>('stride_add_location_sample');
    _addHeartRateSample = _lib.lookupFunction<_StrideAddHeartRateSampleNative,
        _StrideAddHeartRateSampleDart>('stride_add_heart_rate_sample');
    _addStepDelta = _lib.lookupFunction<_StrideAddStepDeltaNative,
        _StrideAddStepDeltaDart>('stride_add_step_delta');
    _tick =
        _lib.lookupFunction<_StrideTickNative, _StrideTickDart>('stride_tick');
    _manualLap = _lib.lookupFunction<_StrideManualLapNative,
        _StrideManualLapDart>('stride_manual_lap');
    _buildCheckpoint = _lib.lookupFunction<_StrideBuildCheckpointNative,
        _StrideBuildCheckpointDart>('stride_build_checkpoint');
    _finish = _lib
        .lookupFunction<_StrideFinishNative, _StrideFinishDart>('stride_finish');
    _getSession = _lib.lookupFunction<_StrideGetSessionNative,
        _StrideGetSessionDart>('stride_get_session');
    _evaluateRecovery = _lib.lookupFunction<_StrideEvaluateRecoveryNative,
        _StrideEvaluateRecoveryDart>('stride_evaluate_recovery');
    _restoreSession = _lib.lookupFunction<_StrideRestoreSessionNative,
        _StrideRestoreSessionDart>('stride_restore_session');
    _setPriorBests = _lib.lookupFunction<_StrideSetPriorBestsNative,
        _StrideSetPriorBestsDart>('stride_set_prior_bests');
    _validate = _lib.lookupFunction<_StrideValidateNative,
        _StrideValidateDart>('stride_validate');
    _detectPersonalRecords = _lib.lookupFunction<
        _StrideDetectPersonalRecordsNative,
        _StrideDetectPersonalRecordsDart>('stride_detect_personal_records');
    _requiredPermissionAction = _lib.lookupFunction<
        _StrideRequiredPermissionActionNative,
        _StrideRequiredPermissionActionDart>(
        'stride_required_permission_action');
    _evaluateStartCapability = _lib.lookupFunction<
        _StrideEvaluateStartCapabilityNative,
        _StrideEvaluateStartCapabilityDart>('stride_evaluate_start_capability');
    _chooseSamplingProfile = _lib.lookupFunction<
        _StrideChooseSamplingProfileNative,
        _StrideChooseSamplingProfileDart>('stride_choose_sampling_profile');
    _convertUnit = _lib.lookupFunction<_StrideConvertUnitNative,
        _StrideConvertUnitDart>('stride_convert_unit');
    _formatPace = _lib.lookupFunction<_StrideFormatPaceNative,
        _StrideFormatPaceDart>('stride_format_pace');
    _detectAchievements = _lib.lookupFunction<_StrideDetectAchievementsNative,
        _StrideDetectAchievementsDart>('stride_detect_achievements');
    _freeString = _lib.lookupFunction<_StrideFreeStringNative,
        _StrideFreeStringDart>('stride_free_string');
  }

  static StrideEngineBindings? _instance;

  /// Lazily loads and looks up all native symbols exactly once per process.
  factory StrideEngineBindings() {
    return _instance ??= StrideEngineBindings._(loadStrideEngineLibrary());
  }

  final DynamicLibrary _lib;

  late final _StrideEngineVersionDart _version;
  late final _StrideCreateSessionDart _createSession;
  late final _StrideDestroySessionDart _destroySession;
  late final _StrideStartDart _start;
  late final _StridePauseDart _pause;
  late final _StrideResumeDart _resume;
  late final _StrideDiscardDart _discard;
  late final _StrideSetGoalDart _setGoal;
  late final _StrideAddLocationSampleDart _addLocationSample;
  late final _StrideAddHeartRateSampleDart _addHeartRateSample;
  late final _StrideAddStepDeltaDart _addStepDelta;
  late final _StrideTickDart _tick;
  late final _StrideManualLapDart _manualLap;
  late final _StrideBuildCheckpointDart _buildCheckpoint;
  late final _StrideFinishDart _finish;
  late final _StrideGetSessionDart _getSession;
  late final _StrideEvaluateRecoveryDart _evaluateRecovery;
  late final _StrideRestoreSessionDart _restoreSession;
  late final _StrideSetPriorBestsDart _setPriorBests;
  late final _StrideValidateDart _validate;
  late final _StrideDetectPersonalRecordsDart _detectPersonalRecords;
  late final _StrideRequiredPermissionActionDart _requiredPermissionAction;
  late final _StrideEvaluateStartCapabilityDart _evaluateStartCapability;
  late final _StrideChooseSamplingProfileDart _chooseSamplingProfile;
  late final _StrideConvertUnitDart _convertUnit;
  late final _StrideFormatPaceDart _formatPace;
  late final _StrideDetectAchievementsDart _detectAchievements;
  late final _StrideFreeStringDart _freeString;

  /// Reads, decodes, and frees a native JSON string pointer.
  Map<String, dynamic> _consume(Pointer<Utf8> ptr) {
    try {
      final jsonStr = ptr.toDartString();
      final decoded = jsonDecode(jsonStr);
      return decoded as Map<String, dynamic>;
    } finally {
      _freeString(ptr);
    }
  }

  Pointer<Utf8> _toNative(String s) => s.toNativeUtf8();

  String version() {
    final env = _consume(_version());
    return env['data'] as String? ?? 'unknown';
  }

  Map<String, dynamic> createSession(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_createSession(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> destroySession(int handle) =>
      _consume(_destroySession(handle));

  Map<String, dynamic> start(int handle) => _consume(_start(handle));

  Map<String, dynamic> pause(int handle, int nowMs) =>
      _consume(_pause(handle, nowMs));

  Map<String, dynamic> resume(int handle) => _consume(_resume(handle));

  Map<String, dynamic> discard(int handle) => _consume(_discard(handle));

  Map<String, dynamic> setGoal(int handle, Map<String, dynamic> goal) {
    final ptr = _toNative(jsonEncode(goal));
    try {
      return _consume(_setGoal(handle, ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> addLocationSample(
    int handle,
    Map<String, dynamic> point,
    int nowMs,
  ) {
    final ptr = _toNative(jsonEncode(point));
    try {
      return _consume(_addLocationSample(handle, ptr, nowMs));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> addHeartRateSample(int handle, int atMs, int bpm) =>
      _consume(_addHeartRateSample(handle, atMs, bpm));

  Map<String, dynamic> addStepDelta(
    int handle,
    int atMs,
    int delta,
    String source,
  ) {
    final ptr = _toNative(source);
    try {
      return _consume(_addStepDelta(handle, atMs, delta, ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> tick(int handle, int nowMs) =>
      _consume(_tick(handle, nowMs));

  Map<String, dynamic> manualLap(int handle, int nowMs) =>
      _consume(_manualLap(handle, nowMs));

  Map<String, dynamic> buildCheckpoint(int handle, int nowMs) =>
      _consume(_buildCheckpoint(handle, nowMs));

  Map<String, dynamic> finish(int handle, int nowMs) =>
      _consume(_finish(handle, nowMs));

  Map<String, dynamic> getSession(int handle) => _consume(_getSession(handle));

  /// Pure decision logic: given a persisted checkpoint JSON and the current
  /// wall-clock time, asks the Rust engine whether the workout should be
  /// resumed, finished-and-saved, or discarded (see
  /// `engine::recovery::evaluate_checkpoint`). Does not touch the session
  /// registry — safe to call before any handle exists.
  Map<String, dynamic> evaluateRecovery(
    Map<String, dynamic> checkpoint,
    int nowMs,
  ) {
    final ptr = _toNative(jsonEncode(checkpoint));
    try {
      return _consume(_evaluateRecovery(ptr, nowMs));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Rebuilds a live `WorkoutSessionController` from a persisted checkpoint
  /// and registers it under a new handle, returned in the response payload
  /// as `{"handle": .., "workout_id": ..}`. The restored controller always
  /// starts in the `Paused` state (see `restore_from_checkpoint` doc
  /// comments in `engine/controller.rs`).
  Map<String, dynamic> restoreSession(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_restoreSession(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Supplies prior personal bests (loaded from local/cloud history) so
  /// the engine can emit a live "personal record possible" coaching nudge.
  /// Optional; call right after `createSession` if history is available.
  Map<String, dynamic> setPriorBests(
    int handle,
    Map<String, dynamic> priorBests,
  ) {
    final ptr = _toNative(jsonEncode(priorBests));
    try {
      return _consume(_setPriorBests(handle, ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Re-runs full workout validation (spec section 26). Dart should call
  /// this after `finish()`, filling in `is_duplicate_of_existing_workout`
  /// from its own history lookup, and OR the `is_blocked` result with the
  /// one `finish()` already returned.
  Map<String, dynamic> validate(Map<String, dynamic> validationInput) {
    final ptr = _toNative(jsonEncode(validationInput));
    try {
      return _consume(_validate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Detects personal records for a finalized `WorkoutSummary` against
  /// caller-supplied prior bests loaded from history.
  Map<String, dynamic> detectPersonalRecords(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_detectPersonalRecords(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Permissions decision logic (spec section 30). Pure/stateless.
  Map<String, dynamic> requiredPermissionAction(
    Map<String, dynamic> request,
  ) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_requiredPermissionAction(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides whether a workout can start full-featured, foreground-only,
  /// or blocked, given precise/background location permission states.
  Map<String, dynamic> evaluateStartCapability(
    Map<String, dynamic> request,
  ) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_evaluateStartCapability(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Battery/sampling manager (spec section 32). Call whenever activity
  /// level or battery state changes to get an updated `SamplingProfile`.
  Map<String, dynamic> chooseSamplingProfile(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_chooseSamplingProfile(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Unit system (spec section 34): applies a single named conversion.
  Map<String, dynamic> convertUnit(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_convertUnit(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Formats a canonical pace (sec/km) as "M:SS" for a display unit.
  Map<String, dynamic> formatPace(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_formatPace(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Achievement/badge engine (spec section 29). Dart owns loading history
  /// beforehand and persisting any newly-returned awards afterward.
  Map<String, dynamic> detectAchievements(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_detectAchievements(ptr));
    } finally {
      malloc.free(ptr);
    }
  }
}

/// Thrown when the native engine reports `{"ok": false, ...}` for a call
/// that the Dart layer expected to succeed unconditionally.
class StrideEngineException implements Exception {
  StrideEngineException(this.message);
  final String message;

  @override
  String toString() => 'StrideEngineException: $message';
}

/// Unwraps the `{"ok": bool, "data"/"error": ...}` envelope, throwing
/// [StrideEngineException] on failure and returning the `data` payload
/// (which may itself be `null`/any JSON value) on success.
dynamic unwrapEnvelope(Map<String, dynamic> envelope) {
  if (envelope['ok'] == true) {
    return envelope['data'];
  }
  throw StrideEngineException(envelope['error']?.toString() ?? 'unknown_error');
}
