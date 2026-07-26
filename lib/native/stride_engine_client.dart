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

  // ─── Cloud synchronization engine (spec section 3) ───────────────
  // All static — no session handle required.

  /// Computes the retry delay (ms) for a sync attempt using exponential
  /// backoff with jitter. Returns the delay in milliseconds.
  static int computeSyncBackoff({required int attempt, int jitterSeed = 0}) {
    final env = _bindings.computeSyncBackoff({
      'attempt': attempt,
      'jitter_seed': jitterSeed,
    });
    return unwrapEnvelope(env)['delay_ms'] as int;
  }

  /// Decides whether a failed sync attempt should be retried, and if so,
  /// after how long. Returns a [StrideSyncRetryDecision].
  static StrideSyncRetryDecision decideSyncRetry({
    required StrideSyncAttemptResult result,
    required int currentRetryCount,
    int jitterSeed = 0,
  }) {
    final env = _bindings.decideSyncRetry({
      'result': _syncAttemptResultToJson(result),
      'current_retry_count': currentRetryCount,
      'jitter_seed': jitterSeed,
    });
    return StrideSyncRetryDecision.fromJson(
        Map<String, dynamic>.from(unwrapEnvelope(env) as Map));
  }

  /// Resolves a sync conflict between local and cloud versions.
  /// Returns a [StrideConflictDecision].
  static StrideConflictDecision resolveSyncConflict({
    required String workoutId,
    required int localUpdatedAt,
    required int cloudUpdatedAt,
    String? cloudDeviceId,
    required String localDeviceId,
    StrideConflictResolutionStrategy strategy =
        StrideConflictResolutionStrategy.lastWriteWins,
  }) {
    final env = _bindings.resolveSyncConflict({
      'workout_id': workoutId,
      'local_updated_at': localUpdatedAt,
      'cloud_updated_at': cloudUpdatedAt,
      if (cloudDeviceId != null) 'cloud_device_id': cloudDeviceId,
      'local_device_id': localDeviceId,
      'strategy': _conflictStrategyToJson(strategy),
    });
    return _conflictDecisionFromJson(unwrapEnvelope(env)['decision'] as String);
  }

  /// Checks whether a local workout is a duplicate of an existing cloud
  /// workout. Returns true if the workout should be skipped (already
  /// synced with identical content).
  static bool detectSyncDuplicate({
    required String localWorkoutId,
    required int localUpdatedAt,
    String? cloudWorkoutId,
    int? cloudUpdatedAt,
  }) {
    final env = _bindings.detectSyncDuplicate({
      'local_workout_id': localWorkoutId,
      'local_updated_at': localUpdatedAt,
      if (cloudWorkoutId != null) 'cloud_workout_id': cloudWorkoutId,
      if (cloudUpdatedAt != null) 'cloud_updated_at': cloudUpdatedAt,
    });
    return unwrapEnvelope(env)['is_duplicate'] as bool;
  }

  /// Decides whether to insert, update, or skip a workout upsert.
  /// Returns a [StrideUpsertDecision].
  static StrideUpsertDecision decideSyncUpsert({
    required String localWorkoutId,
    required int localUpdatedAt,
    String? cloudWorkoutId,
    int? cloudUpdatedAt,
  }) {
    final env = _bindings.decideSyncUpsert({
      'local_workout_id': localWorkoutId,
      'local_updated_at': localUpdatedAt,
      if (cloudWorkoutId != null) 'cloud_workout_id': cloudWorkoutId,
      if (cloudUpdatedAt != null) 'cloud_updated_at': cloudUpdatedAt,
    });
    return _upsertDecisionFromJson(unwrapEnvelope(env)['decision'] as String);
  }

  /// Decides what action the current device should take for a workout in
  /// device-to-device sync. Returns a [StrideDeviceSyncAction].
  static StrideDeviceSyncAction decideDeviceSync({
    required String recordingDeviceId,
    required String currentDeviceId,
    required bool isUploaded,
    required bool isDownloaded,
  }) {
    final env = _bindings.decideDeviceSync({
      'recording_device_id': recordingDeviceId,
      'current_device_id': currentDeviceId,
      'is_uploaded': isUploaded,
      'is_downloaded': isDownloaded,
    });
    return _deviceSyncActionFromJson(unwrapEnvelope(env)['action'] as String);
  }

  /// Decides whether a deletion tombstone should be retried.
  static bool tombstoneShouldRetry({
    required StrideTombstoneState state,
    required int retryCount,
  }) {
    final env = _bindings.tombstoneShouldRetry({
      'state': _tombstoneStateToJson(state),
      'retry_count': retryCount,
    });
    return unwrapEnvelope(env)['should_retry'] as bool;
  }

  /// Decides whether a synced tombstone is old enough to be GC'd.
  static bool tombstoneShouldGc({
    required StrideTombstoneState state,
    required int syncedAt,
    required int nowMs,
  }) {
    final env = _bindings.tombstoneShouldGc({
      'state': _tombstoneStateToJson(state),
      'synced_at': syncedAt,
      'now_ms': nowMs,
    });
    return unwrapEnvelope(env)['should_gc'] as bool;
  }

  // ─── Route file format & storage layout (spec section 4) ────────

  /// Decides which route file format to use for a workout.
  ///
  /// Returns [StrideRouteFileFormat.gpx] when the user wants a GPX
  /// export, [StrideRouteFileFormat.polylineOnly] for short routes
  /// (<50 points), [StrideRouteFileFormat.compressedBinary] when
  /// offline and long, or [StrideRouteFileFormat.gpx] when online and
  /// long.
  static StrideRouteFileFormat decideRouteFormat({
    required int pointCount,
    required bool isOffline,
    required bool wantsGpxExport,
  }) {
    final env = _bindings.decideRouteFormat({
      'point_count': pointCount,
      'is_offline': isOffline,
      'wants_gpx_export': wantsGpxExport,
    });
    return _routeFileFormatFromJson(unwrapEnvelope(env)['format'] as String);
  }

  /// Generates the Cloud Storage object key for a route file.
  ///
  /// Returns `null` for [StrideRouteFileFormat.polylineOnly] (no file
  /// is uploaded in that case).
  static String? generateRouteFilePath({
    required String userId,
    required String workoutId,
    required StrideRouteFileFormat format,
  }) {
    final env = _bindings.generateRouteFilePath({
      'user_id': userId,
      'workout_id': workoutId,
      'format': _routeFileFormatToJson(format),
    });
    return unwrapEnvelope(env)['path'] as String?;
  }

  /// Estimates the byte size of a route file before it is serialized.
  ///
  /// Returns 0 for [StrideRouteFileFormat.polylineOnly].
  static int estimateRouteFileSize({
    required StrideRouteFileFormat format,
    required int pointCount,
  }) {
    final env = _bindings.estimateRouteFileSize({
      'format': _routeFileFormatToJson(format),
      'point_count': pointCount,
    });
    return unwrapEnvelope(env)['size_bytes'] as int;
  }

  /// Decides whether a route file upload should wait for Wi-Fi rather
  /// than proceeding over mobile data.
  ///
  /// Returns `true` (prefer Wi-Fi) when the estimated file size exceeds
  /// 512 KiB and the device is not on Wi-Fi.
  static bool shouldPreferWifiForUpload({
    required StrideRouteFileFormat format,
    required int pointCount,
    required bool isOnWifi,
  }) {
    final env = _bindings.shouldPreferWifiForUpload({
      'format': _routeFileFormatToJson(format),
      'point_count': pointCount,
      'is_on_wifi': isOnWifi,
    });
    return unwrapEnvelope(env)['should_prefer_wifi'] as bool;
  }

  /// Serializes a list of [StrideWorkoutPoint]s into a GPX 1.1 XML
  /// document.
  ///
  /// Only accepted points are emitted. The output is a complete, valid
  /// GPX file suitable for Cloud Storage upload or Strava/Garmin
  /// sharing.
  static String serializeGpx({
    required String workoutId,
    required int startedAtMs,
    required List<StrideWorkoutPoint> points,
  }) {
    final env = _bindings.serializeGpx({
      'workout_id': workoutId,
      'started_at_ms': startedAtMs,
      'points': points.map((p) => p.toJson()).toList(),
    });
    return unwrapEnvelope(env)['gpx'] as String;
  }

  /// Builds [StrideRouteFileMetadata] from controller inputs at
  /// workout-finish time.
  ///
  /// This is the pure decision logic; the actual file serialization and
  /// Cloud Storage upload happen on the Dart side (the engine never
  /// does I/O).
  static StrideRouteFileMetadata buildRouteFileMetadata({
    required String userId,
    required String workoutId,
    required List<StrideWorkoutPoint> points,
    required String deviceSource,
    required bool isOffline,
    required bool wantsGpxExport,
    required int finishedAt,
  }) {
    final env = _bindings.buildRouteFileMetadata({
      'user_id': userId,
      'workout_id': workoutId,
      'points': points.map((p) => p.toJson()).toList(),
      'device_source': deviceSource,
      'is_offline': isOffline,
      'wants_gpx_export': wantsGpxExport,
      'finished_at': finishedAt,
    });
    return StrideRouteFileMetadata.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Checks whether a route file sync state is terminal (no further
  /// automatic action will be taken by the sync loop).
  static bool isRouteSyncTerminal({required StrideRouteFileSyncState state}) {
    final env = _bindings.isRouteSyncTerminal({
      'state': _routeFileSyncStateToJson(state),
    });
    return unwrapEnvelope(env)['is_terminal'] as bool;
  }

  /// Validates that a [StrideRouteSummary] is safe to write to
  /// Firestore.
  ///
  /// Checks:
  ///   - `workoutId` and `userId` are non-empty.
  ///   - `routeFilePath` (if present) starts with `routes/{userId}/`.
  ///   - `encodedPolyline` (if present) is under 1 MiB.
  ///
  /// Returns `null` if valid, or an error message string if not.
  static String? validateRouteSummary({required StrideRouteSummary summary}) {
    final env = _bindings.validateRouteSummary(summary.toJson());
    final data = unwrapEnvelope(env) as Map<String, dynamic>;
    if (data['valid'] == true) return null;
    return data['error'] as String?;
  }

  /// Determines the dominant [StrideLocationSource] of a route — the
  /// source that contributed the most accepted points.
  ///
  /// Returns `null` if the route has no accepted points.
  static StrideLocationSource? dominantLocationSource({
    required List<StrideWorkoutPoint> points,
  }) {
    final env = _bindings.dominantLocationSource({
      'points': points.map((p) => p.toJson()).toList(),
    });
    final source = unwrapEnvelope(env)['source'] as String?;
    return _locationSourceFromJson(source);
  }

  /// Returns the human-readable label for a [StrideLocationSource],
  /// for UI display (e.g. "Phone GPS", "Wear OS", "Health Connect").
  static String locationSourceLabel({required StrideLocationSource source}) {
    final env = _bindings.locationSourceLabel({
      'source': _locationSourceToJson(source),
    });
    return unwrapEnvelope(env)['label'] as String;
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Maps and location services (spec section 5)
  // ─────────────────────────────────────────────────────────────────────────

  /// Determines the [StrideTileProvider] for a given [StrideMapViewType].
  static StrideTileProvider providerForView({required StrideMapViewType view}) {
    final env = _bindings.providerForView({'view': view.toJson()});
    return StrideTileProvider.fromJson(unwrapEnvelope(env)['provider'] as String);
  }

  /// Returns the attribution text for a [StrideTileProvider].
  ///
  /// This text must be displayed somewhere in the map UI (typically the
  /// bottom corner) to comply with the provider's licensing terms.
  static String attributionForProvider({required StrideTileProvider provider}) {
    final env = _bindings.attributionForProvider({'provider': provider.toJson()});
    return unwrapEnvelope(env)['attribution'] as String;
  }

  /// Returns the attribution text for a [StrideMapViewType] (convenience
  /// wrapper that resolves view → provider → attribution).
  static String attributionForView({required StrideMapViewType view}) {
    final env = _bindings.attributionForView({'view': view.toJson()});
    return unwrapEnvelope(env)['attribution'] as String;
  }

  /// Returns the tile URL template for a [StrideTileProvider].
  ///
  /// The template contains `{z}`, `{x}`, `{y}` placeholders that the
  /// Dart layer substitutes with actual tile coordinates.
  static String tileUrlTemplate({required StrideTileProvider provider}) {
    final env = _bindings.tileUrlTemplate({'provider': provider.toJson()});
    return unwrapEnvelope(env)['url_template'] as String;
  }

  /// Classifies a GPS accuracy value (in meters) into a quality level.
  ///
  /// Pass `null` for [accuracyMeters] when the GPS fix has no accuracy
  /// estimate (treated as unavailable).
  static StrideGpsAccuracyLevel classifyGpsAccuracy({double? accuracyMeters}) {
    final env = _bindings.classifyGpsAccuracy({'accuracy_meters': accuracyMeters});
    return StrideGpsAccuracyLevel.fromJson(unwrapEnvelope(env)['level'] as String);
  }

  /// Returns the human-readable description for a GPS accuracy level.
  static String gpsAccuracyDescription({required StrideGpsAccuracyLevel level}) {
    final env = _bindings.gpsAccuracyDescription({'level': level.toJson()});
    return unwrapEnvelope(env)['description'] as String;
  }

  /// Returns the hex color (e.g. `#4CAF50`) for the GPS accuracy dot.
  static String gpsAccuracyColor({required StrideGpsAccuracyLevel level}) {
    final env = _bindings.gpsAccuracyColor({'level': level.toJson()});
    return unwrapEnvelope(env)['color'] as String;
  }

  /// Converts a (lat, lon) pair to a tile coordinate at a zoom level.
  ///
  /// Returns `null` if the latitude is outside the valid Web Mercator
  /// range (±85.0511°).
  static StrideTileCoord? latLonToTile({
    required double lat,
    required double lon,
    required int zoom,
  }) {
    final env = _bindings.latLonToTile({
      'lat': lat,
      'lon': lon,
      'zoom': zoom,
    });
    final tile = unwrapEnvelope(env)['tile'];
    if (tile == null) return null;
    return StrideTileCoord.fromJson(tile as Map<String, dynamic>);
  }

  /// Counts the total number of tiles needed to cover a bounding box
  /// across a range of zoom levels. Used to estimate offline download
  /// size before starting a download.
  static int countTilesInRegion({
    required double minLat,
    required double minLon,
    required double maxLat,
    required double maxLon,
    required int minZoom,
    required int maxZoom,
  }) {
    final env = _bindings.countTilesInRegion({
      'min_lat': minLat,
      'min_lon': minLon,
      'max_lat': maxLat,
      'max_lon': maxLon,
      'min_zoom': minZoom,
      'max_zoom': maxZoom,
    });
    return unwrapEnvelope(env)['tile_count'] as int;
  }

  /// Builds an offline region manifest from the user's download request.
  ///
  /// Computes the tile count and estimated size. The [regionId] should
  /// be a UUID generated by the Dart layer before calling this.
  static StrideOfflineRegion buildOfflineRegion({
    required String regionId,
    required String name,
    required double minLat,
    required double minLon,
    required double maxLat,
    required double maxLon,
    required int minZoom,
    required int maxZoom,
    required StrideTileProvider provider,
    required int nowMs,
  }) {
    final env = _bindings.buildOfflineRegion({
      'region_id': regionId,
      'name': name,
      'min_lat': minLat,
      'min_lon': minLon,
      'max_lat': maxLat,
      'max_lon': maxLon,
      'min_zoom': minZoom,
      'max_zoom': maxZoom,
      'provider': provider.toJson(),
      'now_ms': nowMs,
    });
    return StrideOfflineRegion.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Checks whether the device has enough free storage to download a
  /// new offline region.
  ///
  /// Returns a [StrideStorageAvailabilityResult] with `available: true`
  /// if the download fits within both the device's free space (with a
  /// 10% margin) and the app's 500 MiB total cache budget.
  static StrideStorageAvailabilityResult checkStorageAvailability({
    required int newRegionSizeBytes,
    required int currentTotalCacheBytes,
    required int deviceFreeBytes,
  }) {
    final env = _bindings.checkStorageAvailability({
      'new_region_size_bytes': newRegionSizeBytes,
      'current_total_cache_bytes': currentTotalCacheBytes,
      'device_free_bytes': deviceFreeBytes,
    });
    return StrideStorageAvailabilityResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Checks whether the user can save a new offline region (count limit
  /// of 20 regions).
  static bool canAddRegion({required int currentRegionCount}) {
    final env = _bindings.canAddRegion({'current_region_count': currentRegionCount});
    return unwrapEnvelope(env)['can_add'] as bool;
  }

  /// Selects the LRU eviction candidate from existing offline regions.
  ///
  /// Returns the index of the region to evict, or `null` if no single
  /// region's deletion would free enough space (the Dart layer should
  /// then offer the user a multi-select to delete several).
  static int? selectEvictionCandidate({
    required List<StrideOfflineRegion> regions,
    required int bytesNeeded,
  }) {
    final env = _bindings.selectEvictionCandidate({
      'regions': regions.map((r) => r.toJson()).toList(),
      'bytes_needed': bytesNeeded,
    });
    return unwrapEnvelope(env)['evict_index'] as int?;
  }

  /// Checks whether a downloaded region is stale (not accessed in 90 days).
  static bool isRegionStale({
    required StrideOfflineRegion region,
    required int nowMs,
  }) {
    final env = _bindings.isRegionStale({
      'region': region.toJson(),
      'now_ms': nowMs,
    });
    return unwrapEnvelope(env)['is_stale'] as bool;
  }

  /// Updates a region's `lastAccessedAt` timestamp (for LRU tracking).
  ///
  /// Returns a new [StrideOfflineRegion] with the updated timestamp.
  static StrideOfflineRegion touchRegion({
    required StrideOfflineRegion region,
    required int nowMs,
  }) {
    final env = _bindings.touchRegion({
      'region': region.toJson(),
      'now_ms': nowMs,
    });
    return StrideOfflineRegion.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Selects the Douglas-Peucker simplification epsilon (in degrees)
  /// appropriate for the current map zoom level.
  static double simplificationEpsilonForZoom({required int zoom}) {
    final env = _bindings.simplificationEpsilonForZoom({'zoom': zoom});
    return (unwrapEnvelope(env)['epsilon'] as num).toDouble();
  }

  /// Whether the current zoom level is high enough (>= 15) that the
  /// full-resolution route should be shown instead of the simplified
  /// polyline.
  static bool shouldShowFullResolutionRoute({required int zoom}) {
    final env = _bindings.shouldShowFullResolutionRoute({'zoom': zoom});
    return unwrapEnvelope(env)['show_full_resolution'] as bool;
  }

  /// Whether the recenter button should be visible. Returns `true` when
  /// the map center is more than 50 m from the user's current location.
  static bool shouldShowRecenterButton({required double distanceFromCenterMeters}) {
    final env = _bindings.shouldShowRecenterButton({
      'distance_from_center_meters': distanceFromCenterMeters,
    });
    return unwrapEnvelope(env)['show_recenter'] as bool;
  }

  /// Whether the map should rotate with the user's heading (compass
  /// mode). Returns `true` when the user is moving faster than 1.4 m/s
  /// (~5 km/h, a slow walk) and the heading is valid.
  static bool shouldRotateWithHeading({
    required double speedMps,
    required bool isHeadingValid,
  }) {
    final env = _bindings.shouldRotateWithHeading({
      'speed_mps': speedMps,
      'is_heading_valid': isHeadingValid,
    });
    return unwrapEnvelope(env)['rotate_with_heading'] as bool;
  }

  /// Validates a [StrideSavedRoute] before persisting it.
  ///
  /// Returns `null` if valid, or an error message string if not.
  static String? validateSavedRoute({required StrideSavedRoute route}) {
    final env = _bindings.validateSavedRoute(route.toJson());
    final data = unwrapEnvelope(env) as Map<String, dynamic>;
    if (data['valid'] == true) return null;
    return data['error'] as String?;
  }

  /// Builds the cache key for a tile in the on-device tile cache.
  ///
  /// Format: `tiles/{provider_slug}/{z}/{x}/{y}`.
  static String tileCacheKey({
    required StrideTileProvider provider,
    required StrideTileCoord tile,
  }) {
    final env = _bindings.tileCacheKey({
      'provider': provider.toJson(),
      'tile': tile.toJson(),
    });
    return unwrapEnvelope(env)['cache_key'] as String;
  }

  /// Returns the short filesystem slug for a tile provider (e.g. `osm`,
  /// `satellite`, `esri`, `custom`).
  static String tileProviderSlug({required StrideTileProvider provider}) {
    final env = _bindings.tileProviderSlug({'provider': provider.toJson()});
    return unwrapEnvelope(env)['slug'] as String;
  }

  // -----------------------------------------------------------------------
  // §6 — Authentication / account lifecycle
  // -----------------------------------------------------------------------

  /// Returns a human-readable label for an auth provider.
  static String authProviderLabel({required StrideAuthProvider provider}) {
    final env = _bindings.authProviderLabel({'provider': provider.toJson()});
    return unwrapEnvelope(env)['label'] as String;
  }

  /// Classifies the session state from token timestamps.
  ///
  /// [tokenIssuedAtMs] is when the token was issued (0 = no session).
  /// [tokenExpiresAtMs] is when the token expires.
  /// [nowMs] is the current time in epoch milliseconds.
  /// [isRevoked] is whether the session has been revoked.
  static StrideSessionState classifySessionState({
    required int tokenIssuedAtMs,
    required int tokenExpiresAtMs,
    required int nowMs,
    required bool isRevoked,
  }) {
    final env = _bindings.classifySessionState({
      'token_issued_at_ms': tokenIssuedAtMs,
      'token_expires_at_ms': tokenExpiresAtMs,
      'now_ms': nowMs,
      'is_revoked': isRevoked,
    });
    return StrideSessionState.fromJson(
        unwrapEnvelope(env)['session_state'] as String);
  }

  /// Whether the session needs a token refresh before making an
  /// authenticated API call.
  static bool needsTokenRefresh({required StrideSessionState sessionState}) {
    final env = _bindings.needsTokenRefresh({'session_state': sessionState.toJson()});
    return unwrapEnvelope(env)['needs_refresh'] as bool;
  }

  /// Whether the user must re-sign-in (session is irrevocably lost).
  static bool requiresRelogin({required StrideSessionState sessionState}) {
    final env = _bindings.requiresRelogin({'session_state': sessionState.toJson()});
    return unwrapEnvelope(env)['needs_relogin'] as bool;
  }

  /// Whether a sensitive action requires recent reauthentication.
  ///
  /// [lastAuthAtMs] is when the user last authenticated (0 = never).
  /// [nowMs] is the current time.
  static bool requiresReauthentication({
    required StrideSensitiveAction action,
    required int lastAuthAtMs,
    required int nowMs,
  }) {
    final env = _bindings.requiresReauthentication({
      'action': action.toJson(),
      'last_auth_at_ms': lastAuthAtMs,
      'now_ms': nowMs,
    });
    return unwrapEnvelope(env)['requires_reauth'] as bool;
  }

  /// Returns the reauthentication threshold in milliseconds for a
  /// sensitive action.
  static int reauthThresholdMs({required StrideSensitiveAction action}) {
    final env = _bindings.reauthThresholdMs({'action': action.toJson()});
    return unwrapEnvelope(env)['threshold_ms'] as int;
  }

  /// Returns the human-readable reason for why reauthentication is needed.
  static String reauthReason({required StrideSensitiveAction action}) {
    final env = _bindings.reauthReason({'action': action.toJson()});
    return unwrapEnvelope(env)['reason'] as String;
  }

  /// Decides what action to take regarding email verification.
  ///
  /// [provider] is the auth provider (Google/Apple are auto-verified).
  /// [isVerified] is the current verification state.
  /// [verificationSentAtMs] is when the verification email was last sent
  /// (0 if never sent).
  /// [nowMs] is the current time.
  static StrideVerificationAction decideVerificationAction({
    required StrideAuthProvider provider,
    required bool isVerified,
    required int verificationSentAtMs,
    required int nowMs,
  }) {
    final env = _bindings.decideVerificationAction({
      'provider': provider.toJson(),
      'is_verified': isVerified,
      'verification_sent_at_ms': verificationSentAtMs,
      'now_ms': nowMs,
    });
    return StrideVerificationAction.fromJson(
        unwrapEnvelope(env)['verification_action'] as String);
  }

  /// Whether a verification email can be resent (cooldown check).
  static bool canResendVerification({
    required int verificationSentAtMs,
    required int nowMs,
  }) {
    final env = _bindings.canResendVerification({
      'verification_sent_at_ms': verificationSentAtMs,
      'now_ms': nowMs,
    });
    return unwrapEnvelope(env)['can_resend'] as bool;
  }

  /// Validates a password against the app's password policy.
  ///
  /// Returns a [StridePasswordValidationResult] with `isValid` and
  /// `issues` fields.
  static StridePasswordValidationResult validatePassword(
      {required String password}) {
    final env = _bindings.validatePassword({'password': password});
    return StridePasswordValidationResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the password strength score (0–4).
  static int passwordStrengthScore({required String password}) {
    final env = _bindings.passwordStrengthScore({'password': password});
    return unwrapEnvelope(env)['score'] as int;
  }

  /// Returns a label for a password strength score.
  static String passwordStrengthLabel({required int score}) {
    final env = _bindings.passwordStrengthLabel({'score': score});
    return unwrapEnvelope(env)['label'] as String;
  }

  /// Validates an email address.
  ///
  /// Returns `null` if valid, or an error message if not.
  static String? validateEmail({required String email}) {
    final env = _bindings.validateEmail({'email': email});
    final data = unwrapEnvelope(env) as Map<String, dynamic>;
    if (data['is_valid'] == true) return null;
    return data['error'] as String?;
  }

  /// Returns a human-readable message for an account status.
  static String accountStatusMessage({required StrideAccountStatus status}) {
    final env = _bindings.accountStatusMessage({'status': status.toJson()});
    return unwrapEnvelope(env)['message'] as String;
  }

  /// Whether the user can sign in given an account status.
  static bool canSignIn({required StrideAccountStatus status}) {
    final env = _bindings.canSignIn({'status': status.toJson()});
    return unwrapEnvelope(env)['can_sign_in'] as bool;
  }

  /// Analyzes a login attempt for suspicious activity.
  static StrideSuspiciousLoginResult analyzeLoginAttempt({
    required StrideLoginContext context,
    required StrideLoginHistory history,
  }) {
    final env = _bindings.analyzeLoginAttempt({
      'context': context.toJson(),
      'history': history.toJson(),
    });
    return StrideSuspiciousLoginResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Enumerates all user-data categories for account deletion.
  ///
  /// Returns a list of [StrideUserDataCategoryInfo] with storage
  /// locations for each category.
  static List<StrideUserDataCategoryInfo> enumerateUserDataCategories() {
    final env = _bindings.enumerateUserDataCategories({});
    final data = unwrapEnvelope(env) as Map<String, dynamic>;
    final categories = data['categories'] as List<dynamic>;
    return categories
        .map((e) => StrideUserDataCategoryInfo.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  /// Builds a deletion plan for a scope.
  ///
  /// Returns the ordered list of [StrideUserDataCategory] to delete.
  static List<StrideUserDataCategory> buildDeletionPlan(
      {required StrideDeletionScope scope}) {
    final env = _bindings.buildDeletionPlan({'scope': scope.toJson()});
    final data = unwrapEnvelope(env) as Map<String, dynamic>;
    final categories = data['categories'] as List<dynamic>;
    return categories
        .map((e) => StrideUserDataCategory.fromJson(e as String))
        .toList();
  }

  /// Builds a deletion result from deleted/failed categories.
  static StrideDeletionResult buildDeletionResult({
    required List<StrideUserDataCategory> deleted,
    required List<({StrideUserDataCategory category, String error})> failed,
    required bool authAccountDeleted,
  }) {
    final env = _bindings.buildDeletionResult({
      'deleted': deleted.map((c) => c.toJson()).toList(),
      'failed': failed
          .map((f) => {'category': f.category.toJson(), 'error': f.error})
          .toList(),
      'auth_account_deleted': authAccountDeleted,
    });
    return StrideDeletionResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Whether the app should sign the user out automatically.
  static bool shouldAutoSignout({
    required StrideSessionState sessionState,
    required StrideAccountStatus accountStatus,
  }) {
    final env = _bindings.shouldAutoSignout({
      'session_state': sessionState.toJson(),
      'account_status': accountStatus.toJson(),
    });
    return unwrapEnvelope(env)['should_signout'] as bool;
  }

  /// Whether an anonymous (guest) account can be upgraded to a permanent
  /// account.
  static bool canUpgradeAnonymous({
    required StrideAuthProvider currentProvider,
    required StrideAuthProvider targetProvider,
  }) {
    final env = _bindings.canUpgradeAnonymous({
      'current_provider': currentProvider.toJson(),
      'target_provider': targetProvider.toJson(),
    });
    return unwrapEnvelope(env)['can_upgrade'] as bool;
  }

  // ===========================================================================
  // §7 — Secure Firebase: security validation wrappers
  // ===========================================================================

  /// Returns the Firestore collection path template for a collection.
  static String securityCollectionPathTemplate({
    required StrideFirestoreCollection collection,
  }) {
    final env = _bindings.securityCollectionPathTemplate({
      'collection': collection.toJson(),
    });
    return unwrapEnvelope(env)['path_template'] as String;
  }

  /// Whether a Firestore collection is admin-only.
  static bool securityCollectionIsAdminOnly({
    required StrideFirestoreCollection collection,
  }) {
    final env = _bindings.securityCollectionIsAdminOnly({
      'collection': collection.toJson(),
    });
    return unwrapEnvelope(env)['is_admin_only'] as bool;
  }

  /// Whether a Firestore collection is user-scoped.
  static bool securityCollectionIsUserScoped({
    required StrideFirestoreCollection collection,
  }) {
    final env = _bindings.securityCollectionIsUserScoped({
      'collection': collection.toJson(),
    });
    return unwrapEnvelope(env)['is_user_scoped'] as bool;
  }

  /// Checks access for a given access context.
  static StrideAccessDecision securityCheckAccess({
    required String userId,
    required bool isAdmin,
    required StrideFirestoreCollection collection,
    required StrideAccessType accessType,
    required String docOwnerId,
  }) {
    final env = _bindings.securityCheckAccess({
      'user_id': userId,
      'is_admin': isAdmin,
      'collection': collection.toJson(),
      'access_type': accessType.toJson(),
      'doc_owner_id': docOwnerId,
    });
    return StrideAccessDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Validates that a Firestore path belongs to the given user.
  /// Returns `null` if valid, or an error message if not.
  static String? securityValidatePathOwnership({
    required String path,
    required String userId,
  }) {
    final env = _bindings.securityValidatePathOwnership({
      'path': path,
      'user_id': userId,
    });
    final data = unwrapEnvelope(env);
    if (data['valid'] as bool) return null;
    return data['error'] as String;
  }

  /// Returns the field validation rules for a collection.
  static List<StrideFieldRule> securityFieldRulesForCollection({
    required StrideFirestoreCollection collection,
  }) {
    final env = _bindings.securityFieldRulesForCollection({
      'collection': collection.toJson(),
    });
    final rules = unwrapEnvelope(env)['rules'] as List<dynamic>;
    return rules
        .map((e) => StrideFieldRule.fromJson(Map<String, dynamic>.from(e as Map)))
        .toList();
  }

  /// Returns the allowed field names for a collection.
  static List<String> securityAllowedFieldsForCollection({
    required StrideFirestoreCollection collection,
  }) {
    final env = _bindings.securityAllowedFieldsForCollection({
      'collection': collection.toJson(),
    });
    final fields = unwrapEnvelope(env)['allowed_fields'] as List<dynamic>;
    return fields.map((e) => e as String).toList();
  }

  /// Validates a document against field rules for a collection.
  static StrideFieldValidationResult securityValidateDocument({
    required StrideFirestoreCollection collection,
    required Map<String, dynamic> doc,
  }) {
    final env = _bindings.securityValidateDocument({
      'collection': collection.toJson(),
      'doc': doc,
    });
    return StrideFieldValidationResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Sanitizes a string (trims, removes control chars, strips XSS patterns).
  static String securitySanitizeString({required String input}) {
    final env = _bindings.securitySanitizeString({'input': input});
    return unwrapEnvelope(env)['sanitized'] as String;
  }

  /// Detects injection patterns in a string.
  /// Returns `null` if clean, or a reason string if suspicious.
  static String? securityDetectInjection({required String input}) {
    final env = _bindings.securityDetectInjection({'input': input});
    return unwrapEnvelope(env)['detected'] as String?;
  }

  /// Whether a string is safe (no injection patterns).
  static bool securityIsSafeString({required String input}) {
    final env = _bindings.securityIsSafeString({'input': input});
    return unwrapEnvelope(env)['is_safe'] as bool;
  }

  /// Validates a Cloud Storage path for user ownership.
  /// Returns `null` if valid, or an error message if not.
  static String? securityValidateStoragePath({
    required String path,
    required String userId,
  }) {
    final env = _bindings.securityValidateStoragePath({
      'path': path,
      'user_id': userId,
    });
    final data = unwrapEnvelope(env);
    if (data['valid'] as bool) return null;
    return data['error'] as String;
  }

  /// Returns the access decision for an App Check state.
  static StrideAccessDecision securityAppCheckDecision({
    required StrideAppCheckState state,
  }) {
    final env = _bindings.securityAppCheckDecision({'state': state.toJson()});
    return StrideAccessDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the access decision for a Play Integrity verdict.
  static StrideAccessDecision securityPlayIntegrityDecision({
    required StridePlayIntegrityVerdict verdict,
  }) {
    final env =
        _bindings.securityPlayIntegrityDecision({'verdict': verdict.toJson()});
    return StrideAccessDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Checks and consumes a token from the rate limit bucket.
  /// Returns the rate limit result and updates the bucket.
  static ({StrideRateLimitResult result, StrideRateLimitBucket bucket})
      securityCheckRateLimit({
    required StrideRateLimitBucket bucket,
    required int nowMs,
  }) {
    final env = _bindings.securityCheckRateLimit({
      'bucket': bucket.toJson(),
      'now_ms': nowMs,
    });
    final data = unwrapEnvelope(env) as Map<String, dynamic>;
    final result = StrideRateLimitResult.fromJson(
        Map<String, dynamic>.from(data['result'] as Map));
    final updatedBucket = StrideRateLimitBucket.fromJson(
        Map<String, dynamic>.from(data['bucket'] as Map));
    return (result: result, bucket: updatedBucket);
  }

  /// Returns the default capacity and refill rate for a rate limit category.
  static ({double capacity, double refillRate}) securityRateLimitConfig({
    required StrideRateLimitCategory category,
  }) {
    final env = _bindings.securityRateLimitConfig({'category': category.toJson()});
    final data = unwrapEnvelope(env);
    return (
      capacity: data['capacity'] as double,
      refillRate: data['refill_rate'] as double,
    );
  }

  /// Checks a config map for secret keys.
  static List<String> securityCheckForSecrets({
    required Map<String, dynamic> config,
  }) {
    final env = _bindings.securityCheckForSecrets({'config': config});
    final found = unwrapEnvelope(env)['found_secrets'] as List<dynamic>;
    return found.map((e) => e as String).toList();
  }

  /// Whether a config map is free of secrets.
  static bool securityIsSecretFree({required Map<String, dynamic> config}) {
    final env = _bindings.securityIsSecretFree({'config': config});
    return unwrapEnvelope(env)['is_secret_free'] as bool;
  }

  /// Validates that a project ID matches the expected environment.
  /// Returns `null` if valid, or an error message if not.
  static String? securityValidateProjectId({
    required String projectId,
    required StrideFirebaseEnvironment expected,
  }) {
    final env = _bindings.securityValidateProjectId({
      'project_id': projectId,
      'expected': expected.toJson(),
    });
    final data = unwrapEnvelope(env);
    if (data['valid'] as bool) return null;
    return data['error'] as String;
  }

  /// Parses a project ID string into a Firebase environment.
  static StrideFirebaseEnvironment? securityEnvironmentFromProjectId({
    required String projectId,
  }) {
    final env =
        _bindings.securityEnvironmentFromProjectId({'project_id': projectId});
    final result = unwrapEnvelope(env)['environment'];
    if (result == null) return null;
    return StrideFirebaseEnvironment.fromJson(result as String);
  }

  /// Generates the Firestore security rules text.
  static String securityGenerateFirestoreRules() {
    final env = _bindings.securityGenerateFirestoreRules({});
    return unwrapEnvelope(env)['rules'] as String;
  }

  /// Generates the Cloud Storage security rules text.
  static String securityGenerateStorageRules() {
    final env = _bindings.securityGenerateStorageRules({});
    return unwrapEnvelope(env)['rules'] as String;
  }

// ===========================================================================
// §8 — AI coaching plan & safety guards
// ===========================================================================

  /// Returns the caps (max distance, duration, rest days) for an experience level.
  static StrideExperienceCaps coachingPlanExperienceCaps({
    required StrideExperienceLevel experienceLevel,
  }) {
    final env = _bindings.coachingPlanExperienceCaps({
      'experience_level': experienceLevel.toJson(),
    });
    return StrideExperienceCaps.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Validates a single day plan against the experience-level limits.
  /// Returns a list of validation issues (empty if valid).
  static List<String> coachingPlanValidateDayPlan({
    required StrideDayPlan day,
    required StrideExperienceLevel experienceLevel,
  }) {
    final env = _bindings.coachingPlanValidateDayPlan({
      'day': day.toJson(),
      'experience_level': experienceLevel.toJson(),
    });
    final issues = unwrapEnvelope(env)['issues'] as List<dynamic>;
    return issues.map((e) => e as String).toList();
  }

  /// Validates a weekly plan against limits and the 10% increase rule.
  static StridePlanValidationResult coachingPlanValidateWeeklyPlan({
    required StrideWeeklyPlan plan,
    double? previousWeeklyDistanceM,
  }) {
    final env = _bindings.coachingPlanValidateWeeklyPlan({
      'plan': plan.toJson(),
      'previous_weekly_distance_m': previousWeeklyDistanceM,
    });
    return StridePlanValidationResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Generates a deterministic, rule-based fallback weekly plan.
  static StrideWeeklyPlan coachingPlanGenerateFallback({
    required StrideExperienceLevel experienceLevel,
    required double currentWeeklyDistanceM,
  }) {
    final env = _bindings.coachingPlanGenerateFallback({
      'experience_level': experienceLevel.toJson(),
      'current_weekly_distance_m': currentWeeklyDistanceM,
    });
    return StrideWeeklyPlan.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Generates a non-diagnostic pain response for the given pain type.
  static StridePainResponse coachingPlanRespondToPain({
    required StridePainType pain,
  }) {
    final env = _bindings.coachingPlanRespondToPain({
      'pain': pain.toJson(),
    });
    return StridePainResponse.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Checks whether text contains medical diagnosis language.
  static bool coachingPlanContainsDiagnosis({required String text}) {
    final env = _bindings.coachingPlanContainsDiagnosis({'text': text});
    return unwrapEnvelope(env)['contains_diagnosis'] as bool;
  }

  /// Checks whether text contains weight-loss guarantee language.
  static bool coachingPlanContainsWeightLossPromise({required String text}) {
    final env = _bindings.coachingPlanContainsWeightLossPromise({'text': text});
    return unwrapEnvelope(env)['contains_weight_loss_promise'] as bool;
  }

  /// Validates AI-generated coaching text against content guards.
  static StrideContentValidationResult coachingPlanValidateCoachingText({
    required String text,
  }) {
    final env = _bindings.coachingPlanValidateCoachingText({'text': text});
    return StrideContentValidationResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Generates an escalation message for concerning symptoms.
  static StrideEscalationMessage coachingPlanEscalationMessage({
    required StrideEscalationReason reason,
  }) {
    final env = _bindings.coachingPlanEscalationMessage({
      'reason': reason.toJson(),
    });
    return StrideEscalationMessage.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Processes user feedback on a plan and returns the recommended action.
  static StrideFeedbackResult coachingPlanProcessUserFeedback({
    required StrideUserFeedback feedback,
  }) {
    final env = _bindings.coachingPlanProcessUserFeedback({
      'feedback': feedback.toJson(),
    });
    return StrideFeedbackResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Decides whether the AI should be called or the fallback used.
  static StrideAiAvailability coachingPlanDecideAiAvailability({
    required bool isServiceUp,
    int? remainingQuota,
    required int estimatedCostCents,
    int? costBudgetCents,
    required bool isEnabled,
  }) {
    final env = _bindings.coachingPlanDecideAiAvailability({
      'is_service_up': isServiceUp,
      'remaining_quota': remainingQuota,
      'estimated_cost_cents': estimatedCostCents,
      'cost_budget_cents': costBudgetCents,
      'is_enabled': isEnabled,
    });
    return StrideAiAvailability.fromJson(
        unwrapEnvelope(env) as String);
  }

  /// Whether the rule-based fallback should be used instead of the AI.
  static bool coachingPlanShouldUseFallback({
    required StrideAiAvailability availability,
  }) {
    final env = _bindings.coachingPlanShouldUseFallback({
      'availability': availability.toJson(),
    });
    return unwrapEnvelope(env)['should_use_fallback'] as bool;
  }

  /// Estimates the cost in cents for an AI operation.
  static int coachingPlanEstimateAiCost({
    required StrideAiOperation operation,
  }) {
    final env = _bindings.coachingPlanEstimateAiCost({
      'operation': operation.toJson(),
    });
    return unwrapEnvelope(env)['cost_cents'] as int;
  }

  /// Generates a deterministic cache key for an AI request.
  static String coachingPlanCacheKey({
    required StrideAiOperation operation,
    required String userInput,
  }) {
    final env = _bindings.coachingPlanCacheKey({
      'operation': operation.toJson(),
      'user_input': userInput,
    });
    return unwrapEnvelope(env)['cache_key'] as String;
  }

  /// Summarizes a completed workout using rule-based logic (fallback).
  static String coachingPlanSummarizeWorkout({
    required StrideWorkoutSummaryInput input,
  }) {
    final env = _bindings.coachingPlanSummarizeWorkout(input.toJson());
    return unwrapEnvelope(env)['summary'] as String;
  }

  /// Generates a rule-based encouragement message based on recent activity.
  static String coachingPlanGenerateEncouragement({
    required int daysActiveLastWeek,
    required double totalDistanceLastWeekM,
    required double goalDistanceM,
  }) {
    final env = _bindings.coachingPlanGenerateEncouragement({
      'days_active_last_week': daysActiveLastWeek,
      'total_distance_last_week_m': totalDistanceLastWeekM,
      'goal_distance_m': goalDistanceM,
    });
    return unwrapEnvelope(env)['message'] as String;
  }

  /// Adjusts a plan based on user feedback using rule-based logic (fallback).
  static StridePlanAdjustmentResult coachingPlanAdjustPlan({
    required StridePlanAdjustmentInput input,
  }) {
    final env = _bindings.coachingPlanAdjustPlan(input.toJson());
    return StridePlanAdjustmentResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Recommends a realistic weekly distance progression.
  static double coachingPlanRecommendProgression({
    required double currentWeeklyDistanceM,
    required StrideExperienceLevel experienceLevel,
    required int weeksAtCurrentLevel,
  }) {
    final env = _bindings.coachingPlanRecommendProgression({
      'current_weekly_distance_m': currentWeeklyDistanceM,
      'experience_level': experienceLevel.toJson(),
      'weeks_at_current_level': weeksAtCurrentLevel,
    });
    return unwrapEnvelope(env)['recommended_weekly_distance_m'] as double;
  }

  /// Moderates a user request for safety before processing.
  static StrideRequestModerationResult coachingPlanModerateRequest({
    required String request,
  }) {
    final env = _bindings.coachingPlanModerateRequest({'request': request});
    return StrideRequestModerationResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  // ─── §9 — Calorie/fitness calculations ─────────────────────────────

  /// Produces a full calorie estimate (with method, version, labels, and
  /// source-priority) for the given inputs, following the documented
  /// source-priority chain: wearable > heart_rate > met > distance_weight.
  static StrideCalorieEstimateResult calorieEstimate({
    required StrideCalorieInputs inputs,
  }) {
    final env = _bindings.calorieEstimate(inputs.toJson());
    return StrideCalorieEstimateResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Validates and clamps a calorie value to a plausible range (0–10,000
  /// kcal), returning the clamped value along with whether it was modified.
  static StrideCalorieClampResult calorieClamp({
    required double kcal,
  }) {
    final env = _bindings.calorieClamp({'kcal': kcal});
    return StrideCalorieClampResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the source-priority order for calorie estimation methods, as a
  /// list of (method, rank, label) entries. Documents the fallback chain.
  static List<Map<String, dynamic>> calorieSourcePriority() {
    final env = _bindings.calorieSourcePriority();
    final data = unwrapEnvelope(env);
    return (data as List).cast<Map<String, dynamic>>();
  }

  // ─── §10 — Wearable / Health Connect ─────────────────────────────

  /// Decides the appropriate operating mode (phone-only, watch, or Health
  /// Connect) given the current source availability.
  static StrideFallbackDecision wearableDecideFallback({
    required StrideSourceAvailability availability,
  }) {
    final env = _bindings.wearableDecideFallback(availability.toJson());
    return StrideFallbackDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds a full wearable status snapshot, combining the connection
  /// state, consent state, sync status, fallback decision, revocation
  /// log, and device info.
  static StrideWearableStatus wearableBuildStatus({
    required StrideSourceAvailability availability,
    required StrideHealthConnectConsentState consentState,
    required int nowMs,
    int? lastSyncMs,
    StrideWearableSyncConfig? syncConfig,
    List<StrideSourceRevocationRecord> revocations = const [],
    StrideWearableDeviceInfo? device,
  }) {
    final env = _bindings.wearableBuildStatus({
      'availability': availability.toJson(),
      'consent_state': consentState.toJson(),
      'last_sync_ms': lastSyncMs,
      'now_ms': nowMs,
      'sync_config': (syncConfig ?? StrideWearableSyncConfig()).toJson(),
      'revocations': revocations.map((r) => r.toJson()).toList(),
      'device': device?.toJson(),
    });
    return StrideWearableStatus.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Evaluates the current sync status of a wearable, given the
  /// connection state, the time of the last successful sync, and the
  /// current time.
  static StrideWearableSyncStatusResult wearableSyncStatus({
    required StrideWearableConnectionState connectionState,
    required int nowMs,
    int? lastSyncMs,
    int staleThresholdMs = 300000,
  }) {
    final env = _bindings.wearableSyncStatus({
      'connection_state': connectionState.toJson(),
      'last_sync_ms': lastSyncMs,
      'now_ms': nowMs,
      'stale_threshold_ms': staleThresholdMs,
    });
    return StrideWearableSyncStatusResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Decides which source wins when the same metric arrives from two
  /// sources at approximately the same time (duplicate-record prevention).
  static StrideDeduplicateSourceResult wearableDeduplicateSource({
    required StrideSensorSource sourceA,
    required StrideSensorSource sourceB,
    required StrideMetricType metric,
  }) {
    final env = _bindings.wearableDeduplicateSource({
      'source_a': sourceA.toJson(),
      'source_b': sourceB.toJson(),
      'metric': metric.toJson(),
    });
    return StrideDeduplicateSourceResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Processes a Health Connect consent request result, transitioning
  /// the consent state.
  static StrideConsentResult wearableConsentResult({
    required StrideHealthConnectConsentState currentState,
    required bool granted,
  }) {
    final env = _bindings.wearableConsentResult({
      'current_state': currentState.toJson(),
      'granted': granted,
    });
    return StrideConsentResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  // ─── §11 — Music system ───────────────────────────────────────

  /// Transitions the playback state machine given a command.
  static StrideTransitionResult musicTransition({
    required StridePlaybackState currentState,
    required StridePlaybackCommand command,
  }) {
    final env = _bindings.musicTransition({
      'current_state': currentState.toJson(),
      'command': command.toJson(),
    });
    return StrideTransitionResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Handles an audio focus event, returning the new focus state and
  /// recommended playback action.
  static StrideAudioFocusResult musicAudioFocus({
    required StrideAudioFocusState currentFocus,
    required StrideAudioFocusEvent event,
  }) {
    final env = _bindings.musicAudioFocus({
      'current_focus': currentFocus.toJson(),
      'event': event.toJson(),
    });
    return StrideAudioFocusResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Coordinates music with a coaching prompt, deciding whether to
  /// pause or resume music.
  static StrideCoachingInteropResult musicCoachingInterop({
    required StrideCoachingInteropState currentState,
    required StrideCoachingRequest request,
    required bool musicIsPlaying,
  }) {
    final env = _bindings.musicCoachingInterop({
      'current_state': currentState.toJson(),
      'request': request.toJson(),
      'music_is_playing': musicIsPlaying,
    });
    return StrideCoachingInteropResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Decides what to do when the network is lost during music
  /// streaming.
  static StrideNetworkLossDecision musicNetworkLoss({
    required StrideNetworkState network,
    required StrideMusicSource currentSource,
    required bool hasLocalMedia,
    required int bufferHealthMs,
  }) {
    final env = _bindings.musicNetworkLoss({
      'network': network.toJson(),
      'current_source': currentSource.toJson(),
      'has_local_media': hasLocalMedia,
      'buffer_health_ms': bufferHealthMs,
    });
    return StrideNetworkLossDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Filters blocked content from a playlist.
  static StrideFilterBlockedResult musicFilterBlocked({
    required StridePlaylist playlist,
    required List<String> blockedArtists,
    required List<String> blockedGenres,
  }) {
    final env = _bindings.musicFilterBlocked({
      'playlist': playlist.toJson(),
      'blocked_artists': blockedArtists,
      'blocked_genres': blockedGenres,
    });
    return StrideFilterBlockedResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Decides whether a track should be recommended again based on the
  /// user's feedback history.
  static StrideShouldRecommendResult musicShouldRecommend({
    required List<StrideFeedbackRecord> feedbackHistory,
    required String trackId,
    required int nowMs,
    required int skipCooldownMs,
  }) {
    final env = _bindings.musicShouldRecommend({
      'feedback_history': feedbackHistory.map((r) => r.toJson()).toList(),
      'track_id': trackId,
      'now_ms': nowMs,
      'skip_cooldown_ms': skipCooldownMs,
    });
    return StrideShouldRecommendResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds a full music status snapshot for the UI.
  static StrideMusicStatus musicBuildStatus({
    required StridePlaybackState playback,
    required StrideAudioFocusState focus,
    required StrideCoachingInteropState coaching,
    required StrideMusicSource source,
    required StrideMusicMode mode,
    required StrideNetworkState network,
    int? currentTrackIndex,
    StridePlaylist? playlist,
  }) {
    final env = _bindings.musicBuildStatus({
      'playback': playback.toJson(),
      'focus': focus.toJson(),
      'coaching': coaching.toJson(),
      'source': source.toJson(),
      'mode': mode.toJson(),
      'network': network.toJson(),
      'current_track_index': currentTrackIndex,
      'playlist': playlist?.toJson(),
    });
    return StrideMusicStatus.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Processes a remote control command (from a lock screen, Bluetooth
  /// headset, Wear OS, or notification).
  static StrideTransitionResult musicRemoteControl({
    required StridePlaybackState current,
    required StridePlaybackCommand command,
    required StrideRemoteControlSource source,
  }) {
    final env = _bindings.musicRemoteControl({
      'current': current.toJson(),
      'command': command.toJson(),
      'source': source.toJson(),
    });
    return StrideTransitionResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  // ─── §12 — Background execution ───────────────────────────────

  /// Transitions the foreground service state machine.
  static StrideForegroundServiceTransition backgroundServiceTransition({
    required StrideForegroundServiceState currentState,
    required StrideForegroundServiceCommand command,
  }) {
    final env = _bindings.backgroundServiceTransition({
      'current_state': currentState.toJson(),
      'command': command.toJson(),
    });
    return StrideForegroundServiceTransition.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Decides the checkpoint write interval for the given workout phase
  /// and battery state.
  static StrideCheckpointSchedule backgroundCheckpointInterval({
    required StrideWorkoutPhase phase,
    required StrideCheckpointBatteryContext battery,
  }) {
    final env = _bindings.backgroundCheckpointInterval({
      'phase': phase.toJson(),
      'battery': battery.toJson(),
    });
    return StrideCheckpointSchedule.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Evaluates whether background execution is permitted right now.
  static StrideBackgroundExecutionDecision backgroundEvaluate({
    required StrideBackgroundTrackingPreference preference,
    required bool serviceRunning,
    required bool backgroundLocationGranted,
    required bool workoutActive,
    required StrideCheckpointBatteryContext battery,
  }) {
    final env = _bindings.backgroundEvaluate({
      'preference': preference.toJson(),
      'service_running': serviceRunning,
      'background_location_granted': backgroundLocationGranted,
      'workout_active': workoutActive,
      'battery': battery.toJson(),
    });
    return StrideBackgroundExecutionDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Evaluates whether a workout can be resumed after a process kill.
  static StrideProcessKillRecoveryDecision backgroundProcessKill({
    required int checkpointAgeMs,
    required bool wasActive,
  }) {
    final env = _bindings.backgroundProcessKill({
      'checkpoint_age_ms': checkpointAgeMs,
      'was_active': wasActive,
    });
    return StrideProcessKillRecoveryDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Decides the power mode and notification update interval based on
  /// the battery state.
  static StrideBackgroundPowerModeResult backgroundPowerMode({
    required StrideCheckpointBatteryContext battery,
  }) {
    final env = _bindings.backgroundPowerMode(battery.toJson());
    return StrideBackgroundPowerModeResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Handles an interruption event during a background workout.
  static StrideInterruptionAction backgroundInterruption({
    required StrideInterruptionEvent event,
    required StrideForegroundServiceState serviceState,
    required bool backgroundAllowed,
  }) {
    final env = _bindings.backgroundInterruption({
      'event': event.toJson(),
      'service_state': serviceState.toJson(),
      'background_allowed': backgroundAllowed,
    });
    return StrideInterruptionAction.fromJson(
        unwrapEnvelope(env) as String);
  }

  /// Assesses the current battery usage.
  static StrideBatteryUseAssessment backgroundBatteryAssessment({
    required int gpsIntervalMs,
    required int sensorIntervalMs,
    required int batteryPercent,
    required bool isCharging,
  }) {
    final env = _bindings.backgroundBatteryAssessment({
      'gps_interval_ms': gpsIntervalMs,
      'sensor_interval_ms': sensorIntervalMs,
      'battery_percent': batteryPercent,
      'is_charging': isCharging,
    });
    return StrideBatteryUseAssessment.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds a full background status snapshot for the UI.
  static StrideBackgroundStatus backgroundBuildStatus({
    required StrideForegroundServiceState serviceState,
    required StrideCheckpointBatteryContext battery,
    required StrideBackgroundTrackingPreference preference,
    required bool backgroundLocationGranted,
    required bool workoutActive,
    required int gpsIntervalMs,
    required int sensorIntervalMs,
  }) {
    final env = _bindings.backgroundBuildStatus({
      'service_state': serviceState.toJson(),
      'battery': battery.toJson(),
      'preference': preference.toJson(),
      'background_location_granted': backgroundLocationGranted,
      'workout_active': workoutActive,
      'gps_interval_ms': gpsIntervalMs,
      'sensor_interval_ms': sensorIntervalMs,
    });
    return StrideBackgroundStatus.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the background location explanation text for the Play
  /// Store and in-app rationale dialog. Pass `short: true` for a
  /// shorter version suitable for notifications.
  static String backgroundExplanation({bool short = false}) {
    final env = _bindings.backgroundExplanation({'short': short});
    return unwrapEnvelope(env) as String;
  }

  // ── §13 — Notifications ───────────────────────────────────────

  /// Decides whether & how to deliver a notification given the full
  /// decision context (permission, preferences, quiet hours, voice
  /// coaching, timezone, etc.). This is the main entry point for the
  /// notification decision logic.
  static StrideNotificationDecision notificationDecide({
    required StrideNotificationType notifType,
    required StrideNotificationPreferences preferences,
    required StrideNotificationPermission permission,
    required StrideQuietHoursConfig quietHours,
    required int nowUtcMs,
    required StrideTimezoneContext timezone,
    required StrideNotificationContextParams contextParams,
  }) {
    final env = _bindings.notificationDecide({
      'notif_type': notifType.toJson(),
      'preferences': preferences.toJson(),
      'permission': permission.toJson(),
      'quiet_hours': quietHours.toJson(),
      'now_utc_ms': nowUtcMs,
      'timezone': timezone.toJson(),
      'context_params': contextParams.toJson(),
    });
    return StrideNotificationDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds the notification content (title, body, action label) for
  /// a given notification type and optional context params.
  static StrideNotificationContent notificationContent({
    required StrideNotificationType notifType,
    StrideNotificationContextParams? contextParams,
  }) {
    final env = _bindings.notificationContent({
      'notif_type': notifType.toJson(),
      'context_params': (contextParams ?? StrideNotificationContextParams())
          .toJson(),
    });
    return StrideNotificationContent.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Evaluates whether a notification should be delivered now given
  /// the quiet hours configuration, notification type, local minute,
  /// and critical-bypass flag.
  static StrideQuietHoursDecision notificationQuietHours({
    required StrideQuietHoursConfig quietHours,
    required StrideNotificationType notifType,
    required int localMinute,
    required bool criticalBypass,
  }) {
    final env = _bindings.notificationQuietHours({
      'quiet_hours': quietHours.toJson(),
      'notif_type': notifType.toJson(),
      'local_minute': localMinute,
      'critical_bypass': criticalBypass,
    });
    return StrideQuietHoursDecision.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Evaluates whether a voice coaching announcement should be made
  /// now given the coaching config, quiet hours, announcement kind,
  /// last distance/time, and local minute. Returns a boolean.
  static bool notificationCoaching({
    required StrideVoiceCoachingConfig voiceConfig,
    required StrideQuietHoursConfig quietHours,
    required StrideCoachingAnnouncementKind kind,
    required double lastDistanceM,
    required int lastTimeS,
    required int localMinute,
  }) {
    final env = _bindings.notificationCoaching({
      'voice_config': voiceConfig.toJson(),
      'quiet_hours': quietHours.toJson(),
      'kind': kind.toJson(),
      'last_distance_m': lastDistanceM,
      'last_time_s': lastTimeS,
      'local_minute': localMinute,
    });
    return unwrapEnvelope(env) as bool;
  }

  /// Builds a full notification status snapshot for the UI /
  /// diagnostics given permission, preferences, quiet hours, voice
  /// coaching config, and timezone.
  static StrideNotificationStatus notificationStatus({
    required StrideNotificationPermission permission,
    required StrideNotificationPreferences preferences,
    required StrideQuietHoursConfig quietHours,
    required StrideVoiceCoachingConfig voiceConfig,
    required StrideTimezoneContext timezone,
  }) {
    final env = _bindings.notificationStatus({
      'permission': permission.toJson(),
      'preferences': preferences.toJson(),
      'quiet_hours': quietHours.toJson(),
      'voice_config': voiceConfig.toJson(),
      'timezone': timezone.toJson(),
    });
    return StrideNotificationStatus.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Computes the UTC timestamp (epoch milliseconds) for the next
  /// daily reminder at a given local-time minute.
  static int notificationNextReminder({
    required int nowUtcMs,
    required int localMinute,
    required StrideTimezoneContext timezone,
  }) {
    final env = _bindings.notificationNextReminder({
      'now_utc_ms': nowUtcMs,
      'local_minute': localMinute,
      'timezone': timezone.toJson(),
    });
    return unwrapEnvelope(env) as int;
  }

  // ── §14 — Error / Recovery States ───────────────────────────────

  /// Decides what to do in response to an error given the error
  /// context and retry policy. This is the main entry point for the
  /// error recovery decision logic.
  static StrideRecoveryAction errorDecideRecovery({
    required StrideErrorContext error,
    required StrideRetryPolicy policy,
  }) {
    final env = _bindings.errorDecideRecovery({
      'error': error.toJson(),
      'policy': policy.toJson(),
    });
    return StrideRecoveryAction.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Computes the retry delay in milliseconds for a given attempt
  /// number and retry policy using exponential backoff with jitter.
  static int errorRetryDelay({
    required int attempt,
    required StrideRetryPolicy policy,
  }) {
    final env = _bindings.errorRetryDelay({
      'attempt': attempt,
      'policy': policy.toJson(),
    });
    return unwrapEnvelope(env) as int;
  }

  /// Validates and performs an error state transition. Returns an
  /// [StrideErrorStateTransition] with `isValid` and `message`.
  static StrideErrorStateTransition errorTransition({
    required StrideErrorState from,
    required StrideErrorState to,
  }) {
    final env = _bindings.errorTransition({
      'from': from.toJson(),
      'to': to.toJson(),
    });
    return StrideErrorStateTransition.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds an overall engine health status from an error registry.
  static StrideEngineHealthStatus errorHealthStatus({
    required StrideErrorRegistry registry,
  }) {
    final env = _bindings.errorHealthStatus({
      'registry': registry.toJson(),
    });
    return StrideEngineHealthStatus.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns whether errors of a given category are retryable.
  static bool errorIsRecoverable({
    required StrideErrorCategory category,
  }) {
    final env = _bindings.errorIsRecoverable({
      'category': category.toJson(),
    });
    return unwrapEnvelope(env) as bool;
  }

  // ── §15 — Testing ──────────────────────────────────────────────

  /// Runs a test suite (simulated) and returns the suite with results
  /// filled in. The actual test execution happens on the Dart/Kotlin
  /// side — this produces a result structure that can be serialized
  /// and sent to the UI.
  static StrideTestSuite testingRunSuite({
    required StrideTestSuite suite,
  }) {
    final env = _bindings.testingRunSuite({
      'suite': suite.toJson(),
    });
    return StrideTestSuite.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Runs a real-device scenario (simulated) and returns the scenario
  /// result (all behaviors passed, zero duration — simulated run).
  static StrideScenarioResult testingRunScenario({
    required StrideRealDeviceScenario scenario,
  }) {
    final env = _bindings.testingRunScenario({
      'scenario': scenario.toJson(),
    });
    return StrideScenarioResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds a test report from a test registry, aggregating all suite
  /// and scenario results with coverage metrics.
  static StrideTestReport testingBuildReport({
    required String name,
    required StrideTestRegistry registry,
  }) {
    final env = _bindings.testingBuildReport({
      'name': name,
      'registry': registry.toJson(),
    });
    return StrideTestReport.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Looks up a test config by its id from the full test registry.
  /// Returns the [StrideTestConfig] if found, or throws
  /// [StrideEngineException] if the test id is not found.
  static StrideTestConfig testingGetConfig({
    required String testId,
  }) {
    final env = _bindings.testingGetConfig({
      'test_id': testId,
    });
    return StrideTestConfig.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the list of all standard test suite names.
  static List<String> testingListSuites() {
    final env = _bindings.testingListSuites({});
    return (unwrapEnvelope(env) as List).cast<String>();
  }

  // ─── §16 Monitoring ───────────────────────────────────────────────

  /// Checks all alert thresholds against the provided metric values and
  /// returns every alert that was triggered.
  static List<StrideAlert> monitoringCheckAlerts({
    required int timestampMs,
    StrideAlertThresholds? thresholds,
    double cloudFunctionErrorRate = 0.0,
    int cloudFunctionLatencyMs = 0,
    int firestoreReadsPerDay = 0,
    int firestoreWritesPerDay = 0,
    int firestoreDeletesPerDay = 0,
    int storageUsageBytes = 0,
    int storageBandwidthBytes = 0,
    int billingBudgetCents = 0,
    int aiCostPerDayCents = 0,
    double aiErrorRate = 0.0,
    double syncFailureRate = 0.0,
    double uptime = 1.0,
    double crashRatePer1000 = 0.0,
  }) {
    final env = _bindings.monitoringCheckAlerts({
      'thresholds': (thresholds ?? const StrideAlertThresholds()).toJson(),
      'timestamp_ms': timestampMs,
      'cloud_function_error_rate': cloudFunctionErrorRate,
      'cloud_function_latency_ms': cloudFunctionLatencyMs,
      'firestore_reads_per_day': firestoreReadsPerDay,
      'firestore_writes_per_day': firestoreWritesPerDay,
      'firestore_deletes_per_day': firestoreDeletesPerDay,
      'storage_usage_bytes': storageUsageBytes,
      'storage_bandwidth_bytes': storageBandwidthBytes,
      'billing_budget_cents': billingBudgetCents,
      'ai_cost_per_day_cents': aiCostPerDayCents,
      'ai_error_rate': aiErrorRate,
      'sync_failure_rate': syncFailureRate,
      'uptime': uptime,
      'crash_rate_per_1000': crashRatePer1000,
    });
    final data = unwrapEnvelope(env) as List;
    return data
        .map((e) => StrideAlert.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  /// Builds a release-health dashboard from the provided metric values
  /// and optional alert list.
  static StrideReleaseHealthDashboard monitoringBuildDashboard({
    required String appVersion,
    required int generatedAtMs,
    double crashFreeRate = 0.0,
    int activeUsers24h = 0,
    int workouts24h = 0,
    double syncFailureRate = 0.0,
    double aiErrorRate = 0.0,
    double overallUptime = 0.0,
    int aiCost24hCents = 0,
    int firestoreReads24h = 0,
    int storageUsageBytes = 0,
    List<StrideAlert> alerts = const [],
  }) {
    final env = _bindings.monitoringBuildDashboard({
      'app_version': appVersion,
      'generated_at_ms': generatedAtMs,
      'crash_free_rate': crashFreeRate,
      'active_users_24h': activeUsers24h,
      'workouts_24h': workouts24h,
      'sync_failure_rate': syncFailureRate,
      'ai_error_rate': aiErrorRate,
      'overall_uptime': overallUptime,
      'ai_cost_24h_cents': aiCost24hCents,
      'firestore_reads_24h': firestoreReads24h,
      'storage_usage_bytes': storageUsageBytes,
      'alerts': alerts.map((a) => a.toJson()).toList(),
    });
    return StrideReleaseHealthDashboard.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Creates a structured log entry with the given level, category,
  /// message, and optional context/session/user.
  static StrideLogEntry monitoringLogEntry({
    required int timestampMs,
    required StrideLogLevel level,
    required StrideMonitoringCategory category,
    required String message,
    List<StrideKeyValuePair> context = const [],
    String? sessionId,
    String? userId,
  }) {
    final env = _bindings.monitoringLogEntry({
      'timestamp_ms': timestampMs,
      'level': level.toJson(),
      'category': category.toJson(),
      'message': message,
      'context': context.map((e) => e.toJson()).toList(),
      'session_id': sessionId,
      'user_id': userId,
    });
    return StrideLogEntry.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Creates a crash report with the given severity, exception type,
  /// message, and optional stack trace, breadcrumbs, device info, and
  /// session context.
  static StrideCrashReport monitoringCrashReport({
    required int timestampMs,
    required StrideCrashSeverity severity,
    required String exceptionType,
    required String message,
    String? stackTrace,
    List<StrideLogEntry> breadcrumbs = const [],
    String appVersion = '',
    String deviceModel = '',
    String osVersion = '',
    bool duringWorkout = false,
    String? sessionId,
  }) {
    final env = _bindings.monitoringCrashReport({
      'timestamp_ms': timestampMs,
      'severity': severity.toJson(),
      'exception_type': exceptionType,
      'message': message,
      'stack_trace': stackTrace,
      'breadcrumbs': breadcrumbs.map((e) => e.toJson()).toList(),
      'app_version': appVersion,
      'device_model': deviceModel,
      'os_version': osVersion,
      'during_workout': duringWorkout,
      'session_id': sessionId,
    });
    return StrideCrashReport.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the standard uptime monitor with all six monitored
  /// services (auth, firestore, cloud_storage, cloud_functions,
  /// ai_backend, push_notifications), all in Unknown status.
  static StrideUptimeMonitor monitoringUptime() {
    final env = _bindings.monitoringUptime({});
    return StrideUptimeMonitor.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  // ─── §17 Backups and disaster recovery ─────────────────────────

  /// Returns the default backup schedule configuration (weekly,
  /// Sunday 02:00 UTC, 30-day retention, all collections, includes
  /// Cloud Storage, bucket `stride-backups`, region `us-central1`).
  static StrideBackupConfig backupGetConfig() {
    final env = _bindings.backupGetConfig({});
    return StrideBackupConfig.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Builds a backup manifest from the given config, timestamp, and
  /// list of backup records, returning the assembled manifest with
  /// recomputed aggregate fields (total size, success/fail counts).
  static StrideBackupManifest backupManifest({
    StrideBackupConfig? config,
    required int generatedAtMs,
    List<StrideBackupRecord> records = const [],
  }) {
    final env = _bindings.backupManifest({
      'config': (config ?? StrideBackupConfig()).toJson(),
      'generated_at_ms': generatedAtMs,
      'records': records.map((e) => e.toJson()).toList(),
    });
    return StrideBackupManifest.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Executes a restore operation from the given restore request,
  /// returning the completed restore result with the given document
  /// and file counts, size, and validation flag.
  static StrideRestoreResult backupRestore({
    required StrideRestoreRequest request,
    required int completedAtMs,
    int documentsRestored = 0,
    int filesRestored = 0,
    int sizeRestoredBytes = 0,
    bool validationPassed = true,
  }) {
    final env = _bindings.backupRestore({
      'request': request.toJson(),
      'completed_at_ms': completedAtMs,
      'documents_restored': documentsRestored,
      'files_restored': filesRestored,
      'size_restored_bytes': sizeRestoredBytes,
      'validation_passed': validationPassed,
    });
    return StrideRestoreResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Runs the default migration plan (v1→v2), executing all steps
  /// and returning the completed plan with all steps marked
  /// `completed`.
  static StrideMigrationPlan backupMigrate({
    required int createdAtMs,
    required int executedAtMs,
  }) {
    final env = _bindings.backupMigrate({
      'created_at_ms': createdAtMs,
      'executed_at_ms': executedAtMs,
    });
    return StrideMigrationPlan.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Creates a user data export result from the given export request,
  /// returning the completed export with the given size, workout and
  /// route counts, and download URL.
  static StrideExportResult backupExport({
    required StrideExportRequest request,
    required int completedAtMs,
    int sizeBytes = 0,
    int workoutCount = 0,
    int routeCount = 0,
    String downloadUrl = '',
  }) {
    final env = _bindings.backupExport({
      'request': request.toJson(),
      'completed_at_ms': completedAtMs,
      'size_bytes': sizeBytes,
      'workout_count': workoutCount,
      'route_count': routeCount,
      'download_url': downloadUrl,
    });
    return StrideExportResult.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  // ── §18 Privacy, legal, and safety ──────────────────────────────────

  /// Returns the default privacy policy, terms of service, health
  /// disclaimer, and retention policy as a combined JSON object.
  static Map<String, dynamic> privacyGetPolicy({
    int lastUpdatedAtMs = 0,
  }) {
    final env = _bindings.privacyGetPolicy({
      'last_updated_ms': lastUpdatedAtMs,
    });
    final data = unwrapEnvelope(env) as Map<String, dynamic>;
    return {
      'privacy_policy': StridePrivacyPolicy.fromJson(
          data['privacy_policy'] as Map<String, dynamic>),
      'terms_of_service': StrideTermsOfService.fromJson(
          data['terms_of_service'] as Map<String, dynamic>),
      'health_disclaimer': StrideHealthDisclaimer.fromJson(
          data['health_disclaimer'] as Map<String, dynamic>),
      'retention_policy': StridePrivacyRetentionPolicy.fromJson(
          data['retention_policy'] as Map<String, dynamic>),
    };
  }

  /// Returns the default consent registry with all consent types and
  /// their default (not-yet-granted) status.
  static StrideConsentRegistry privacyConsent({
    int lastUpdatedAtMs = 0,
  }) {
    final env = _bindings.privacyConsent({
      'last_updated_ms': lastUpdatedAtMs,
    });
    return StrideConsentRegistry.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns all data disclosures (location, wearable, AI, music,
  /// account, device).
  static List<StrideDataDisclosure> privacyDisclosures() {
    final env = _bindings.privacyDisclosures({});
    final list = unwrapEnvelope(env) as List<dynamic>;
    return list
        .map((e) => StrideDataDisclosure.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  /// Computes and returns the privacy compliance status by checking all
  /// privacy components.
  static StridePrivacyComplianceStatus privacyCompliance({
    int lastUpdatedAtMs = 0,
    bool healthAcknowledged = false,
  }) {
    final env = _bindings.privacyCompliance({
      'last_updated_ms': lastUpdatedAtMs,
      'health_acknowledged': healthAcknowledged,
    });
    return StridePrivacyComplianceStatus.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the Google Play Data Safety form (data type × purpose ×
  /// sharing matrix).
  static StrideDataSafetyForm privacyDataSafety({
    int lastUpdatedAtMs = 0,
  }) {
    final env = _bindings.privacyDataSafety({
      'last_updated_ms': lastUpdatedAtMs,
    });
    return StrideDataSafetyForm.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the app identity — the immutable store-level identity
  /// (app name, package ID, version, SDK levels).
  static StrideAppIdentity releaseIdentity() {
    final env = _bindings.releaseIdentity({});
    return StrideAppIdentity.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Computes and returns the release readiness status from all
  /// components (identity, signing, bundle, versioning, assets,
  /// listing, permissions, reviewer, stack, data safety).
  static StrideReleaseReadinessStatus releaseReadiness({
    bool dataSafetySubmitted = false,
  }) {
    final env = _bindings.releaseReadiness({
      'data_safety_submitted': dataSafetySubmitted,
    });
    return StrideReleaseReadinessStatus.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the permission declarations for the app, including the
  /// background-location justification text required by Google Play.
  static StridePermissionDeclarations releasePermissions() {
    final env = _bindings.releasePermissions({});
    return StridePermissionDeclarations.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the staged rollout plan — the internal, closed, and
  /// production testing track configuration with rollout percentages.
  static StrideStagedRolloutPlan releaseRollout() {
    final env = _bindings.releaseRollout({});
    return StrideStagedRolloutPlan.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }

  /// Returns the production stack inventory — the list of backend
  /// services, SDKs, and infrastructure components used by the app.
  static StrideProductionStack releaseStack() {
    final env = _bindings.releaseStack({});
    return StrideProductionStack.fromJson(
        unwrapEnvelope(env) as Map<String, dynamic>);
  }
}
