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
}
