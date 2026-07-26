/// Dart-side mirrors of the JSON shapes produced by the Rust
/// `stride_engine` crate (see `rust/stride_engine/src/models/*.rs`).
///
/// These are intentionally plain, immutable data classes built from the
/// decoded JSON maps the FFI boundary hands back — they exist purely so
/// the rest of the Flutter app can work with typed objects instead of
/// raw `Map<String, dynamic>` everywhere `StrideEngineBindings` is used.
library stride_models;

class StrideWorkoutSession {
  final String workoutId;
  final String userId;
  final String requestedActivityType;
  final String resolvedActivityType;
  final String state;
  final int startedAt;
  final int? endedAt;

  final double totalDistanceMeters;
  final int elapsedMs;
  final int activeMs;
  final int movingMs;
  final int pausedMs;

  final double currentSpeedMps;
  final double averageSpeedMps;
  final double maxSpeedMps;

  final double currentPaceSecPerKm;
  final double averagePaceSecPerKm;
  final double fastestPaceSecPerKm;

  final int stepCount;
  final double currentCadenceSpm;
  final double averageCadenceSpm;

  final double elevationGainMeters;
  final double elevationLossMeters;
  final double? minAltitudeMeters;
  final double? maxAltitudeMeters;

  final double caloriesEstimated;
  final String calorieEstimateMethod;

  final String movementState;
  final bool isAutoPaused;

  final int? lastPointAt;
  final double weightKg;
  final int? averageHeartRate;

  StrideWorkoutSession.fromJson(Map<String, dynamic> j)
      : workoutId = j['workout_id'] as String,
        userId = j['user_id'] as String,
        requestedActivityType = j['requested_activity_type'] as String,
        resolvedActivityType = j['resolved_activity_type'] as String,
        state = j['state'] as String,
        startedAt = j['started_at'] as int,
        endedAt = j['ended_at'] as int?,
        totalDistanceMeters = (j['total_distance_meters'] as num).toDouble(),
        elapsedMs = j['elapsed_ms'] as int,
        activeMs = j['active_ms'] as int,
        movingMs = j['moving_ms'] as int,
        pausedMs = j['paused_ms'] as int,
        currentSpeedMps = (j['current_speed_mps'] as num).toDouble(),
        averageSpeedMps = (j['average_speed_mps'] as num).toDouble(),
        maxSpeedMps = (j['max_speed_mps'] as num).toDouble(),
        currentPaceSecPerKm = (j['current_pace_sec_per_km'] as num).toDouble(),
        averagePaceSecPerKm =
            (j['average_pace_sec_per_km'] as num).toDouble(),
        fastestPaceSecPerKm =
            (j['fastest_pace_sec_per_km'] as num).toDouble(),
        stepCount = j['step_count'] as int,
        currentCadenceSpm = (j['current_cadence_spm'] as num).toDouble(),
        averageCadenceSpm = (j['average_cadence_spm'] as num).toDouble(),
        elevationGainMeters =
            (j['elevation_gain_meters'] as num).toDouble(),
        elevationLossMeters =
            (j['elevation_loss_meters'] as num).toDouble(),
        minAltitudeMeters = (j['min_altitude_meters'] as num?)?.toDouble(),
        maxAltitudeMeters = (j['max_altitude_meters'] as num?)?.toDouble(),
        caloriesEstimated = (j['calories_estimated'] as num).toDouble(),
        calorieEstimateMethod = j['calorie_estimate_method'] as String,
        movementState = j['movement_state'] as String,
        isAutoPaused = j['is_auto_paused'] as bool,
        lastPointAt = j['last_point_at'] as int?,
        weightKg = (j['weight_kg'] as num).toDouble(),
        averageHeartRate = j['average_heart_rate'] as int?;
}

class StrideWorkoutSplit {
  final int splitNumber;
  final double distanceMeters;
  final int durationMs;
  final double averagePaceSecPerKm;
  final double averageSpeedMps;
  final double elevationGainMeters;
  final double elevationLossMeters;
  final double? averageHeartRateBpm;
  final double averageCadenceSpm;
  final double caloriesEstimated;

  StrideWorkoutSplit.fromJson(Map<String, dynamic> j)
      : splitNumber = j['split_number'] as int,
        distanceMeters = (j['distance_meters'] as num).toDouble(),
        durationMs = j['duration_ms'] as int,
        averagePaceSecPerKm =
            (j['average_pace_sec_per_km'] as num).toDouble(),
        averageSpeedMps = (j['average_speed_mps'] as num).toDouble(),
        elevationGainMeters =
            (j['elevation_gain_meters'] as num).toDouble(),
        elevationLossMeters =
            (j['elevation_loss_meters'] as num).toDouble(),
        averageHeartRateBpm =
            (j['average_heart_rate_bpm'] as num?)?.toDouble(),
        averageCadenceSpm = (j['average_cadence_spm'] as num).toDouble(),
        caloriesEstimated = (j['calories_estimated'] as num).toDouble();
}

class StrideCoachingEvent {
  final String workoutId;
  final String eventType;
  final int occurredAt;
  final Map<String, dynamic>? data;

  StrideCoachingEvent.fromJson(Map<String, dynamic> j)
      : workoutId = j['workout_id'] as String,
        eventType = j['event_type'] as String,
        occurredAt = j['occurred_at'] as int,
        data = j['data'] as Map<String, dynamic>?;
}

class StrideGoalProgress {
  final String goalType;
  final double targetValue;
  final double currentValue;
  final double percentComplete;
  final double remainingValue;
  final bool isComplete;
  final double? paceNeededSecPerKm;

  StrideGoalProgress.fromJson(Map<String, dynamic> j)
      : goalType = j['goal_type'] as String,
        targetValue = (j['target_value'] as num).toDouble(),
        currentValue = (j['current_value'] as num).toDouble(),
        percentComplete = (j['percent_complete'] as num).toDouble(),
        remainingValue = (j['remaining_value'] as num).toDouble(),
        isComplete = j['is_complete'] as bool,
        paceNeededSecPerKm =
            (j['pace_needed_sec_per_km'] as num?)?.toDouble();
}

/// Everything returned from `add_location_sample` / `tick` — the live
/// snapshot the workout UI redraws itself from on every update.
class StrideLiveUpdate {
  final StrideWorkoutSession session;
  final List<StrideWorkoutSplit> newlyCompletedSplits;
  final List<StrideCoachingEvent> coachingEvents;
  final StrideGoalProgress? goalProgress;
  final String gpsQuality;
  final bool? lastPointAccepted;
  final String? lastRejectionReason;

  StrideLiveUpdate.fromJson(Map<String, dynamic> j)
      : session =
            StrideWorkoutSession.fromJson(j['session'] as Map<String, dynamic>),
        newlyCompletedSplits = (j['newly_completed_splits'] as List)
            .map((e) => StrideWorkoutSplit.fromJson(e as Map<String, dynamic>))
            .toList(),
        coachingEvents = (j['coaching_events'] as List)
            .map((e) => StrideCoachingEvent.fromJson(e as Map<String, dynamic>))
            .toList(),
        goalProgress = j['goal_progress'] != null
            ? StrideGoalProgress.fromJson(
                j['goal_progress'] as Map<String, dynamic>)
            : null,
        gpsQuality = j['gps_quality'] as String,
        lastPointAccepted = j['last_point_accepted'] as bool?,
        lastRejectionReason = j['last_rejection_reason'] as String?;
}

class StrideWorkoutSummary {
  final String workoutId;
  final String userId;
  final String activityType;
  final int startedAt;
  final int endedAt;
  final int totalDurationMs;
  final int movingMs;
  final int pausedMs;
  final double distanceMeters;
  final double averagePaceSecPerKm;
  final double averageSpeedMps;
  final double maxSpeedMps;
  final double caloriesEstimated;
  final String calorieEstimateMethod;
  final int stepCount;
  final double averageCadenceSpm;
  final double? averageHeartRateBpm;
  final int? maxHeartRateBpm;
  final int? minHeartRateBpm;
  final double elevationGainMeters;
  final double elevationLossMeters;
  final List<StrideWorkoutSplit> splits;
  final double? startLatitude;
  final double? startLongitude;
  final double? endLatitude;
  final double? endLongitude;
  final String? encodedPolyline;
  final StrideGoalProgress? goalResult;
  final bool hasFlaggedSegments;
  final List<String> validationWarnings;
  final bool isBlocked;
  final List<String> blockReasons;

  StrideWorkoutSummary.fromJson(Map<String, dynamic> j)
      : workoutId = j['workout_id'] as String,
        userId = j['user_id'] as String,
        activityType = j['activity_type'] as String,
        startedAt = j['started_at'] as int,
        endedAt = j['ended_at'] as int,
        totalDurationMs = j['total_duration_ms'] as int,
        movingMs = j['moving_ms'] as int,
        pausedMs = j['paused_ms'] as int,
        distanceMeters = (j['distance_meters'] as num).toDouble(),
        averagePaceSecPerKm =
            (j['average_pace_sec_per_km'] as num).toDouble(),
        averageSpeedMps = (j['average_speed_mps'] as num).toDouble(),
        maxSpeedMps = (j['max_speed_mps'] as num).toDouble(),
        caloriesEstimated = (j['calories_estimated'] as num).toDouble(),
        calorieEstimateMethod = j['calorie_estimate_method'] as String,
        stepCount = j['step_count'] as int,
        averageCadenceSpm = (j['average_cadence_spm'] as num).toDouble(),
        averageHeartRateBpm =
            (j['average_heart_rate_bpm'] as num?)?.toDouble(),
        maxHeartRateBpm = j['max_heart_rate_bpm'] as int?,
        minHeartRateBpm = j['min_heart_rate_bpm'] as int?,
        elevationGainMeters =
            (j['elevation_gain_meters'] as num).toDouble(),
        elevationLossMeters =
            (j['elevation_loss_meters'] as num).toDouble(),
        splits = (j['splits'] as List)
            .map((e) => StrideWorkoutSplit.fromJson(e as Map<String, dynamic>))
            .toList(),
        startLatitude = (j['start_latitude'] as num?)?.toDouble(),
        startLongitude = (j['start_longitude'] as num?)?.toDouble(),
        endLatitude = (j['end_latitude'] as num?)?.toDouble(),
        endLongitude = (j['end_longitude'] as num?)?.toDouble(),
        encodedPolyline = j['encoded_polyline'] as String?,
        goalResult = j['goal_result'] != null
            ? StrideGoalProgress.fromJson(
                j['goal_result'] as Map<String, dynamic>)
            : null,
        hasFlaggedSegments = j['has_flagged_segments'] as bool,
        validationWarnings = (j['validation_warnings'] as List)
            .map((e) => e as String)
            .toList(),
        isBlocked = j['is_blocked'] as bool? ?? false,
        blockReasons = ((j['block_reasons'] as List?) ?? const [])
            .map((e) => e as String)
            .toList();

  /// Serializes this summary back to the JSON shape the Rust engine
  /// expects as input to `stride_detect_personal_records` /
  /// `stride_detect_achievements` (both take a `WorkoutSummary`).
  Map<String, dynamic> toJson() => {
        'workout_id': workoutId,
        'user_id': userId,
        'activity_type': activityType,
        'started_at': startedAt,
        'ended_at': endedAt,
        'total_duration_ms': totalDurationMs,
        'moving_ms': movingMs,
        'paused_ms': pausedMs,
        'distance_meters': distanceMeters,
        'average_pace_sec_per_km': averagePaceSecPerKm,
        'average_speed_mps': averageSpeedMps,
        'max_speed_mps': maxSpeedMps,
        'calories_estimated': caloriesEstimated,
        'calorie_estimate_method': calorieEstimateMethod,
        'step_count': stepCount,
        'average_cadence_spm': averageCadenceSpm,
        'average_heart_rate_bpm': averageHeartRateBpm,
        'max_heart_rate_bpm': maxHeartRateBpm,
        'min_heart_rate_bpm': minHeartRateBpm,
        'elevation_gain_meters': elevationGainMeters,
        'elevation_loss_meters': elevationLossMeters,
        'splits': const [], // splits are not re-serialized round-trip here
        'start_latitude': startLatitude,
        'start_longitude': startLongitude,
        'end_latitude': endLatitude,
        'end_longitude': endLongitude,
        'encoded_polyline': encodedPolyline,
        'goal_result': null,
        'has_flagged_segments': hasFlaggedSegments,
        'validation_warnings': validationWarnings,
        'is_blocked': isBlocked,
        'block_reasons': blockReasons,
      };
}

/// Mirrors `PersonalRecordType` in `engine/personal_records.rs`.
enum StridePersonalRecordType {
  fastestMile,
  fastestKilometer,
  longestDistance,
  longestDuration,
  greatestElevationGain,
  highestAveragePace,
  mostCalories,
}

StridePersonalRecordType _personalRecordTypeFromJson(String raw) {
  switch (raw) {
    case 'fastest_mile':
      return StridePersonalRecordType.fastestMile;
    case 'fastest_kilometer':
      return StridePersonalRecordType.fastestKilometer;
    case 'longest_distance':
      return StridePersonalRecordType.longestDistance;
    case 'longest_duration':
      return StridePersonalRecordType.longestDuration;
    case 'greatest_elevation_gain':
      return StridePersonalRecordType.greatestElevationGain;
    case 'highest_average_pace':
      return StridePersonalRecordType.highestAveragePace;
    case 'most_calories':
      return StridePersonalRecordType.mostCalories;
    default:
      throw ArgumentError('Unknown personal record type: $raw');
  }
}

/// Mirrors `PersonalRecord` in `models/summary.rs`. Returned by
/// [StrideEngineClient.detectPersonalRecords].
class StridePersonalRecord {
  final StridePersonalRecordType recordType;
  final double value;
  final String workoutId;
  final int achievedAt;

  StridePersonalRecord.fromJson(Map<String, dynamic> j)
      : recordType = _personalRecordTypeFromJson(j['record_type'] as String),
        value = (j['value'] as num).toDouble(),
        workoutId = j['workout_id'] as String,
        achievedAt = j['achieved_at'] as int;
}

/// Mirrors `ValidationResult` in `engine/validation.rs`. Returned by
/// [StrideEngineClient.validate].
class StrideValidationResult {
  final bool isBlocked;
  final List<String> blockReasons;
  final List<String> warnings;

  StrideValidationResult.fromJson(Map<String, dynamic> j)
      : isBlocked = j['is_blocked'] as bool,
        blockReasons = (j['block_reasons'] as List)
            .map((e) => e as String)
            .toList(),
        warnings =
            (j['warnings'] as List).map((e) => e as String).toList();
}

/// Mirrors `RequiredAction` in `engine/permissions.rs`.
enum StrideRequiredAction {
  none,
  showRationaleThenRequest,
  requestDirectly,
  openAppSettings,
  useLimitedFallback,
  pauseAndPromptReRequest,
}

StrideRequiredAction strideRequiredActionFromJson(String raw) {
  switch (raw) {
    case 'none':
      return StrideRequiredAction.none;
    case 'show_rationale_then_request':
      return StrideRequiredAction.showRationaleThenRequest;
    case 'request_directly':
      return StrideRequiredAction.requestDirectly;
    case 'open_app_settings':
      return StrideRequiredAction.openAppSettings;
    case 'use_limited_fallback':
      return StrideRequiredAction.useLimitedFallback;
    case 'pause_and_prompt_re_request':
      return StrideRequiredAction.pauseAndPromptReRequest;
    default:
      throw ArgumentError('Unknown required action: $raw');
  }
}

/// Mirrors `WorkoutStartCapability` in `engine/permissions.rs`.
enum StrideWorkoutStartCapability { fullFeatured, foregroundOnlyFallback, blocked }

StrideWorkoutStartCapability strideWorkoutStartCapabilityFromJson(String raw) {
  switch (raw) {
    case 'full_featured':
      return StrideWorkoutStartCapability.fullFeatured;
    case 'foreground_only_fallback':
      return StrideWorkoutStartCapability.foregroundOnlyFallback;
    case 'blocked':
      return StrideWorkoutStartCapability.blocked;
    default:
      throw ArgumentError('Unknown start capability: $raw');
  }
}

/// Mirrors `SamplingProfile` in `engine/battery.rs`. Returned by
/// [StrideEngineClient.chooseSamplingProfile].
class StrideSamplingProfile {
  final int gpsIntervalMs;
  final double gpsMinDistanceM;
  final bool highAccuracy;
  final int sensorIntervalMs;
  final int dbBatchFlushMs;

  StrideSamplingProfile.fromJson(Map<String, dynamic> j)
      : gpsIntervalMs = j['gps_interval_ms'] as int,
        gpsMinDistanceM = (j['gps_min_distance_m'] as num).toDouble(),
        highAccuracy = j['high_accuracy'] as bool,
        sensorIntervalMs = j['sensor_interval_ms'] as int,
        dbBatchFlushMs = j['db_batch_flush_ms'] as int;
}

/// Mirrors `AchievementType` in `engine/achievements.rs`.
enum StrideAchievementType {
  firstWalk,
  firstRun,
  firstMile,
  first5k,
  sevenDayStreak,
  tenTotalMiles,
  fiftyTotalMiles,
  newPaceRecord,
  weeklyGoalCompleted,
}

StrideAchievementType _achievementTypeFromJson(String raw) {
  switch (raw) {
    case 'first_walk':
      return StrideAchievementType.firstWalk;
    case 'first_run':
      return StrideAchievementType.firstRun;
    case 'first_mile':
      return StrideAchievementType.firstMile;
    case 'first5k':
      return StrideAchievementType.first5k;
    case 'seven_day_streak':
      return StrideAchievementType.sevenDayStreak;
    case 'ten_total_miles':
      return StrideAchievementType.tenTotalMiles;
    case 'fifty_total_miles':
      return StrideAchievementType.fiftyTotalMiles;
    case 'new_pace_record':
      return StrideAchievementType.newPaceRecord;
    case 'weekly_goal_completed':
      return StrideAchievementType.weeklyGoalCompleted;
    default:
      throw ArgumentError('Unknown achievement type: $raw');
  }
}

/// Mirrors `AchievementEvent` in `engine/achievements.rs`. Returned by
/// [StrideEngineClient.detectAchievements].
class StrideAchievementEvent {
  final StrideAchievementType achievementType;
  final String workoutId;
  final int achievedAt;

  StrideAchievementEvent.fromJson(Map<String, dynamic> j)
      : achievementType = _achievementTypeFromJson(j['achievement_type'] as String),
        workoutId = j['workout_id'] as String,
        achievedAt = j['achieved_at'] as int;
}

/// Mirrors `PriorBests` in `engine/personal_records.rs`. All fields are
/// optional \u2014 `null` means "no prior record", so the next qualifying
/// workout will unconditionally set that record. Constructed by Dart from
/// locally/cloud-persisted workout history.
class StridePriorBests {
  const StridePriorBests({
    this.fastestMileSecPerKm,
    this.fastestKilometerSecPerKm,
    this.longestDistanceMeters,
    this.longestDurationMs,
    this.greatestElevationGainMeters,
    this.highestAveragePaceSecPerKm,
    this.mostCalories,
  });

  final double? fastestMileSecPerKm;
  final double? fastestKilometerSecPerKm;
  final double? longestDistanceMeters;
  final int? longestDurationMs;
  final double? greatestElevationGainMeters;
  final double? highestAveragePaceSecPerKm;
  final double? mostCalories;

  Map<String, dynamic> toJson() => {
        'fastest_mile_sec_per_km': fastestMileSecPerKm,
        'fastest_kilometer_sec_per_km': fastestKilometerSecPerKm,
        'longest_distance_meters': longestDistanceMeters,
        'longest_duration_ms': longestDurationMs,
        'greatest_elevation_gain_meters': greatestElevationGainMeters,
        'highest_average_pace_sec_per_km': highestAveragePaceSecPerKm,
        'most_calories': mostCalories,
      };
}

/// Mirrors `PermissionState` in `engine/permissions.rs`.
enum StridePermissionState {
  notRequested,
  granted,
  denied,
  permanentlyDenied,
  revokedDuringUse,
}

/// Mirrors `PermissionContext` in `engine/permissions.rs`.
enum StridePermissionContext { initial, deniedOnce, duringActiveWorkout }

/// Mirrors `WorkoutActivityLevel` in `engine/battery.rs`.
enum StrideWorkoutActivityLevel { active, paused, autoPaused }

/// Mirrors `ConversionKind` in `engine/units.rs`.
enum StrideConversionKind {
  metersToMiles,
  milesToMeters,
  metersToFeet,
  feetToMeters,
  kgToLb,
  lbToKg,
  mpsToKmh,
  kmhToMps,
  mpsToMph,
  mphToMps,
  paceSecPerKmToSecPerMile,
  paceSecPerMileToSecPerKm,
}

/// Mirrors `AchievementHistory` in `engine/achievements.rs`. Dart owns
/// loading this from local/cloud storage and persisting
/// [alreadyAwarded] (merged with any newly-returned events) afterward.
class StrideAchievementHistory {
  const StrideAchievementHistory({
    this.totalWalks = 0,
    this.totalRuns = 0,
    this.lifetimeDistanceMetersBefore = 0.0,
    this.currentStreakDays = 0,
    this.alreadyAwarded = const <StrideAchievementType>{},
  });

  final int totalWalks;
  final int totalRuns;
  final double lifetimeDistanceMetersBefore;
  final int currentStreakDays;
  final Set<StrideAchievementType> alreadyAwarded;

  Map<String, dynamic> toJson() => {
        'total_walks': totalWalks,
        'total_runs': totalRuns,
        'lifetime_distance_meters_before': lifetimeDistanceMetersBefore,
        'current_streak_days': currentStreakDays,
        'already_awarded':
            alreadyAwarded.map(_achievementTypeToJson).toList(),
      };
}

String _achievementTypeToJson(StrideAchievementType t) {
  switch (t) {
    case StrideAchievementType.firstWalk:
      return 'first_walk';
    case StrideAchievementType.firstRun:
      return 'first_run';
    case StrideAchievementType.firstMile:
      return 'first_mile';
    case StrideAchievementType.first5k:
      return 'first5k';
    case StrideAchievementType.sevenDayStreak:
      return 'seven_day_streak';
    case StrideAchievementType.tenTotalMiles:
      return 'ten_total_miles';
    case StrideAchievementType.fiftyTotalMiles:
      return 'fifty_total_miles';
    case StrideAchievementType.newPaceRecord:
      return 'new_pace_record';
    case StrideAchievementType.weeklyGoalCompleted:
      return 'weekly_goal_completed';
  }
}

/// Mirrors `RecoveryOption` in `engine/recovery.rs`.
enum StrideRecoveryOption { resumeWorkout, finishAndSave, discardWorkout }

StrideRecoveryOption _recoveryOptionFromJson(String raw) {
  switch (raw) {
    case 'resume_workout':
      return StrideRecoveryOption.resumeWorkout;
    case 'finish_and_save':
      return StrideRecoveryOption.finishAndSave;
    case 'discard_workout':
      return StrideRecoveryOption.discardWorkout;
    default:
      throw ArgumentError('Unknown recovery option: $raw');
  }
}

/// Mirrors `RecoveryDecision` in `engine/recovery.rs`. Returned by
/// [StrideEngineClient.evaluateRecovery] to describe what recovery UI to
/// present after finding a leftover in-progress workout at app startup.
class StrideRecoveryDecision {
  final List<StrideRecoveryOption> offeredOptions;
  final StrideRecoveryOption recommendedOption;
  final String reason;

  StrideRecoveryDecision.fromJson(Map<String, dynamic> j)
      : offeredOptions = (j['offered_options'] as List)
            .map((e) => _recoveryOptionFromJson(e as String))
            .toList(),
        recommendedOption =
            _recoveryOptionFromJson(j['recommended_option'] as String),
        reason = j['reason'] as String;
}

// ─── Cloud synchronization models (spec section 3) ───────────────────
// Mirrors the types in `engine/sync.rs`. These are used by
// [StrideEngineClient]'s sync helper methods.

/// What kind of sync operation is being performed.
enum StrideSyncOperation {
  uploadSummary,
  uploadRoute,
  deleteWorkout,
  downloadUpdates,
}

/// The outcome of a sync attempt for a single item.
enum StrideSyncAttemptResult {
  success,
  retryableFailure,
  permanentFailure,
  conflict,
}

String _syncAttemptResultToJson(StrideSyncAttemptResult r) {
  switch (r) {
    case StrideSyncAttemptResult.success:
      return 'success';
    case StrideSyncAttemptResult.retryableFailure:
      return 'retryable_failure';
    case StrideSyncAttemptResult.permanentFailure:
      return 'permanent_failure';
    case StrideSyncAttemptResult.conflict:
      return 'conflict';
  }
}

/// Strategy for resolving a sync conflict.
enum StrideConflictResolutionStrategy {
  lastWriteWins,
  serverAuthoritative,
  localWins,
  manualMerge,
}

String _conflictStrategyToJson(StrideConflictResolutionStrategy s) {
  switch (s) {
    case StrideConflictResolutionStrategy.lastWriteWins:
      return 'last_write_wins';
    case StrideConflictResolutionStrategy.serverAuthoritative:
      return 'server_authoritative';
    case StrideConflictResolutionStrategy.localWins:
      return 'local_wins';
    case StrideConflictResolutionStrategy.manualMerge:
      return 'manual_merge';
  }
}

/// The decision returned by conflict resolution.
enum StrideConflictDecision {
  keepLocal,
  keepCloud,
  noConflict,
  requireManualMerge,
}

StrideConflictDecision _conflictDecisionFromJson(String raw) {
  switch (raw) {
    case 'keep_local':
      return StrideConflictDecision.keepLocal;
    case 'keep_cloud':
      return StrideConflictDecision.keepCloud;
    case 'no_conflict':
      return StrideConflictDecision.noConflict;
    case 'require_manual_merge':
      return StrideConflictDecision.requireManualMerge;
    default:
      throw ArgumentError('Unknown conflict decision: $raw');
  }
}

/// Whether to insert, update, or skip a workout upsert.
enum StrideUpsertDecision {
  insert,
  update,
  skip,
}

StrideUpsertDecision _upsertDecisionFromJson(String raw) {
  switch (raw) {
    case 'insert':
      return StrideUpsertDecision.insert;
    case 'update':
      return StrideUpsertDecision.update;
    case 'skip':
      return StrideUpsertDecision.skip;
    default:
      throw ArgumentError('Unknown upsert decision: $raw');
  }
}

/// What action the current device should take for a workout in
/// device-to-device sync.
enum StrideDeviceSyncAction {
  upload,
  download,
  noAction,
}

StrideDeviceSyncAction _deviceSyncActionFromJson(String raw) {
  switch (raw) {
    case 'upload':
      return StrideDeviceSyncAction.upload;
    case 'download':
      return StrideDeviceSyncAction.download;
    case 'no_action':
      return StrideDeviceSyncAction.noAction;
    default:
      throw ArgumentError('Unknown device sync action: $raw');
  }
}

/// State of a deletion tombstone in the pending_deletions queue.
enum StrideTombstoneState {
  pending,
  deleting,
  synced,
  failed,
}

String _tombstoneStateToJson(StrideTombstoneState s) {
  switch (s) {
    case StrideTombstoneState.pending:
      return 'pending';
    case StrideTombstoneState.deleting:
      return 'deleting';
    case StrideTombstoneState.synced:
      return 'synced';
    case StrideTombstoneState.failed:
      return 'failed';
  }
}

/// Result of `stride_decide_sync_retry`: whether to retry and the delay.
class StrideSyncRetryDecision {
  final bool shouldRetry;
  final int? delayMs;

  StrideSyncRetryDecision.fromJson(Map<String, dynamic> j)
      : shouldRetry = j['retry'] as bool,
        delayMs = j['delay_ms'] as int?;
}

// =========================================================================
// Route file format & storage layout (spec section 4)
// =========================================================================

/// Which on-disk / Cloud-Storage format the full-resolution route uses.
enum StrideRouteFileFormat {
  gpx,
  compressedBinary,
  polylineOnly,
}

String _routeFileFormatToJson(StrideRouteFileFormat f) {
  switch (f) {
    case StrideRouteFileFormat.gpx:
      return 'gpx';
    case StrideRouteFileFormat.compressedBinary:
      return 'compressed_binary';
    case StrideRouteFileFormat.polylineOnly:
      return 'polyline_only';
  }
}

StrideRouteFileFormat _routeFileFormatFromJson(String raw) {
  switch (raw) {
    case 'gpx':
      return StrideRouteFileFormat.gpx;
    case 'compressed_binary':
      return StrideRouteFileFormat.compressedBinary;
    case 'polyline_only':
      return StrideRouteFileFormat.polylineOnly;
    default:
      throw ArgumentError('Unknown route file format: $raw');
  }
}

/// Sync state of a route file, tracked independently from the Firestore
/// summary document's sync state. A route can be uploaded to Cloud
/// Storage while the summary row is still pending, or vice versa.
enum StrideRouteFileSyncState {
  localOnly,
  uploading,
  uploaded,
  failed,
}

String _routeFileSyncStateToJson(StrideRouteFileSyncState s) {
  switch (s) {
    case StrideRouteFileSyncState.localOnly:
      return 'local_only';
    case StrideRouteFileSyncState.uploading:
      return 'uploading';
    case StrideRouteFileSyncState.uploaded:
      return 'uploaded';
    case StrideRouteFileSyncState.failed:
      return 'failed';
  }
}

StrideRouteFileSyncState _routeFileSyncStateFromJson(String raw) {
  switch (raw) {
    case 'local_only':
      return StrideRouteFileSyncState.localOnly;
    case 'uploading':
      return StrideRouteFileSyncState.uploading;
    case 'uploaded':
      return StrideRouteFileSyncState.uploaded;
    case 'failed':
      return StrideRouteFileSyncState.failed;
    default:
      throw ArgumentError('Unknown route file sync state: $raw');
  }
}

/// Metadata about a route file — its Cloud Storage path, format, size,
/// point count, device source, and sync state. Stored in the local
/// SQLite `route_files` table alongside the raw points.
class StrideRouteFileMetadata {
  final String? filePath;
  final StrideRouteFileFormat format;
  final int sizeBytes;
  final int pointCount;
  final String deviceSource;
  final StrideRouteFileSyncState syncState;
  final int createdAt;
  final int updatedAt;

  StrideRouteFileMetadata.fromJson(Map<String, dynamic> j)
      : filePath = j['file_path'] as String?,
        format = _routeFileFormatFromJson(j['format'] as String),
        sizeBytes = j['size_bytes'] as int,
        pointCount = j['point_count'] as int,
        deviceSource = j['device_source'] as String,
        syncState = _routeFileSyncStateFromJson(j['sync_state'] as String),
        createdAt = j['created_at'] as int,
        updatedAt = j['updated_at'] as int;

  Map<String, dynamic> toJson() => {
        'file_path': filePath,
        'format': _routeFileFormatToJson(format),
        'size_bytes': sizeBytes,
        'point_count': pointCount,
        'device_source': deviceSource,
        'sync_state': _routeFileSyncStateToJson(syncState),
        'created_at': createdAt,
        'updated_at': updatedAt,
      };
}

/// Sync state of a Firestore summary document (mirrors the Rust
/// `SyncQueueState` enum). Reused for the `sync_state` field on
/// `StrideRouteSummary`.
enum StrideSyncQueueState {
  pending,
  uploading,
  synced,
  failed,
}

String _syncQueueStateToJson(StrideSyncQueueState s) {
  switch (s) {
    case StrideSyncQueueState.pending:
      return 'pending';
    case StrideSyncQueueState.uploading:
      return 'uploading';
    case StrideSyncQueueState.synced:
      return 'synced';
    case StrideSyncQueueState.failed:
      return 'failed';
  }
}

StrideSyncQueueState _syncQueueStateFromJson(String raw) {
  switch (raw) {
    case 'pending':
      return StrideSyncQueueState.pending;
    case 'uploading':
      return StrideSyncQueueState.uploading;
    case 'synced':
      return StrideSyncQueueState.synced;
    case 'failed':
      return StrideSyncQueueState.failed;
    default:
      throw ArgumentError('Unknown sync queue state: $raw');
  }
}

/// Where a GPS fix came from (mirrors the Rust `LocationSource` enum).
/// Used by `dominantLocationSource` to attribute a route to its
/// primary recording source.
enum StrideLocationSource {
  phoneGps,
  wearOs,
  healthConnect,
  manual,
  serverCorrected,
  estimated,
}

String _locationSourceToJson(StrideLocationSource s) {
  switch (s) {
    case StrideLocationSource.phoneGps:
      return 'phone_gps';
    case StrideLocationSource.wearOs:
      return 'wear_os';
    case StrideLocationSource.healthConnect:
      return 'health_connect';
    case StrideLocationSource.manual:
      return 'manual';
    case StrideLocationSource.serverCorrected:
      return 'server_corrected';
    case StrideLocationSource.estimated:
      return 'estimated';
  }
}

StrideLocationSource? _locationSourceFromJson(String? raw) {
  if (raw == null) return null;
  switch (raw) {
    case 'phone_gps':
      return StrideLocationSource.phoneGps;
    case 'wear_os':
      return StrideLocationSource.wearOs;
    case 'health_connect':
      return StrideLocationSource.healthConnect;
    case 'manual':
      return StrideLocationSource.manual;
    case 'server_corrected':
      return StrideLocationSource.serverCorrected;
    case 'estimated':
      return StrideLocationSource.estimated;
    default:
      throw ArgumentError('Unknown location source: $raw');
  }
}

/// Activity type (mirrors the Rust `ActivityType` enum). Used by
/// `StrideRouteSummary` for the `activity_type` field.
enum StrideActivityType {
  walk,
  run,
  hike,
  autoDetect,
}

String _activityTypeToJson(StrideActivityType a) {
  switch (a) {
    case StrideActivityType.walk:
      return 'walk';
    case StrideActivityType.run:
      return 'run';
    case StrideActivityType.hike:
      return 'hike';
    case StrideActivityType.autoDetect:
      return 'auto_detect';
  }
}

StrideActivityType _activityTypeFromJson(String raw) {
  switch (raw) {
    case 'walk':
      return StrideActivityType.walk;
    case 'run':
      return StrideActivityType.run;
    case 'hike':
      return StrideActivityType.hike;
    case 'auto_detect':
      return StrideActivityType.autoDetect;
    default:
      throw ArgumentError('Unknown activity type: $raw');
  }
}

/// A single GPS point, as deserialized from the Rust `WorkoutPoint`
/// JSON. Used by `serializeGpx` and `dominantLocationSource`.
class StrideWorkoutPoint {
  final String pointId;
  final String workoutId;
  final double latitude;
  final double longitude;
  final double? altitudeMeters;
  final double? accuracyMeters;
  final double? speedMetersPerSecond;
  final double? bearingDegrees;
  final int recordedAt;
  final StrideLocationSource source;
  final bool isMockLocation;
  final bool accepted;

  StrideWorkoutPoint.fromJson(Map<String, dynamic> j)
      : pointId = j['point_id'] as String,
        workoutId = j['workout_id'] as String,
        latitude = (j['latitude'] as num).toDouble(),
        longitude = (j['longitude'] as num).toDouble(),
        altitudeMeters = (j['altitude_meters'] as num?)?.toDouble(),
        accuracyMeters = (j['accuracy_meters'] as num?)?.toDouble(),
        speedMetersPerSecond =
            (j['speed_meters_per_second'] as num?)?.toDouble(),
        bearingDegrees = (j['bearing_degrees'] as num?)?.toDouble(),
        recordedAt = j['recorded_at'] as int,
        source = _locationSourceFromJson(j['source'] as String)!,
        isMockLocation = j['is_mock_location'] as bool? ?? false,
        accepted = j['accepted'] as bool? ?? false;

  Map<String, dynamic> toJson() => {
        'point_id': pointId,
        'workout_id': workoutId,
        'latitude': latitude,
        'longitude': longitude,
        'altitude_meters': altitudeMeters,
        'accuracy_meters': accuracyMeters,
        'speed_meters_per_second': speedMetersPerSecond,
        'bearing_degrees': bearingDegrees,
        'recorded_at': recordedAt,
        'source': _locationSourceToJson(source),
        'is_mock_location': isMockLocation,
        'accepted': accepted,
      };
}

/// Firestore-ready summary of a workout. This is the compact document
/// shape — every field the spec lists for the Firestore summary row,
/// with the full-resolution route reduced to start/end coordinates +
/// an encoded polyline + a route file path pointing at Cloud Storage.
class StrideRouteSummary {
  final String workoutId;
  final String userId;
  final StrideActivityType activityType;
  final int startedAt;
  final int endedAt;
  final int durationMs;
  final double distanceMeters;
  final double averagePaceSecPerKm;
  final double averageSpeedMps;
  final int steps;
  final double? averageHeartRateBpm;
  final int? maxHeartRateBpm;
  final double calories;
  final String calorieMethod;
  final double? startLatitude;
  final double? startLongitude;
  final double? endLatitude;
  final double? endLongitude;
  final String? encodedPolyline;
  final String? routeFilePath;
  final StrideRouteFileFormat routeFileFormat;
  final StrideSyncQueueState syncState;
  final String deviceSource;
  final int createdAt;
  final int updatedAt;

  StrideRouteSummary.fromJson(Map<String, dynamic> j)
      : workoutId = j['workout_id'] as String,
        userId = j['user_id'] as String,
        activityType = _activityTypeFromJson(j['activity_type'] as String),
        startedAt = j['started_at'] as int,
        endedAt = j['ended_at'] as int,
        durationMs = j['duration_ms'] as int,
        distanceMeters = (j['distance_meters'] as num).toDouble(),
        averagePaceSecPerKm =
            (j['average_pace_sec_per_km'] as num).toDouble(),
        averageSpeedMps = (j['average_speed_mps'] as num).toDouble(),
        steps = j['steps'] as int,
        averageHeartRateBpm =
            (j['average_heart_rate_bpm'] as num?)?.toDouble(),
        maxHeartRateBpm = j['max_heart_rate_bpm'] as int?,
        calories = (j['calories'] as num).toDouble(),
        calorieMethod = j['calorie_method'] as String,
        startLatitude = (j['start_latitude'] as num?)?.toDouble(),
        startLongitude = (j['start_longitude'] as num?)?.toDouble(),
        endLatitude = (j['end_latitude'] as num?)?.toDouble(),
        endLongitude = (j['end_longitude'] as num?)?.toDouble(),
        encodedPolyline = j['encoded_polyline'] as String?,
        routeFilePath = j['route_file_path'] as String?,
        routeFileFormat =
            _routeFileFormatFromJson(j['route_file_format'] as String),
        syncState = _syncQueueStateFromJson(j['sync_state'] as String),
        deviceSource = j['device_source'] as String,
        createdAt = j['created_at'] as int,
        updatedAt = j['updated_at'] as int;

  Map<String, dynamic> toJson() => {
        'workout_id': workoutId,
        'user_id': userId,
        'activity_type': _activityTypeToJson(activityType),
        'started_at': startedAt,
        'ended_at': endedAt,
        'duration_ms': durationMs,
        'distance_meters': distanceMeters,
        'average_pace_sec_per_km': averagePaceSecPerKm,
        'average_speed_mps': averageSpeedMps,
        'steps': steps,
        'average_heart_rate_bpm': averageHeartRateBpm,
        'max_heart_rate_bpm': maxHeartRateBpm,
        'calories': calories,
        'calorie_method': calorieMethod,
        'start_latitude': startLatitude,
        'start_longitude': startLongitude,
        'end_latitude': endLatitude,
        'end_longitude': endLongitude,
        'encoded_polyline': encodedPolyline,
        'route_file_path': routeFilePath,
        'route_file_format': _routeFileFormatToJson(routeFileFormat),
        'sync_state': _syncQueueStateToJson(syncState),
        'device_source': deviceSource,
        'created_at': createdAt,
        'updated_at': updatedAt,
      };
}

/// Result of `stride_validate_route_summary`: whether the summary is
/// valid for Firestore, and an error message if not.
class StrideRouteSummaryValidation {
  final bool valid;
  final String? error;

  StrideRouteSummaryValidation.fromJson(Map<String, dynamic> j)
      : valid = j['valid'] as bool,
        error = j['error'] as String?;
}

// ─────────────────────────────────────────────────────────────────────────────
// §5 Maps and location services
// ─────────────────────────────────────────────────────────────────────────────

/// Which tile layer the user is currently viewing.
enum StrideMapViewType {
  standard,
  satellite,
  hybrid,
  terrain;

  static StrideMapViewType fromJson(String s) {
    switch (s) {
      case 'standard':
        return StrideMapViewType.standard;
      case 'satellite':
        return StrideMapViewType.satellite;
      case 'hybrid':
        return StrideMapViewType.hybrid;
      case 'terrain':
        return StrideMapViewType.terrain;
      default:
        return StrideMapViewType.standard;
    }
  }

  String toJson() => name;
}

/// Which tile provider supplies tiles for a map view.
enum StrideTileProvider {
  openStreetMap,
  satelliteProvider,
  esri,
  custom;

  static StrideTileProvider fromJson(String s) {
    switch (s) {
      case 'open_street_map':
        return StrideTileProvider.openStreetMap;
      case 'satellite_provider':
        return StrideTileProvider.satelliteProvider;
      case 'esri':
        return StrideTileProvider.esri;
      case 'custom':
        return StrideTileProvider.custom;
      default:
        return StrideTileProvider.openStreetMap;
    }
  }

  String toJson() {
    switch (this) {
      case StrideTileProvider.openStreetMap:
        return 'open_street_map';
      case StrideTileProvider.satelliteProvider:
        return 'satellite_provider';
      case StrideTileProvider.esri:
        return 'esri';
      case StrideTileProvider.custom:
        return 'custom';
    }
  }
}

/// GPS accuracy quality level for the accuracy indicator.
enum StrideGpsAccuracyLevel {
  excellent,
  good,
  fair,
  poor,
  unavailable;

  static StrideGpsAccuracyLevel fromJson(String s) {
    switch (s) {
      case 'excellent':
        return StrideGpsAccuracyLevel.excellent;
      case 'good':
        return StrideGpsAccuracyLevel.good;
      case 'fair':
        return StrideGpsAccuracyLevel.fair;
      case 'poor':
        return StrideGpsAccuracyLevel.poor;
      case 'unavailable':
        return StrideGpsAccuracyLevel.unavailable;
      default:
        return StrideGpsAccuracyLevel.unavailable;
    }
  }

  String toJson() => name;
}

/// A tile coordinate (z, x, y) in the slippy map XYZ addressing scheme.
class StrideTileCoord {
  final int z;
  final int x;
  final int y;

  StrideTileCoord({required this.z, required this.x, required this.y});

  StrideTileCoord.fromJson(Map<String, dynamic> j)
      : z = j['z'] as int,
        x = j['x'] as int,
        y = j['y'] as int;

  Map<String, dynamic> toJson() => {'z': z, 'x': x, 'y': y};
}

/// A saved offline region manifest.
class StrideOfflineRegion {
  final String regionId;
  final String name;
  final double minLat;
  final double minLon;
  final double maxLat;
  final double maxLon;
  final int minZoom;
  final int maxZoom;
  final int tileCount;
  final int estimatedSizeBytes;
  final StrideTileProvider provider;
  final int downloadedAt;
  final int lastAccessedAt;

  StrideOfflineRegion({
    required this.regionId,
    required this.name,
    required this.minLat,
    required this.minLon,
    required this.maxLat,
    required this.maxLon,
    required this.minZoom,
    required this.maxZoom,
    required this.tileCount,
    required this.estimatedSizeBytes,
    required this.provider,
    required this.downloadedAt,
    required this.lastAccessedAt,
  });

  StrideOfflineRegion.fromJson(Map<String, dynamic> j)
      : regionId = j['region_id'] as String,
        name = j['name'] as String,
        minLat = (j['min_lat'] as num).toDouble(),
        minLon = (j['min_lon'] as num).toDouble(),
        maxLat = (j['max_lat'] as num).toDouble(),
        maxLon = (j['max_lon'] as num).toDouble(),
        minZoom = j['min_zoom'] as int,
        maxZoom = j['max_zoom'] as int,
        tileCount = j['tile_count'] as int,
        estimatedSizeBytes = j['estimated_size_bytes'] as int,
        provider = StrideTileProvider.fromJson(j['provider'] as String),
        downloadedAt = j['downloaded_at'] as int,
        lastAccessedAt = j['last_accessed_at'] as int;

  Map<String, dynamic> toJson() => {
        'region_id': regionId,
        'name': name,
        'min_lat': minLat,
        'min_lon': minLon,
        'max_lat': maxLat,
        'max_lon': maxLon,
        'min_zoom': minZoom,
        'max_zoom': maxZoom,
        'tile_count': tileCount,
        'estimated_size_bytes': estimatedSizeBytes,
        'provider': provider.toJson(),
        'downloaded_at': downloadedAt,
        'last_accessed_at': lastAccessedAt,
      };
}

/// A saved route the user has bookmarked for future reference.
class StrideSavedRoute {
  final String routeId;
  final String name;
  final String userId;
  final String encodedPolyline;
  final double startLat;
  final double startLon;
  final double distanceMeters;
  final int savedAt;

  StrideSavedRoute({
    required this.routeId,
    required this.name,
    required this.userId,
    required this.encodedPolyline,
    required this.startLat,
    required this.startLon,
    required this.distanceMeters,
    required this.savedAt,
  });

  StrideSavedRoute.fromJson(Map<String, dynamic> j)
      : routeId = j['route_id'] as String,
        name = j['name'] as String,
        userId = j['user_id'] as String,
        encodedPolyline = j['encoded_polyline'] as String,
        startLat = (j['start_lat'] as num).toDouble(),
        startLon = (j['start_lon'] as num).toDouble(),
        distanceMeters = (j['distance_meters'] as num).toDouble(),
        savedAt = j['saved_at'] as int;

  Map<String, dynamic> toJson() => {
        'route_id': routeId,
        'name': name,
        'user_id': userId,
        'encoded_polyline': encodedPolyline,
        'start_lat': startLat,
        'start_lon': startLon,
        'distance_meters': distanceMeters,
        'saved_at': savedAt,
      };
}

/// Result of validating a saved route.
class StrideSavedRouteValidation {
  final bool valid;
  final String? error;

  StrideSavedRouteValidation.fromJson(Map<String, dynamic> j)
      : valid = j['valid'] as bool,
        error = j['error'] as String?;
}

/// Result of a storage-availability check for offline download.
class StrideStorageAvailabilityResult {
  final bool available;
  final String? error;

  StrideStorageAvailabilityResult.fromJson(Map<String, dynamic> j)
      : available = j['available'] as bool,
        error = j['error'] as String?;
}

// ===========================================================================
// §6 — Authentication / account lifecycle models
// ===========================================================================

/// Which authentication provider the user signed in with.
enum StrideAuthProvider {
  emailPassword,
  google,
  apple,
  anonymous;

  static StrideAuthProvider fromJson(String s) {
    switch (s) {
      case 'email_password':
        return StrideAuthProvider.emailPassword;
      case 'google':
        return StrideAuthProvider.google;
      case 'apple':
        return StrideAuthProvider.apple;
      case 'anonymous':
        return StrideAuthProvider.anonymous;
      default:
        return StrideAuthProvider.emailPassword;
    }
  }

  String toJson() {
    switch (this) {
      case StrideAuthProvider.emailPassword:
        return 'email_password';
      case StrideAuthProvider.google:
        return 'google';
      case StrideAuthProvider.apple:
        return 'apple';
      case StrideAuthProvider.anonymous:
        return 'anonymous';
    }
  }
}

/// The state of an authentication session.
enum StrideSessionState {
  valid,
  refreshing,
  expired,
  noSession,
  revoked;

  static StrideSessionState fromJson(String s) {
    switch (s) {
      case 'valid':
        return StrideSessionState.valid;
      case 'refreshing':
        return StrideSessionState.refreshing;
      case 'expired':
        return StrideSessionState.expired;
      case 'no_session':
        return StrideSessionState.noSession;
      case 'revoked':
        return StrideSessionState.revoked;
      default:
        return StrideSessionState.noSession;
    }
  }

  String toJson() {
    switch (this) {
      case StrideSessionState.valid:
        return 'valid';
      case StrideSessionState.refreshing:
        return 'refreshing';
      case StrideSessionState.expired:
        return 'expired';
      case StrideSessionState.noSession:
        return 'no_session';
      case StrideSessionState.revoked:
        return 'revoked';
    }
  }
}

/// An action that requires recent reauthentication.
enum StrideSensitiveAction {
  changePassword,
  changeEmail,
  deleteAccount,
  linkAuthProvider,
  deleteAllUserData;

  static StrideSensitiveAction fromJson(String s) {
    switch (s) {
      case 'change_password':
        return StrideSensitiveAction.changePassword;
      case 'change_email':
        return StrideSensitiveAction.changeEmail;
      case 'delete_account':
        return StrideSensitiveAction.deleteAccount;
      case 'link_auth_provider':
        return StrideSensitiveAction.linkAuthProvider;
      case 'delete_all_user_data':
        return StrideSensitiveAction.deleteAllUserData;
      default:
        return StrideSensitiveAction.changePassword;
    }
  }

  String toJson() {
    switch (this) {
      case StrideSensitiveAction.changePassword:
        return 'change_password';
      case StrideSensitiveAction.changeEmail:
        return 'change_email';
      case StrideSensitiveAction.deleteAccount:
        return 'delete_account';
      case StrideSensitiveAction.linkAuthProvider:
        return 'link_auth_provider';
      case StrideSensitiveAction.deleteAllUserData:
        return 'delete_all_user_data';
    }
  }
}

/// Whether the user's email is verified.
enum StrideEmailVerificationState {
  verified,
  pending,
  notSent,
  notApplicable;

  static StrideEmailVerificationState fromJson(String s) {
    switch (s) {
      case 'verified':
        return StrideEmailVerificationState.verified;
      case 'pending':
        return StrideEmailVerificationState.pending;
      case 'not_sent':
        return StrideEmailVerificationState.notSent;
      case 'not_applicable':
        return StrideEmailVerificationState.notApplicable;
      default:
        return StrideEmailVerificationState.notSent;
    }
  }

  String toJson() {
    switch (this) {
      case StrideEmailVerificationState.verified:
        return 'verified';
      case StrideEmailVerificationState.pending:
        return 'pending';
      case StrideEmailVerificationState.notSent:
        return 'not_sent';
      case StrideEmailVerificationState.notApplicable:
        return 'not_applicable';
    }
  }
}

/// What action to take regarding email verification.
enum StrideVerificationAction {
  verified,
  sendVerification,
  waitForResend,
  notApplicable;

  static StrideVerificationAction fromJson(String s) {
    switch (s) {
      case 'verified':
        return StrideVerificationAction.verified;
      case 'send_verification':
        return StrideVerificationAction.sendVerification;
      case 'wait_for_resend':
        return StrideVerificationAction.waitForResend;
      case 'not_applicable':
        return StrideVerificationAction.notApplicable;
      default:
        return StrideVerificationAction.notApplicable;
    }
  }

  String toJson() {
    switch (this) {
      case StrideVerificationAction.verified:
        return 'verified';
      case StrideVerificationAction.sendVerification:
        return 'send_verification';
      case StrideVerificationAction.waitForResend:
        return 'wait_for_resend';
      case StrideVerificationAction.notApplicable:
        return 'not_applicable';
    }
  }
}

/// The status of a user account.
enum StrideAccountStatus {
  active,
  adminDisabled,
  temporarilyLocked,
  pendingDeletion;

  static StrideAccountStatus fromJson(String s) {
    switch (s) {
      case 'active':
        return StrideAccountStatus.active;
      case 'admin_disabled':
        return StrideAccountStatus.adminDisabled;
      case 'temporarily_locked':
        return StrideAccountStatus.temporarilyLocked;
      case 'pending_deletion':
        return StrideAccountStatus.pendingDeletion;
      default:
        return StrideAccountStatus.active;
    }
  }

  String toJson() {
    switch (this) {
      case StrideAccountStatus.active:
        return 'active';
      case StrideAccountStatus.adminDisabled:
        return 'admin_disabled';
      case StrideAccountStatus.temporarilyLocked:
        return 'temporarily_locked';
      case StrideAccountStatus.pendingDeletion:
        return 'pending_deletion';
    }
  }
}

/// What action to take when a suspicious login is detected.
enum StrideSuspiciousLoginAction {
  allow,
  allowWithNotification,
  requireVerification,
  block;

  static StrideSuspiciousLoginAction fromJson(String s) {
    switch (s) {
      case 'allow':
        return StrideSuspiciousLoginAction.allow;
      case 'allow_with_notification':
        return StrideSuspiciousLoginAction.allowWithNotification;
      case 'require_verification':
        return StrideSuspiciousLoginAction.requireVerification;
      case 'block':
        return StrideSuspiciousLoginAction.block;
      default:
        return StrideSuspiciousLoginAction.allow;
    }
  }

  String toJson() {
    switch (this) {
      case StrideSuspiciousLoginAction.allow:
        return 'allow';
      case StrideSuspiciousLoginAction.allowWithNotification:
        return 'allow_with_notification';
      case StrideSuspiciousLoginAction.requireVerification:
        return 'require_verification';
      case StrideSuspiciousLoginAction.block:
        return 'block';
    }
  }
}

/// A category of user-owned data for account deletion.
enum StrideUserDataCategory {
  workouts,
  routeFiles,
  stepSamples,
  heartRateSamples,
  checkpoints,
  achievements,
  personalRecords,
  syncQueueItems,
  coachingHistory,
  userProfile,
  offlineRegions,
  savedRoutes,
  goals;

  static StrideUserDataCategory fromJson(String s) {
    switch (s) {
      case 'workouts':
        return StrideUserDataCategory.workouts;
      case 'route_files':
        return StrideUserDataCategory.routeFiles;
      case 'step_samples':
        return StrideUserDataCategory.stepSamples;
      case 'heart_rate_samples':
        return StrideUserDataCategory.heartRateSamples;
      case 'checkpoints':
        return StrideUserDataCategory.checkpoints;
      case 'achievements':
        return StrideUserDataCategory.achievements;
      case 'personal_records':
        return StrideUserDataCategory.personalRecords;
      case 'sync_queue_items':
        return StrideUserDataCategory.syncQueueItems;
      case 'coaching_history':
        return StrideUserDataCategory.coachingHistory;
      case 'user_profile':
        return StrideUserDataCategory.userProfile;
      case 'offline_regions':
        return StrideUserDataCategory.offlineRegions;
      case 'saved_routes':
        return StrideUserDataCategory.savedRoutes;
      case 'goals':
        return StrideUserDataCategory.goals;
      default:
        return StrideUserDataCategory.workouts;
    }
  }

  String toJson() {
    switch (this) {
      case StrideUserDataCategory.workouts:
        return 'workouts';
      case StrideUserDataCategory.routeFiles:
        return 'route_files';
      case StrideUserDataCategory.stepSamples:
        return 'step_samples';
      case StrideUserDataCategory.heartRateSamples:
        return 'heart_rate_samples';
      case StrideUserDataCategory.checkpoints:
        return 'checkpoints';
      case StrideUserDataCategory.achievements:
        return 'achievements';
      case StrideUserDataCategory.personalRecords:
        return 'personal_records';
      case StrideUserDataCategory.syncQueueItems:
        return 'sync_queue_items';
      case StrideUserDataCategory.coachingHistory:
        return 'coaching_history';
      case StrideUserDataCategory.userProfile:
        return 'user_profile';
      case StrideUserDataCategory.offlineRegions:
        return 'offline_regions';
      case StrideUserDataCategory.savedRoutes:
        return 'saved_routes';
      case StrideUserDataCategory.goals:
        return 'goals';
    }
  }
}

/// The scope of an account/data deletion.
enum StrideDeletionScope {
  authAccountOnly,
  allUserData,
  dataOnly;

  static StrideDeletionScope fromJson(String s) {
    switch (s) {
      case 'auth_account_only':
        return StrideDeletionScope.authAccountOnly;
      case 'all_user_data':
        return StrideDeletionScope.allUserData;
      case 'data_only':
        return StrideDeletionScope.dataOnly;
      default:
        return StrideDeletionScope.allUserData;
    }
  }

  String toJson() {
    switch (this) {
      case StrideDeletionScope.authAccountOnly:
        return 'auth_account_only';
      case StrideDeletionScope.allUserData:
        return 'all_user_data';
      case StrideDeletionScope.dataOnly:
        return 'data_only';
    }
  }
}

/// Result of validating a password against the password policy.
class StridePasswordValidationResult {
  final bool isValid;
  final List<String> issues;

  StridePasswordValidationResult.fromJson(Map<String, dynamic> j)
      : isValid = j['is_valid'] as bool,
        issues = (j['issues'] as List<dynamic>? ?? [])
            .map((e) => e as String)
            .toList();
}

/// Context describing a login attempt for suspicious-activity analysis.
class StrideLoginContext {
  final String userId;
  final String ipAddress;
  final double latitude;
  final double longitude;
  final int loginAtMs;
  final String deviceFingerprint;
  final String userAgent;

  StrideLoginContext({
    required this.userId,
    required this.ipAddress,
    required this.latitude,
    required this.longitude,
    required this.loginAtMs,
    required this.deviceFingerprint,
    required this.userAgent,
  });

  Map<String, dynamic> toJson() => {
        'user_id': userId,
        'ip_address': ipAddress,
        'latitude': latitude,
        'longitude': longitude,
        'login_at_ms': loginAtMs,
        'device_fingerprint': deviceFingerprint,
        'user_agent': userAgent,
      };
}

/// The user's known login history for suspicious-activity analysis.
class StrideLoginHistory {
  final List<String> knownIpAddresses;
  final List<String> knownDeviceFingerprints;
  final List<List<double>> knownLocations;
  final int lastLoginAtMs;
  final List<double> lastLoginLocation;

  StrideLoginHistory({
    required this.knownIpAddresses,
    required this.knownDeviceFingerprints,
    required this.knownLocations,
    required this.lastLoginAtMs,
    required this.lastLoginLocation,
  });

  Map<String, dynamic> toJson() => {
        'known_ip_addresses': knownIpAddresses,
        'known_device_fingerprints': knownDeviceFingerprints,
        'known_locations':
            knownLocations.map((loc) => loc.toList()).toList(),
        'last_login_at_ms': lastLoginAtMs,
        'last_login_location': lastLoginLocation,
      };
}

/// Result of analyzing a login attempt for suspicious activity.
class StrideSuspiciousLoginResult {
  final bool isSuspicious;
  final List<String> reasons;
  final StrideSuspiciousLoginAction recommendedAction;

  StrideSuspiciousLoginResult.fromJson(Map<String, dynamic> j)
      : isSuspicious = j['is_suspicious'] as bool,
        reasons = (j['reasons'] as List<dynamic>? ?? [])
            .map((e) => e as String)
            .toList(),
        recommendedAction = StrideSuspiciousLoginAction.fromJson(
            j['recommended_action'] as String);
}

/// Information about a user-data category for deletion.
class StrideUserDataCategoryInfo {
  final StrideUserDataCategory category;
  final String name;
  final String storageLocation;
  final bool isFirestore;
  final bool isCloudStorage;
  final bool isLocal;
  final String pathPrefix;

  StrideUserDataCategoryInfo.fromJson(Map<String, dynamic> j)
      : category = StrideUserDataCategory.fromJson(j['category'] as String),
        name = j['name'] as String,
        storageLocation = j['storage_location'] as String,
        isFirestore = j['is_firestore'] as bool,
        isCloudStorage = j['is_cloud_storage'] as bool,
        isLocal = j['is_local'] as bool,
        pathPrefix = j['path_prefix'] as String;
}

/// The result of an account deletion operation.
class StrideDeletionResult {
  final List<StrideUserDataCategory> deleted;
  final List<Map<String, dynamic>> failed;
  final bool authAccountDeleted;
  final bool isComplete;

  StrideDeletionResult.fromJson(Map<String, dynamic> j)
      : deleted = (j['deleted'] as List<dynamic>? ?? [])
            .map((e) => StrideUserDataCategory.fromJson(e as String))
            .toList(),
        failed = (j['failed'] as List<dynamic>? ?? [])
            .map((e) => e as Map<String, dynamic>.from(e as Map))
            .toList(),
        authAccountDeleted = j['auth_account_deleted'] as bool,
        isComplete = j['is_complete'] as bool;
}

// ===========================================================================
// §7 — Secure Firebase: security validation models
// ===========================================================================

/// A Firestore collection in the S.T.R.I.D.E. database.
enum StrideFirestoreCollection {
  users,
  workouts,
  achievements,
  goals,
  trainingPlans,
  savedRoutes,
  dailySummaries,
  checkpoints,
  stepSamples,
  heartRateSamples,
  syncQueueItems,
  coachingHistory,
  personalRecords,
  adminFlaggedWorkouts,
  adminReportedUsers,
  adminAppConfig;

  static StrideFirestoreCollection fromJson(String s) {
    switch (s) {
      case 'users':
        return StrideFirestoreCollection.users;
      case 'workouts':
        return StrideFirestoreCollection.workouts;
      case 'achievements':
        return StrideFirestoreCollection.achievements;
      case 'goals':
        return StrideFirestoreCollection.goals;
      case 'training_plans':
        return StrideFirestoreCollection.trainingPlans;
      case 'saved_routes':
        return StrideFirestoreCollection.savedRoutes;
      case 'daily_summaries':
        return StrideFirestoreCollection.dailySummaries;
      case 'checkpoints':
        return StrideFirestoreCollection.checkpoints;
      case 'step_samples':
        return StrideFirestoreCollection.stepSamples;
      case 'heart_rate_samples':
        return StrideFirestoreCollection.heartRateSamples;
      case 'sync_queue_items':
        return StrideFirestoreCollection.syncQueueItems;
      case 'coaching_history':
        return StrideFirestoreCollection.coachingHistory;
      case 'personal_records':
        return StrideFirestoreCollection.personalRecords;
      case 'admin_flagged_workouts':
        return StrideFirestoreCollection.adminFlaggedWorkouts;
      case 'admin_reported_users':
        return StrideFirestoreCollection.adminReportedUsers;
      case 'admin_app_config':
        return StrideFirestoreCollection.adminAppConfig;
      default:
        return StrideFirestoreCollection.users;
    }
  }

  String toJson() {
    switch (this) {
      case StrideFirestoreCollection.users:
        return 'users';
      case StrideFirestoreCollection.workouts:
        return 'workouts';
      case StrideFirestoreCollection.achievements:
        return 'achievements';
      case StrideFirestoreCollection.goals:
        return 'goals';
      case StrideFirestoreCollection.trainingPlans:
        return 'training_plans';
      case StrideFirestoreCollection.savedRoutes:
        return 'saved_routes';
      case StrideFirestoreCollection.dailySummaries:
        return 'daily_summaries';
      case StrideFirestoreCollection.checkpoints:
        return 'checkpoints';
      case StrideFirestoreCollection.stepSamples:
        return 'step_samples';
      case StrideFirestoreCollection.heartRateSamples:
        return 'heart_rate_samples';
      case StrideFirestoreCollection.syncQueueItems:
        return 'sync_queue_items';
      case StrideFirestoreCollection.coachingHistory:
        return 'coaching_history';
      case StrideFirestoreCollection.personalRecords:
        return 'personal_records';
      case StrideFirestoreCollection.adminFlaggedWorkouts:
        return 'admin_flagged_workouts';
      case StrideFirestoreCollection.adminReportedUsers:
        return 'admin_reported_users';
      case StrideFirestoreCollection.adminAppConfig:
        return 'admin_app_config';
    }
  }
}

/// The type of access being requested.
enum StrideAccessType {
  read,
  write,
  delete;

  static StrideAccessType fromJson(String s) {
    switch (s) {
      case 'read':
        return StrideAccessType.read;
      case 'write':
        return StrideAccessType.write;
      case 'delete':
        return StrideAccessType.delete;
      default:
        return StrideAccessType.read;
    }
  }

  String toJson() {
    switch (this) {
      case StrideAccessType.read:
        return 'read';
      case StrideAccessType.write:
        return 'write';
      case StrideAccessType.delete:
        return 'delete';
    }
  }
}

/// The result of an access-control check.
enum StrideAccessDecision {
  allow,
  denyOwnerMismatch,
  denyAdminOnly,
  denyNotAuthenticated,
  denyReadOnly,
  denyInvalidPath;

  static StrideAccessDecision fromJson(String s) {
    switch (s) {
      case 'allow':
        return StrideAccessDecision.allow;
      case 'deny_owner_mismatch':
        return StrideAccessDecision.denyOwnerMismatch;
      case 'deny_admin_only':
        return StrideAccessDecision.denyAdminOnly;
      case 'deny_not_authenticated':
        return StrideAccessDecision.denyNotAuthenticated;
      case 'deny_read_only':
        return StrideAccessDecision.denyReadOnly;
      case 'deny_invalid_path':
        return StrideAccessDecision.denyInvalidPath;
      default:
        return StrideAccessDecision.denyInvalidPath;
    }
  }

  String toJson() {
    switch (this) {
      case StrideAccessDecision.allow:
        return 'allow';
      case StrideAccessDecision.denyOwnerMismatch:
        return 'deny_owner_mismatch';
      case StrideAccessDecision.denyAdminOnly:
        return 'deny_admin_only';
      case StrideAccessDecision.denyNotAuthenticated:
        return 'deny_not_authenticated';
      case StrideAccessDecision.denyReadOnly:
        return 'deny_read_only';
      case StrideAccessDecision.denyInvalidPath:
        return 'deny_invalid_path';
    }
  }

  bool get isAllowed => this == StrideAccessDecision.allow;
}

/// A field type in a Firestore document.
enum StrideFieldType {
  string,
  integer,
  float,
  boolean,
  timestamp,
  array,
  map,
  null_;

  static StrideFieldType fromJson(String s) {
    switch (s) {
      case 'string':
        return StrideFieldType.string;
      case 'integer':
        return StrideFieldType.integer;
      case 'float':
        return StrideFieldType.float;
      case 'boolean':
        return StrideFieldType.boolean;
      case 'timestamp':
        return StrideFieldType.timestamp;
      case 'array':
        return StrideFieldType.array;
      case 'map':
        return StrideFieldType.map;
      case 'null':
        return StrideFieldType.null_;
      default:
        return StrideFieldType.string;
    }
  }

  String toJson() {
    switch (this) {
      case StrideFieldType.string:
        return 'string';
      case StrideFieldType.integer:
        return 'integer';
      case StrideFieldType.float:
        return 'float';
      case StrideFieldType.boolean:
        return 'boolean';
      case StrideFieldType.timestamp:
        return 'timestamp';
      case StrideFieldType.array:
        return 'array';
      case StrideFieldType.map:
        return 'map';
      case StrideFieldType.null_:
        return 'null';
    }
  }
}

/// The attestation state from Firebase App Check.
enum StrideAppCheckState {
  valid,
  stale,
  missing,
  invalid,
  notEnforced;

  static StrideAppCheckState fromJson(String s) {
    switch (s) {
      case 'valid':
        return StrideAppCheckState.valid;
      case 'stale':
        return StrideAppCheckState.stale;
      case 'missing':
        return StrideAppCheckState.missing;
      case 'invalid':
        return StrideAppCheckState.invalid;
      case 'not_enforced':
        return StrideAppCheckState.notEnforced;
      default:
        return StrideAppCheckState.missing;
    }
  }

  String toJson() {
    switch (this) {
      case StrideAppCheckState.valid:
        return 'valid';
      case StrideAppCheckState.stale:
        return 'stale';
      case StrideAppCheckState.missing:
        return 'missing';
      case StrideAppCheckState.invalid:
        return 'invalid';
      case StrideAppCheckState.notEnforced:
        return 'not_enforced';
    }
  }
}

/// The Play Integrity API verdict for an Android device.
enum StridePlayIntegrityVerdict {
  pass,
  appIntegrityFailed,
  deviceIntegrityFailed,
  accountIntegrityFailed,
  basicIntegrityFailed;

  static StridePlayIntegrityVerdict fromJson(String s) {
    switch (s) {
      case 'pass':
        return StridePlayIntegrityVerdict.pass;
      case 'app_integrity_failed':
        return StridePlayIntegrityVerdict.appIntegrityFailed;
      case 'device_integrity_failed':
        return StridePlayIntegrityVerdict.deviceIntegrityFailed;
      case 'account_integrity_failed':
        return StridePlayIntegrityVerdict.accountIntegrityFailed;
      case 'basic_integrity_failed':
        return StridePlayIntegrityVerdict.basicIntegrityFailed;
      default:
        return StridePlayIntegrityVerdict.basicIntegrityFailed;
    }
  }

  String toJson() {
    switch (this) {
      case StridePlayIntegrityVerdict.pass:
        return 'pass';
      case StridePlayIntegrityVerdict.appIntegrityFailed:
        return 'app_integrity_failed';
      case StridePlayIntegrityVerdict.deviceIntegrityFailed:
        return 'device_integrity_failed';
      case StridePlayIntegrityVerdict.accountIntegrityFailed:
        return 'account_integrity_failed';
      case StridePlayIntegrityVerdict.basicIntegrityFailed:
        return 'basic_integrity_failed';
    }
  }

  bool get isPass => this == StridePlayIntegrityVerdict.pass;
}

/// A rate limit category for Cloud Functions endpoints.
enum StrideRateLimitCategory {
  aiCoaching,
  workoutSync,
  routeUpload,
  generalApi;

  static StrideRateLimitCategory fromJson(String s) {
    switch (s) {
      case 'ai_coaching':
        return StrideRateLimitCategory.aiCoaching;
      case 'workout_sync':
        return StrideRateLimitCategory.workoutSync;
      case 'route_upload':
        return StrideRateLimitCategory.routeUpload;
      case 'general_api':
        return StrideRateLimitCategory.generalApi;
      default:
        return StrideRateLimitCategory.generalApi;
    }
  }

  String toJson() {
    switch (this) {
      case StrideRateLimitCategory.aiCoaching:
        return 'ai_coaching';
      case StrideRateLimitCategory.workoutSync:
        return 'workout_sync';
      case StrideRateLimitCategory.routeUpload:
        return 'route_upload';
      case StrideRateLimitCategory.generalApi:
        return 'general_api';
    }
  }
}

/// The Firebase environment (development, staging, or production).
enum StrideFirebaseEnvironment {
  development,
  staging,
  production;

  static StrideFirebaseEnvironment fromJson(String s) {
    switch (s) {
      case 'development':
        return StrideFirebaseEnvironment.development;
      case 'staging':
        return StrideFirebaseEnvironment.staging;
      case 'production':
        return StrideFirebaseEnvironment.production;
      default:
        return StrideFirebaseEnvironment.development;
    }
  }

  String toJson() {
    switch (this) {
      case StrideFirebaseEnvironment.development:
        return 'development';
      case StrideFirebaseEnvironment.staging:
        return 'staging';
      case StrideFirebaseEnvironment.production:
        return 'production';
    }
  }
}

/// Context for an access-control decision.
class StrideAccessContext {
  final String userId;
  final bool isAdmin;
  final StrideFirestoreCollection collection;
  final StrideAccessType accessType;
  final String docOwnerId;

  StrideAccessContext({
    required this.userId,
    required this.isAdmin,
    required this.collection,
    required this.accessType,
    required this.docOwnerId,
  });

  Map<String, dynamic> toJson() => {
        'user_id': userId,
        'is_admin': isAdmin,
        'collection': collection.toJson(),
        'access_type': accessType.toJson(),
        'doc_owner_id': docOwnerId,
      };

  StrideAccessContext.fromJson(Map<String, dynamic> j)
      : userId = j['user_id'] as String,
        isAdmin = j['is_admin'] as bool,
        collection = StrideFirestoreCollection.fromJson(j['collection'] as String),
        accessType = StrideAccessType.fromJson(j['access_type'] as String),
        docOwnerId = j['doc_owner_id'] as String;
}

/// A field validation rule for a Firestore collection.
class StrideFieldRule {
  final String field;
  final StrideFieldType fieldType;
  final bool required;
  final double? min;
  final double? max;

  StrideFieldRule({
    required this.field,
    required this.fieldType,
    required this.required,
    this.min,
    this.max,
  });

  Map<String, dynamic> toJson() => {
        'field': field,
        'field_type': fieldType.toJson(),
        'required': required,
        'min': min,
        'max': max,
      };

  StrideFieldRule.fromJson(Map<String, dynamic> j)
      : field = j['field'] as String,
        fieldType = StrideFieldType.fromJson(j['field_type'] as String),
        required = j['required'] as bool,
        min = j['min']?.toDouble(),
        max = j['max']?.toDouble();
}

/// A validation issue found in a document.
class StrideFieldValidationIssue {
  final String field;
  final String issue;

  StrideFieldValidationIssue.fromJson(Map<String, dynamic> j)
      : field = j['field'] as String,
        issue = j['issue'] as String;
}

/// The result of validating a document's fields.
class StrideFieldValidationResult {
  final bool isValid;
  final List<StrideFieldValidationIssue> issues;

  StrideFieldValidationResult.fromJson(Map<String, dynamic> j)
      : isValid = j['is_valid'] as bool,
        issues = (j['issues'] as List<dynamic>? ?? [])
            .map((e) => StrideFieldValidationIssue.fromJson(
                Map<String, dynamic>.from(e as Map)))
            .toList();
}

/// A token-bucket rate limiter state.
class StrideRateLimitBucket {
  final double capacity;
  final double refillRate;
  final double currentTokens;
  final int lastRefillMs;

  StrideRateLimitBucket({
    required this.capacity,
    required this.refillRate,
    required this.currentTokens,
    required this.lastRefillMs,
  });

  Map<String, dynamic> toJson() => {
        'capacity': capacity,
        'refill_rate': refillRate,
        'current_tokens': currentTokens,
        'last_refill_ms': lastRefillMs,
      };

  StrideRateLimitBucket.fromJson(Map<String, dynamic> j)
      : capacity = j['capacity'].toDouble(),
        refillRate = j['refill_rate'].toDouble(),
        currentTokens = j['current_tokens'].toDouble(),
        lastRefillMs = j['last_refill_ms'] as int;
}

/// The result of a rate-limit check.
class StrideRateLimitResult {
  final bool allowed;
  final double remainingTokens;
  final int retryAfterMs;

  StrideRateLimitResult.fromJson(Map<String, dynamic> j)
      : allowed = j['allowed'] as bool,
        remainingTokens = j['remaining_tokens'].toDouble(),
        retryAfterMs = j['retry_after_ms'] as int;
}

// ===========================================================================
// §8 — AI coaching plan & safety guards
// ===========================================================================

/// The user's experience level.
///
/// Mirrors `coaching_plan::ExperienceLevel`.
enum StrideExperienceLevel {
  beginner,
  intermediate,
  advanced;

  static StrideExperienceLevel fromJson(String s) {
    switch (s) {
      case 'beginner':
        return StrideExperienceLevel.beginner;
      case 'intermediate':
        return StrideExperienceLevel.intermediate;
      case 'advanced':
        return StrideExperienceLevel.advanced;
      default:
        throw FormatException('Unknown ExperienceLevel: $s');
    }
  }

  String toJson() => name;
}

/// The difficulty/intensity of a single workout day.
///
/// Mirrors `coaching_plan::WorkoutDifficulty`.
enum StrideWorkoutDifficulty {
  easy,
  moderate,
  hard;

  static StrideWorkoutDifficulty fromJson(String s) {
    switch (s) {
      case 'easy':
        return StrideWorkoutDifficulty.easy;
      case 'moderate':
        return StrideWorkoutDifficulty.moderate;
      case 'hard':
        return StrideWorkoutDifficulty.hard;
      default:
        throw FormatException('Unknown WorkoutDifficulty: $s');
    }
  }

  String toJson() => name;
}

/// A single day in a weekly training plan.
class StrideDayPlan {
  final int dayOfWeek;
  final bool isRestDay;
  final double targetDistanceM;
  final int targetDurationS;
  final StrideWorkoutDifficulty difficulty;
  final String description;

  StrideDayPlan({
    required this.dayOfWeek,
    required this.isRestDay,
    required this.targetDistanceM,
    required this.targetDurationS,
    required this.difficulty,
    required this.description,
  });

  Map<String, dynamic> toJson() => {
        'day_of_week': dayOfWeek,
        'is_rest_day': isRestDay,
        'target_distance_m': targetDistanceM,
        'target_duration_s': targetDurationS,
        'difficulty': difficulty.toJson(),
        'description': description,
      };

  StrideDayPlan.fromJson(Map<String, dynamic> j)
      : dayOfWeek = j['day_of_week'] as int,
        isRestDay = j['is_rest_day'] as bool,
        targetDistanceM = j['target_distance_m'].toDouble(),
        targetDurationS = j['target_duration_s'] as int,
        difficulty = StrideWorkoutDifficulty.fromJson(j['difficulty'] as String),
        description = j['description'] as String;
}

/// A 7-day weekly training plan.
class StrideWeeklyPlan {
  final List<StrideDayPlan> days;
  final double weeklyDistanceM;
  final int weeklyDurationS;
  final int restDays;
  final StrideExperienceLevel experienceLevel;

  StrideWeeklyPlan({
    required this.days,
    required this.weeklyDistanceM,
    required this.weeklyDurationS,
    required this.restDays,
    required this.experienceLevel,
  });

  Map<String, dynamic> toJson() => {
        'days': days.map((d) => d.toJson()).toList(),
        'weekly_distance_m': weeklyDistanceM,
        'weekly_duration_s': weeklyDurationS,
        'rest_days': restDays,
        'experience_level': experienceLevel.toJson(),
      };

  StrideWeeklyPlan.fromJson(Map<String, dynamic> j)
      : days = (j['days'] as List<dynamic>)
            .map((e) => StrideDayPlan.fromJson(Map<String, dynamic>.from(e as Map)))
            .toList(),
        weeklyDistanceM = j['weekly_distance_m'].toDouble(),
        weeklyDurationS = j['weekly_duration_s'] as int,
        restDays = j['rest_days'] as int,
        experienceLevel =
            StrideExperienceLevel.fromJson(j['experience_level'] as String);
}

/// The result of validating a training plan.
class StridePlanValidationResult {
  final bool isValid;
  final List<String> issues;
  final List<String> adjustments;

  StridePlanValidationResult.fromJson(Map<String, dynamic> j)
      : isValid = j['is_valid'] as bool,
        issues = (j['issues'] as List<dynamic>? ?? [])
            .map((e) => e as String)
            .toList(),
        adjustments = (j['adjustments'] as List<dynamic>? ?? [])
            .map((e) => e as String)
            .toList();
}

/// Experience-level caps returned by `coachingPlanExperienceCaps`.
class StrideExperienceCaps {
  final StrideExperienceLevel experienceLevel;
  final String label;
  final double maxSingleSessionDistanceM;
  final int maxSingleSessionDurationS;
  final double maxWeeklyDistanceM;
  final int recommendedRestDaysPerWeek;

  StrideExperienceCaps.fromJson(Map<String, dynamic> j)
      : experienceLevel =
            StrideExperienceLevel.fromJson(j['experience_level'] as String),
        label = j['label'] as String,
        maxSingleSessionDistanceM =
            j['max_single_session_distance_m'].toDouble(),
        maxSingleSessionDurationS =
            j['max_single_session_duration_s'] as int,
        maxWeeklyDistanceM = j['max_weekly_distance_m'].toDouble(),
        recommendedRestDaysPerWeek =
            j['recommended_rest_days_per_week'] as int;
}

/// The type of pain the user reports.
///
/// Mirrors `coaching_plan::PainType`.
enum StridePainType {
  none,
  mild,
  moderate,
  severe,
  joint,
  chest,
  dizziness;

  static StridePainType fromJson(String s) {
    switch (s) {
      case 'none':
        return StridePainType.none;
      case 'mild':
        return StridePainType.mild;
      case 'moderate':
        return StridePainType.moderate;
      case 'severe':
        return StridePainType.severe;
      case 'joint':
        return StridePainType.joint;
      case 'chest':
        return StridePainType.chest;
      case 'dizziness':
        return StridePainType.dizziness;
      default:
        throw FormatException('Unknown PainType: $s');
    }
  }

  String toJson() => name;
}

/// The action the system recommends in response to pain.
///
/// Mirrors `coaching_plan::PainAction`.
enum StridePainAction {
  continueAction,
  reduceIntensity,
  rest,
  stopAndRest,
  seekMedicalAttention;

  static StridePainAction fromJson(String s) {
    switch (s) {
      case 'continue':
        return StridePainAction.continueAction;
      case 'reduce_intensity':
        return StridePainAction.reduceIntensity;
      case 'rest':
        return StridePainAction.rest;
      case 'stop_and_rest':
        return StridePainAction.stopAndRest;
      case 'seek_medical_attention':
        return StridePainAction.seekMedicalAttention;
      default:
        throw FormatException('Unknown PainAction: $s');
    }
  }

  String toJson() {
    switch (this) {
      case StridePainAction.continueAction:
        return 'continue';
      case StridePainAction.reduceIntensity:
        return 'reduce_intensity';
      case StridePainAction.rest:
        return 'rest';
      case StridePainAction.stopAndRest:
        return 'stop_and_rest';
      case StridePainAction.seekMedicalAttention:
        return 'seek_medical_attention';
    }
  }
}

/// The system's response to a user-reported pain.
class StridePainResponse {
  final StridePainAction action;
  final String message;
  final bool shouldEscalate;
  final bool shouldStopTraining;

  StridePainResponse.fromJson(Map<String, dynamic> j)
      : action = StridePainAction.fromJson(j['action'] as String),
        message = j['message'] as String,
        shouldEscalate = j['should_escalate'] as bool,
        shouldStopTraining = j['should_stop_training'] as bool;
}

/// The result of validating AI-generated coaching text.
class StrideContentValidationResult {
  final bool isSafe;
  final List<String> violations;

  StrideContentValidationResult.fromJson(Map<String, dynamic> j)
      : isSafe = j['is_safe'] as bool,
        violations = (j['violations'] as List<dynamic>? ?? [])
            .map((e) => e as String)
            .toList();
}

/// The reason for escalating to a medical professional.
///
/// Mirrors `coaching_plan::EscalationReason`.
enum StrideEscalationReason {
  chestPain,
  severeDizziness,
  severePersistentPain,
  cannotBearWeight,
  neurologicalSymptoms,
  persistentUnexplainedSymptoms;

  static StrideEscalationReason fromJson(String s) {
    switch (s) {
      case 'chest_pain':
        return StrideEscalationReason.chestPain;
      case 'severe_dizziness':
        return StrideEscalationReason.severeDizziness;
      case 'severe_persistent_pain':
        return StrideEscalationReason.severePersistentPain;
      case 'cannot_bear_weight':
        return StrideEscalationReason.cannotBearWeight;
      case 'neurological_symptoms':
        return StrideEscalationReason.neurologicalSymptoms;
      case 'persistent_unexplained_symptoms':
        return StrideEscalationReason.persistentUnexplainedSymptoms;
      default:
        throw FormatException('Unknown EscalationReason: $s');
    }
  }

  String toJson() {
    switch (this) {
      case StrideEscalationReason.chestPain:
        return 'chest_pain';
      case StrideEscalationReason.severeDizziness:
        return 'severe_dizziness';
      case StrideEscalationReason.severePersistentPain:
        return 'severe_persistent_pain';
      case StrideEscalationReason.cannotBearWeight:
        return 'cannot_bear_weight';
      case StrideEscalationReason.neurologicalSymptoms:
        return 'neurological_symptoms';
      case StrideEscalationReason.persistentUnexplainedSymptoms:
        return 'persistent_unexplained_symptoms';
    }
  }
}

/// An escalation message for the user.
class StrideEscalationMessage {
  final StrideEscalationReason reason;
  final String message;
  final bool recommendEmergency;

  StrideEscalationMessage.fromJson(Map<String, dynamic> j)
      : reason = StrideEscalationReason.fromJson(j['reason'] as String),
        message = j['message'] as String,
        recommendEmergency = j['recommend_emergency'] as bool;
}

/// The type of feedback the user gives on a plan.
///
/// Mirrors `coaching_plan::UserFeedback`.
enum StrideUserFeedback {
  accept,
  reject,
  modify,
  tooHard,
  tooEasy,
  noTime,
  injured;

  static StrideUserFeedback fromJson(String s) {
    switch (s) {
      case 'accept':
        return StrideUserFeedback.accept;
      case 'reject':
        return StrideUserFeedback.reject;
      case 'modify':
        return StrideUserFeedback.modify;
      case 'too_hard':
        return StrideUserFeedback.tooHard;
      case 'too_easy':
        return StrideUserFeedback.tooEasy;
      case 'no_time':
        return StrideUserFeedback.noTime;
      case 'injured':
        return StrideUserFeedback.injured;
      default:
        throw FormatException('Unknown UserFeedback: $s');
    }
  }

  String toJson() {
    switch (this) {
      case StrideUserFeedback.accept:
        return 'accept';
      case StrideUserFeedback.reject:
        return 'reject';
      case StrideUserFeedback.modify:
        return 'modify';
      case StrideUserFeedback.tooHard:
        return 'too_hard';
      case StrideUserFeedback.tooEasy:
        return 'too_easy';
      case StrideUserFeedback.noTime:
        return 'no_time';
      case StrideUserFeedback.injured:
        return 'injured';
    }
  }
}

/// The action to take based on user feedback.
///
/// Mirrors `coaching_plan::FeedbackAction`.
enum StrideFeedbackAction {
  keepPlan,
  adjustPlan,
  reduceDifficulty,
  increaseDifficulty,
  reduceTimeCommitment,
  switchToRecovery,
  escalateToHumanCoach;

  static StrideFeedbackAction fromJson(String s) {
    switch (s) {
      case 'keep_plan':
        return StrideFeedbackAction.keepPlan;
      case 'adjust_plan':
        return StrideFeedbackAction.adjustPlan;
      case 'reduce_difficulty':
        return StrideFeedbackAction.reduceDifficulty;
      case 'increase_difficulty':
        return StrideFeedbackAction.increaseDifficulty;
      case 'reduce_time_commitment':
        return StrideFeedbackAction.reduceTimeCommitment;
      case 'switch_to_recovery':
        return StrideFeedbackAction.switchToRecovery;
      case 'escalate_to_human_coach':
        return StrideFeedbackAction.escalateToHumanCoach;
      default:
        throw FormatException('Unknown FeedbackAction: $s');
    }
  }

  String toJson() {
    switch (this) {
      case StrideFeedbackAction.keepPlan:
        return 'keep_plan';
      case StrideFeedbackAction.adjustPlan:
        return 'adjust_plan';
      case StrideFeedbackAction.reduceDifficulty:
        return 'reduce_difficulty';
      case StrideFeedbackAction.increaseDifficulty:
        return 'increase_difficulty';
      case StrideFeedbackAction.reduceTimeCommitment:
        return 'reduce_time_commitment';
      case StrideFeedbackAction.switchToRecovery:
        return 'switch_to_recovery';
      case StrideFeedbackAction.escalateToHumanCoach:
        return 'escalate_to_human_coach';
    }
  }
}

/// The result of processing user feedback.
class StrideFeedbackResult {
  final StrideFeedbackAction action;
  final String message;
  final bool shouldRegenerate;

  StrideFeedbackResult.fromJson(Map<String, dynamic> j)
      : action = StrideFeedbackAction.fromJson(j['action'] as String),
        message = j['message'] as String,
        shouldRegenerate = j['should_regenerate'] as bool;
}

/// Whether the AI service is available for use.
///
/// Mirrors `coaching_plan::AiAvailability`.
enum StrideAiAvailability {
  available,
  serviceUnavailable,
  quotaExceeded,
  costBudgetExceeded,
  disabled;

  static StrideAiAvailability fromJson(String s) {
    switch (s) {
      case 'available':
        return StrideAiAvailability.available;
      case 'service_unavailable':
        return StrideAiAvailability.serviceUnavailable;
      case 'quota_exceeded':
        return StrideAiAvailability.quotaExceeded;
      case 'cost_budget_exceeded':
        return StrideAiAvailability.costBudgetExceeded;
      case 'disabled':
        return StrideAiAvailability.disabled;
      default:
        throw FormatException('Unknown AiAvailability: $s');
    }
  }

  String toJson() {
    switch (this) {
      case StrideAiAvailability.available:
        return 'available';
      case StrideAiAvailability.serviceUnavailable:
        return 'service_unavailable';
      case StrideAiAvailability.quotaExceeded:
        return 'quota_exceeded';
      case StrideAiAvailability.costBudgetExceeded:
        return 'cost_budget_exceeded';
      case StrideAiAvailability.disabled:
        return 'disabled';
    }
  }
}

/// The type of AI operation being performed.
///
/// Mirrors `coaching_plan::AiOperation`.
enum StrideAiOperation {
  generatePlan,
  adjustPlan,
  summarizeWorkout,
  encouragement,
  answerQuestion,
  analyzeProgress;

  static StrideAiOperation fromJson(String s) {
    switch (s) {
      case 'generate_plan':
        return StrideAiOperation.generatePlan;
      case 'adjust_plan':
        return StrideAiOperation.adjustPlan;
      case 'summarize_workout':
        return StrideAiOperation.summarizeWorkout;
      case 'encouragement':
        return StrideAiOperation.encouragement;
      case 'answer_question':
        return StrideAiOperation.answerQuestion;
      case 'analyze_progress':
        return StrideAiOperation.analyzeProgress;
      default:
        throw FormatException('Unknown AiOperation: $s');
    }
  }

  String toJson() {
    switch (this) {
      case StrideAiOperation.generatePlan:
        return 'generate_plan';
      case StrideAiOperation.adjustPlan:
        return 'adjust_plan';
      case StrideAiOperation.summarizeWorkout:
        return 'summarize_workout';
      case StrideAiOperation.encouragement:
        return 'encouragement';
      case StrideAiOperation.answerQuestion:
        return 'answer_question';
      case StrideAiOperation.analyzeProgress:
        return 'analyze_progress';
    }
  }
}

/// Input data for summarizing a completed workout.
class StrideWorkoutSummaryInput {
  final double distanceM;
  final int durationS;
  final double avgSpeedMps;
  final int steps;
  final int calories;
  final StrideExperienceLevel experienceLevel;

  StrideWorkoutSummaryInput({
    required this.distanceM,
    required this.durationS,
    required this.avgSpeedMps,
    required this.steps,
    required this.calories,
    required this.experienceLevel,
  });

  Map<String, dynamic> toJson() => {
        'distance_m': distanceM,
        'duration_s': durationS,
        'avg_speed_mps': avgSpeedMps,
        'steps': steps,
        'calories': calories,
        'experience_level': experienceLevel.toJson(),
      };
}

/// Input for adjusting a plan based on user feedback.
class StridePlanAdjustmentInput {
  final double currentWeeklyDistanceM;
  final StrideExperienceLevel experienceLevel;
  final StrideUserFeedback feedback;
  final bool isInjured;

  StridePlanAdjustmentInput({
    required this.currentWeeklyDistanceM,
    required this.experienceLevel,
    required this.feedback,
    required this.isInjured,
  });

  Map<String, dynamic> toJson() => {
        'current_weekly_distance_m': currentWeeklyDistanceM,
        'experience_level': experienceLevel.toJson(),
        'feedback': feedback.toJson(),
        'is_injured': isInjured,
      };
}

/// The result of a plan adjustment.
class StridePlanAdjustmentResult {
  final double adjustedWeeklyDistanceM;
  final bool difficultyReduced;
  final int restDaysAdded;
  final String message;

  StridePlanAdjustmentResult.fromJson(Map<String, dynamic> j)
      : adjustedWeeklyDistanceM =
            j['adjusted_weekly_distance_m'].toDouble(),
        difficultyReduced = j['difficulty_reduced'] as bool,
        restDaysAdded = j['rest_days_added'] as int,
        message = j['message'] as String;
}

/// The result of moderating a user request for safety.
class StrideRequestModerationResult {
  final bool isSafe;
  final String? refusalMessage;
  final String? flagReason;

  StrideRequestModerationResult.fromJson(Map<String, dynamic> j)
      : isSafe = j['is_safe'] as bool,
        refusalMessage = j['refusal_message'] as String?,
        flagReason = j['flag_reason'] as String?;
}

// ─── §9 — Calorie/fitness calculations ─────────────────────────────

/// The version of the calorie calculation engine that produced a given
/// estimate, surfaced on every `CalorieEstimate` so the UI and cloud
/// can display the calculation version alongside the value.
enum StrideCalorieVersion {
  v1,
  v2;

  String get label => switch (this) {
    StrideCalorieVersion.v1 => 'Calorie estimate v1 (MET/HR/wearable)',
    StrideCalorieVersion.v2 => 'Calorie estimate v2 (grade-adjusted)',
  };

  static StrideCalorieVersion get current => StrideCalorieVersion.v2;

  static StrideCalorieVersion fromJson(String s) => switch (s) {
    'v1' => StrideCalorieVersion.v1,
    'v2' => StrideCalorieVersion.v2,
    _ => StrideCalorieVersion.v2,
  };

  String toJson() => switch (this) {
    StrideCalorieVersion.v1 => 'v1',
    StrideCalorieVersion.v2 => 'v2',
  };
}

/// The method used to produce a calorie estimate, following a strict
/// source-priority order: wearable > heart_rate > met > distance_weight.
enum StrideCalorieMethod {
  wearable,
  heartRate,
  met,
  distanceWeight;

  String get asStr => switch (this) {
    StrideCalorieMethod.wearable => 'wearable',
    StrideCalorieMethod.heartRate => 'heart_rate',
    StrideCalorieMethod.met => 'met',
    StrideCalorieMethod.distanceWeight => 'distance_weight',
  };

  String get label => switch (this) {
    StrideCalorieMethod.wearable => 'Wearable (device-reported)',
    StrideCalorieMethod.heartRate => 'Heart-rate based (Keytel et al.)',
    StrideCalorieMethod.met => 'MET-based (activity compendium)',
    StrideCalorieMethod.distanceWeight => 'Distance & weight (rough estimate)',
  };

  int get priorityRank => switch (this) {
    StrideCalorieMethod.wearable => 1,
    StrideCalorieMethod.heartRate => 2,
    StrideCalorieMethod.met => 3,
    StrideCalorieMethod.distanceWeight => 4,
  };

  static StrideCalorieMethod fromJson(String s) => switch (s) {
    'wearable' => StrideCalorieMethod.wearable,
    'heart_rate' => StrideCalorieMethod.heartRate,
    'met' => StrideCalorieMethod.met,
    'distance_weight' => StrideCalorieMethod.distanceWeight,
    _ => StrideCalorieMethod.met,
  };

  String toJson() => asStr;
}

/// Inputs for a calorie estimate, following the source-priority chain:
/// wearable > heart_rate > met > distance_weight.
class StrideCalorieInputs {
  final double? wearableKcal;
  final double? weightKg;
  final int durationMs;
  final double distanceMeters;
  final double averageSpeedMps;
  final double? averageHeartRateBpm;
  final int? ageYears;
  final bool? isMale;
  final String activityType;
  final double elevationGainMeters;

  StrideCalorieInputs({
    this.wearableKcal,
    this.weightKg,
    required this.durationMs,
    required this.distanceMeters,
    required this.averageSpeedMps,
    this.averageHeartRateBpm,
    this.ageYears,
    this.isMale,
    this.activityType = 'walk',
    this.elevationGainMeters = 0.0,
  });

  Map<String, dynamic> toJson() => {
    'wearable_kcal': wearableKcal,
    'weight_kg': weightKg,
    'duration_ms': durationMs,
    'distance_meters': distanceMeters,
    'average_speed_mps': averageSpeedMps,
    'average_heart_rate_bpm': averageHeartRateBpm,
    'age_years': ageYears,
    'is_male': isMale,
    'activity_type': activityType,
    'elevation_gain_meters': elevationGainMeters,
  };
}

/// A calorie estimate with the method and version that produced it.
class StrideCalorieEstimate {
  final double kcal;
  final StrideCalorieMethod method;
  final StrideCalorieVersion version;

  StrideCalorieEstimate({
    required this.kcal,
    required this.method,
    required this.version,
  });

  StrideCalorieEstimate.fromJson(Map<String, dynamic> j)
      : kcal = (j['kcal'] as num).toDouble(),
        method = StrideCalorieMethod.fromJson(j['method'] as String),
        version = StrideCalorieVersion.fromJson(j['version'] as String);

  Map<String, dynamic> toJson() => {
    'kcal': kcal,
    'method': method.toJson(),
    'version': version.toJson(),
  };
}

/// A full calorie estimate result with display labels, version, and
/// source-priority information, suitable for the UI layer.
class StrideCalorieEstimateResult {
  final double kcal;
  final StrideCalorieMethod method;
  final StrideCalorieVersion version;
  final String methodStr;
  final String versionStr;
  final String methodLabel;
  final String versionLabel;
  final String displayLabel;
  final int sourcePriority;
  final bool isEstimate;

  StrideCalorieEstimateResult.fromJson(Map<String, dynamic> j)
      : kcal = (j['kcal'] as num).toDouble(),
        method = StrideCalorieMethod.fromJson(j['method'] as String),
        version = StrideCalorieVersion.fromJson(j['version'] as String),
        methodStr = j['method_str'] as String,
        versionStr = j['version_str'] as String,
        methodLabel = j['method_label'] as String,
        versionLabel = j['version_label'] as String,
        displayLabel = j['display_label'] as String,
        sourcePriority = j['source_priority'] as int,
        isEstimate = j['is_estimate'] as bool;
}

/// The result of clamping a calorie value to a plausible range.
class StrideCalorieClampResult {
  final double original;
  final double clamped;
  final bool wasModified;
  final bool isPlausible;

  StrideCalorieClampResult.fromJson(Map<String, dynamic> j)
      : original = (j['original'] as num).toDouble(),
        clamped = (j['clamped'] as num).toDouble(),
        wasModified = j['was_modified'] as bool,
        isPlausible = j['is_plausible'] as bool;
}
