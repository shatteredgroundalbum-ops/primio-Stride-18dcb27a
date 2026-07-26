import '../models/walking_session.dart';
import 'stride_engine_bindings.dart';
import 'stride_models.dart';

/// High-level, typed Dart facade over the raw [StrideEngineBindings] JSON
/// calls. This is what the rest of the Flutter app (services, providers,
/// screens) should use — nothing outside `lib/native/` should touch
/// `StrideEngineBindings` or raw JSON maps directly.
///
/// One [StrideEngineClient] instance corresponds to one native
/// `WorkoutSessionController` (identified by [handle]) for the lifetime of
/// a single workout. Create a new client per workout via
/// [StrideEngineClient.create]; call [dispose] once the workout has been
/// persisted/discarded to free the native-side memory.
class StrideEngineClient {
  StrideEngineClient._(this._bindings, this.handle, this.workoutId);

  final StrideEngineBindings _bindings;
  final int handle;
  final String workoutId;

  static final StrideEngineBindings _shared = StrideEngineBindings();

  /// Returns the loaded native crate's version string — useful as a
  /// startup sanity check that the `.so`/static lib linked correctly.
  static String engineVersion() => _shared.version();

  /// Creates a new native workout session and returns a client bound to
  /// it. [activityType] must be one of `"walk"`, `"run"`, `"hike"`,
  /// `"auto_detect"`.
  static StrideEngineClient create({
    required String userId,
    required SessionType activityType,
    required double weightKg,
    required DateTime startedAt,
  }) {
    final bindings = _shared;
    final response = bindings.createSession({
      'user_id': userId,
      'activity_type': _activityTypeToJson(activityType),
      'weight_kg': weightKg,
      'started_at': startedAt.millisecondsSinceEpoch,
    });
    final data = unwrapEnvelope(response) as Map<String, dynamic>;
    return StrideEngineClient._(
      bindings,
      data['handle'] as int,
      data['workout_id'] as String,
    );
  }

  static String _activityTypeToJson(SessionType type) {
    switch (type) {
      case SessionType.walk:
        return 'walk';
      case SessionType.run:
        return 'run';
      case SessionType.hike:
        return 'hike';
    }
  }

  /// Pure decision logic (no handle/controller involved) — call this at
  /// app startup after loading the most recent row from the
  /// `workout_checkpoints` SQLite table (see `RecoveryService`) to decide
  /// whether to offer resume/finish/discard.
  static StrideRecoveryDecision evaluateRecovery({
    required Map<String, dynamic> checkpointJson,
    required DateTime now,
  }) {
    final env = _shared.evaluateRecovery(
        checkpointJson, now.millisecondsSinceEpoch);
    return StrideRecoveryDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Rebuilds a live native controller from a previously-persisted
  /// checkpoint (crash/kill/restart recovery — spec section 23). The
  /// restored session always starts in the `Paused` state; call [resume]
  /// once the caller is ready to continue live tracking.
  static StrideEngineClient restore({
    required Map<String, dynamic> checkpointJson,
    required String userId,
    required SessionType activityType,
    required double weightKg,
  }) {
    final bindings = _shared;
    final response = bindings.restoreSession({
      'checkpoint': checkpointJson,
      'user_id': userId,
      'activity_type': _activityTypeToJson(activityType),
      'weight_kg': weightKg,
    });
    final data = unwrapEnvelope(response) as Map<String, dynamic>;
    return StrideEngineClient._(
      bindings,
      data['handle'] as int,
      data['workout_id'] as String,
    );
  }

  StrideCoachingEvent start() {
    final env = _bindings.start(handle);
    return StrideCoachingEvent.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  void pause(DateTime now) {
    unwrapEnvelope(_bindings.pause(handle, now.millisecondsSinceEpoch));
  }

  void resume() {
    unwrapEnvelope(_bindings.resume(handle));
  }

  void discard() {
    unwrapEnvelope(_bindings.discard(handle));
  }

  void setDistanceGoalMeters(double meters) {
    unwrapEnvelope(_bindings.setGoal(handle, {
      'goal_type': 'distance',
      'target_value': meters,
    }));
  }

  void setDurationGoalMs(int ms) {
    unwrapEnvelope(_bindings.setGoal(handle, {
      'goal_type': 'duration',
      'target_value': ms.toDouble(),
    }));
  }

  /// Feeds one raw GPS fix into the engine. Returns the full live snapshot
  /// (updated distance/pace/speed/splits/coaching events/goal progress)
  /// for the UI to render.
  StrideLiveUpdate addLocationSample({
    required double latitude,
    required double longitude,
    double? altitudeMeters,
    double? accuracyMeters,
    double? speedMetersPerSecond,
    double? bearingDegrees,
    required DateTime recordedAt,
    String source = 'phone_gps',
    bool isMockLocation = false,
    required DateTime now,
  }) {
    final env = _bindings.addLocationSample(
      handle,
      {
        'point_id': 'dart-${recordedAt.microsecondsSinceEpoch}',
        'workout_id': workoutId,
        'latitude': latitude,
        'longitude': longitude,
        'altitude_meters': altitudeMeters,
        'accuracy_meters': accuracyMeters,
        'altitude_accuracy_meters': null,
        'speed_meters_per_second': speedMetersPerSecond,
        'bearing_degrees': bearingDegrees,
        'recorded_at': recordedAt.millisecondsSinceEpoch,
        'source': source,
        'is_mock_location': isMockLocation,
        'accepted': false,
        'rejection_reason': null,
      },
      now.millisecondsSinceEpoch,
    );
    return StrideLiveUpdate.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  void addHeartRateSample({required DateTime at, required int bpm}) {
    unwrapEnvelope(
        _bindings.addHeartRateSample(handle, at.millisecondsSinceEpoch, bpm));
  }

  void addStepDelta({
    required DateTime at,
    required int delta,
    String source = 'phone_step_sensor',
  }) {
    unwrapEnvelope(_bindings.addStepDelta(
        handle, at.millisecondsSinceEpoch, delta, source));
  }

  /// Periodic (~1/sec) advance with no new GPS point. Call this from a
  /// `Timer.periodic` while the workout is active/paused.
  StrideLiveUpdate tick(DateTime now) {
    final env = _bindings.tick(handle, now.millisecondsSinceEpoch);
    return StrideLiveUpdate.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  StrideWorkoutSplit manualLap(DateTime now) {
    final env = _bindings.manualLap(handle, now.millisecondsSinceEpoch);
    return StrideWorkoutSplit.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds a durable crash-recovery snapshot for local persistence
  /// (SQLite, via the existing `LocalDatabase`).
  Map<String, dynamic> buildCheckpoint(DateTime now) {
    final env = _bindings.buildCheckpoint(handle, now.millisecondsSinceEpoch);
    return unwrapEnvelope(env) as Map<String, dynamic>;
  }

  StrideWorkoutSummary finish(DateTime now) {
    final env = _bindings.finish(handle, now.millisecondsSinceEpoch);
    return StrideWorkoutSummary.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  StrideWorkoutSession getSession() {
    final env = _bindings.getSession(handle);
    return StrideWorkoutSession.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Supplies prior personal bests (loaded from local/cloud workout
  /// history) so the live controller can emit a "personal record
  /// possible" coaching nudge while the workout is still in progress.
  /// Optional; call right after [create]/[restore] if history is
  /// available. Safe to omit \u2014 post-finish [StrideEngineClient.
  /// detectPersonalRecords] still works without this ever being called.
  void setPriorBests(StridePriorBests priorBests) {
    unwrapEnvelope(_bindings.setPriorBests(handle, priorBests.toJson()));
  }

  /// Frees the native controller. Must be called exactly once, after the
  /// workout has been finished/discarded and no further calls with this
  /// [handle] will be made.
  void dispose() {
    unwrapEnvelope(_bindings.destroySession(handle));
  }

  // \u2500\u2500\u2500 Stateless decision-logic helpers \u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500
  // These do not require a handle/session \u2014 they wrap pure Rust decision
  // logic (spec sections 26, 28, 29, 30, 32, 34) and may be called as
  // static-style helpers from anywhere in the app (permission screens,
  // post-workout summary screens, settings, etc.).

  /// Re-runs full workout validation (spec section 26). Dart should call
  /// this after [finish], filling in [isDuplicateOfExistingWorkout] from
  /// its own local-history lookup, and OR the resulting `isBlocked` with
  /// the one [finish] already returned via [StrideWorkoutSummary.isBlocked].
  static StrideValidationResult validate({
    required DateTime startedAt,
    required DateTime endedAt,
    required double distanceMeters,
    required int durationMs,
    required double averageSpeedMps,
    required double maxSpeedMps,
    required double caloriesEstimated,
    required int routePointCount,
    required bool hasVehicleFlaggedSegment,
    required int gpsGapCount,
    required bool isDuplicateOfExistingWorkout,
  }) {
    final env = _shared.validate({
      'started_at': startedAt.millisecondsSinceEpoch,
      'ended_at': endedAt.millisecondsSinceEpoch,
      'distance_meters': distanceMeters,
      'duration_ms': durationMs,
      'average_speed_mps': averageSpeedMps,
      'max_speed_mps': maxSpeedMps,
      'calories_estimated': caloriesEstimated,
      'route_point_count': routePointCount,
      'has_vehicle_flagged_segment': hasVehicleFlaggedSegment,
      'gps_gap_count': gpsGapCount,
      'is_duplicate_of_existing_workout': isDuplicateOfExistingWorkout,
    });
    return StrideValidationResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Detects personal records (spec section 28) for a finalized
  /// [summary] against [priorBests] loaded from local/cloud history.
  static List<StridePersonalRecord> detectPersonalRecords({
    required StrideWorkoutSummary summary,
    required StridePriorBests priorBests,
  }) {
    final env = _shared.detectPersonalRecords({
      'summary': summary.toJson(),
      'prior_bests': priorBests.toJson(),
    });
    final data = unwrapEnvelope(env) as List;
    return data
        .map((e) => StridePersonalRecord.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  /// Permissions decision logic (spec section 30) \u2014 given the current
  /// permission [state] and [context], returns the action the app should
  /// take next (e.g. request directly, show rationale, open settings).
  static StrideRequiredAction requiredPermissionAction({
    required StridePermissionState state,
    required StridePermissionContext context,
  }) {
    final env = _shared.requiredPermissionAction({
      'state': _permissionStateToJson(state),
      'context': _permissionContextToJson(context),
    });
    return strideRequiredActionFromJson(
        unwrapEnvelope(env) as String);
  }

  /// Decides whether a workout can start full-featured, foreground-only,
  /// or blocked, given the current precise/background location
  /// permission states.
  static StrideWorkoutStartCapability evaluateStartCapability({
    required StridePermissionState preciseLocation,
    required StridePermissionState backgroundLocation,
  }) {
    final env = _shared.evaluateStartCapability({
      'precise_location': _permissionStateToJson(preciseLocation),
      'background_location': _permissionStateToJson(backgroundLocation),
    });
    return strideWorkoutStartCapabilityFromJson(
        unwrapEnvelope(env) as String);
  }

  /// Battery/sampling manager (spec section 32). Call whenever activity
  /// level or battery state changes to get an updated sampling profile,
  /// then reconfigure the GPS/sensor listeners accordingly.
  static StrideSamplingProfile chooseSamplingProfile({
    required StrideWorkoutActivityLevel activity,
    required int batteryPercent,
    required bool batterySaverEnabled,
    required bool isCharging,
  }) {
    final env = _shared.chooseSamplingProfile({
      'activity': _activityLevelToJson(activity),
      'battery': {
        'battery_percent': batteryPercent,
        'battery_saver_enabled': batterySaverEnabled,
        'is_charging': isCharging,
      },
    });
    return StrideSamplingProfile.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Unit system (spec section 34): applies a single named conversion.
  static double convertUnit(StrideConversionKind kind, double value) {
    final env = _shared.convertUnit({
      'kind': _conversionKindToJson(kind),
      'value': value,
    });
    return (unwrapEnvelope(env) as num).toDouble();
  }

  /// Formats a canonical pace (seconds per km) as "M:SS" for the
  /// requested display unit.
  static String formatPace(double secPerKm, {required bool useMiles}) {
    final env = _shared.formatPace({
      'sec_per_km': secPerKm,
      'unit': useMiles ? 'miles' : 'kilometers',
    });
    return unwrapEnvelope(env) as String;
  }

  /// Achievement/badge engine (spec section 29). Dart owns loading
  /// [history] beforehand and persisting any newly-returned awards
  /// (merging them into `history.alreadyAwarded`) afterward so they are
  /// never re-issued.
  static List<StrideAchievementEvent> detectAchievements({
    required StrideWorkoutSummary summary,
    required StrideAchievementHistory history,
    bool weeklyGoalCompletedByThisWorkout = false,
    bool newPaceRecordSet = false,
  }) {
    final env = _shared.detectAchievements({
      'summary': summary.toJson(),
      'history': history.toJson(),
      'weekly_goal_completed_by_this_workout':
          weeklyGoalCompletedByThisWorkout,
      'new_pace_record_set': newPaceRecordSet,
    });
    final data = unwrapEnvelope(env) as List;
    return data
        .map(
            (e) => StrideAchievementEvent.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  static String _permissionStateToJson(StridePermissionState s) {
    switch (s) {
      case StridePermissionState.notRequested:
        return 'not_requested';
      case StridePermissionState.granted:
        return 'granted';
      case StridePermissionState.denied:
        return 'denied';
      case StridePermissionState.permanentlyDenied:
        return 'permanently_denied';
      case StridePermissionState.revokedDuringUse:
        return 'revoked_during_use';
    }
  }

  static String _permissionContextToJson(StridePermissionContext c) {
    switch (c) {
      case StridePermissionContext.initial:
        return 'initial';
      case StridePermissionContext.deniedOnce:
        return 'denied_once';
      case StridePermissionContext.duringActiveWorkout:
        return 'during_active_workout';
    }
  }

  static String _activityLevelToJson(StrideWorkoutActivityLevel a) {
    switch (a) {
      case StrideWorkoutActivityLevel.active:
        return 'active';
      case StrideWorkoutActivityLevel.paused:
        return 'paused';
      case StrideWorkoutActivityLevel.autoPaused:
        return 'auto_paused';
    }
  }

  static String _conversionKindToJson(StrideConversionKind k) {
    switch (k) {
      case StrideConversionKind.metersToMiles:
        return 'meters_to_miles';
      case StrideConversionKind.milesToMeters:
        return 'miles_to_meters';
      case StrideConversionKind.metersToFeet:
        return 'meters_to_feet';
      case StrideConversionKind.feetToMeters:
        return 'feet_to_meters';
      case StrideConversionKind.kgToLb:
        return 'kg_to_lb';
      case StrideConversionKind.lbToKg:
        return 'lb_to_kg';
      case StrideConversionKind.mpsToKmh:
        return 'mps_to_kmh';
      case StrideConversionKind.kmhToMps:
        return 'kmh_to_mps';
      case StrideConversionKind.mpsToMph:
        return 'mps_to_mph';
      case StrideConversionKind.mphToMps:
        return 'mph_to_mps';
      case StrideConversionKind.paceSecPerKmToSecPerMile:
        return 'pace_sec_per_km_to_sec_per_mile';
      case StrideConversionKind.paceSecPerMileToSecPerKm:
        return 'pace_sec_per_mile_to_sec_per_km';
    }
  }
}
