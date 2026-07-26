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

// ─── §10 — Wearable / Health Connect ─────────────────────────────

/// The connection state of a wearable device or the Health Connect
/// platform.
enum StrideWearableConnectionState {
  disconnected,
  connected,
  unreachable,
  healthConnectActive,
  revoked,
  permissionDenied,
  syncing,
  error;

  bool get isDataAvailable => switch (this) {
    StrideWearableConnectionState.connected ||
    StrideWearableConnectionState.healthConnectActive ||
    StrideWearableConnectionState.syncing => true,
    _ => false,
  };

  bool get isRevoked => switch (this) {
    StrideWearableConnectionState.revoked ||
    StrideWearableConnectionState.permissionDenied => true,
    _ => false,
  };

  String get label => switch (this) {
    StrideWearableConnectionState.disconnected => 'No wearable connected',
    StrideWearableConnectionState.connected => 'Watch connected',
    StrideWearableConnectionState.unreachable => 'Watch unreachable',
    StrideWearableConnectionState.healthConnectActive => 'Health Connect active',
    StrideWearableConnectionState.revoked => 'Health Connect revoked',
    StrideWearableConnectionState.permissionDenied => 'Health Connect denied',
    StrideWearableConnectionState.syncing => 'Syncing from watch',
    StrideWearableConnectionState.error => 'Wearable error',
  };

  static StrideWearableConnectionState fromJson(String s) => switch (s) {
    'disconnected' => StrideWearableConnectionState.disconnected,
    'connected' => StrideWearableConnectionState.connected,
    'unreachable' => StrideWearableConnectionState.unreachable,
    'health_connect_active' => StrideWearableConnectionState.healthConnectActive,
    'revoked' => StrideWearableConnectionState.revoked,
    'permission_denied' => StrideWearableConnectionState.permissionDenied,
    'syncing' => StrideWearableConnectionState.syncing,
    'error' => StrideWearableConnectionState.error,
    _ => StrideWearableConnectionState.disconnected,
  };

  String toJson() => switch (this) {
    StrideWearableConnectionState.disconnected => 'disconnected',
    StrideWearableConnectionState.connected => 'connected',
    StrideWearableConnectionState.unreachable => 'unreachable',
    StrideWearableConnectionState.healthConnectActive => 'health_connect_active',
    StrideWearableConnectionState.revoked => 'revoked',
    StrideWearableConnectionState.permissionDenied => 'permission_denied',
    StrideWearableConnectionState.syncing => 'syncing',
    StrideWearableConnectionState.error => 'error',
  };
}

/// Which data sources are currently available for a given workout.
class StrideSourceAvailability {
  final bool watchConnected;
  final bool healthConnectGranted;
  final bool phoneGpsAvailable;
  final bool phoneStepSensorAvailable;
  final bool phoneHeartRateAvailable;

  StrideSourceAvailability({
    this.watchConnected = false,
    this.healthConnectGranted = false,
    this.phoneGpsAvailable = false,
    this.phoneStepSensorAvailable = false,
    this.phoneHeartRateAvailable = false,
  });

  bool get hasWearable => watchConnected || healthConnectGranted;
  bool get isPhoneOnly => !hasWearable;

  StrideSourceAvailability.fromJson(Map<String, dynamic> j)
      : watchConnected = j['watch_connected'] as bool? ?? false,
        healthConnectGranted = j['health_connect_granted'] as bool? ?? false,
        phoneGpsAvailable = j['phone_gps_available'] as bool? ?? false,
        phoneStepSensorAvailable = j['phone_step_sensor_available'] as bool? ?? false,
        phoneHeartRateAvailable = j['phone_heart_rate_available'] as bool? ?? false;

  Map<String, dynamic> toJson() => {
    'watch_connected': watchConnected,
    'health_connect_granted': healthConnectGranted,
    'phone_gps_available': phoneGpsAvailable,
    'phone_step_sensor_available': phoneStepSensorAvailable,
    'phone_heart_rate_available': phoneHeartRateAvailable,
  };
}

/// Why a data source was revoked or became unavailable.
enum StrideRevocationReason {
  userRevoked,
  userDenied,
  watchDisconnected,
  watchOutOfRange,
  apiError,
  systemRevoked;

  bool get isUserInitiated => switch (this) {
    StrideRevocationReason.userRevoked ||
    StrideRevocationReason.userDenied => true,
    _ => false,
  };

  bool get isRecoverable => switch (this) {
    StrideRevocationReason.watchOutOfRange ||
    StrideRevocationReason.apiError => true,
    _ => false,
  };

  String get label => switch (this) {
    StrideRevocationReason.userRevoked => 'User revoked Health Connect permissions',
    StrideRevocationReason.userDenied => 'User denied permission request',
    StrideRevocationReason.watchDisconnected => 'Watch disconnected',
    StrideRevocationReason.watchOutOfRange => 'Watch out of range',
    StrideRevocationReason.apiError => 'API error',
    StrideRevocationReason.systemRevoked => 'System revoked permissions',
  };

  static StrideRevocationReason fromJson(String s) => switch (s) {
    'user_revoked' => StrideRevocationReason.userRevoked,
    'user_denied' => StrideRevocationReason.userDenied,
    'watch_disconnected' => StrideRevocationReason.watchDisconnected,
    'watch_out_of_range' => StrideRevocationReason.watchOutOfRange,
    'api_error' => StrideRevocationReason.apiError,
    'system_revoked' => StrideRevocationReason.systemRevoked,
    _ => StrideRevocationReason.apiError,
  };

  String toJson() => switch (this) {
    StrideRevocationReason.userRevoked => 'user_revoked',
    StrideRevocationReason.userDenied => 'user_denied',
    StrideRevocationReason.watchDisconnected => 'watch_disconnected',
    StrideRevocationReason.watchOutOfRange => 'watch_out_of_range',
    StrideRevocationReason.apiError => 'api_error',
    StrideRevocationReason.systemRevoked => 'system_revoked',
  };
}

/// The sensor source type (from Rust `SensorSource` enum).
enum StrideSensorSource {
  phoneGps,
  phoneStepSensor,
  phoneAccelerometer,
  wearOs,
  healthConnect,
  manualEntry,
  serverCorrected,
  estimated;

  int get priority => switch (this) {
    StrideSensorSource.healthConnect => 100,
    StrideSensorSource.wearOs => 90,
    StrideSensorSource.phoneStepSensor => 70,
    StrideSensorSource.phoneGps => 60,
    StrideSensorSource.phoneAccelerometer => 50,
    StrideSensorSource.serverCorrected => 40,
    StrideSensorSource.manualEntry => 30,
    StrideSensorSource.estimated => 10,
  };

  static StrideSensorSource fromJson(String s) => switch (s) {
    'phone_gps' => StrideSensorSource.phoneGps,
    'phone_step_sensor' => StrideSensorSource.phoneStepSensor,
    'phone_accelerometer' => StrideSensorSource.phoneAccelerometer,
    'wear_os' => StrideSensorSource.wearOs,
    'health_connect' => StrideSensorSource.healthConnect,
    'manual_entry' => StrideSensorSource.manualEntry,
    'server_corrected' => StrideSensorSource.serverCorrected,
    'estimated' => StrideSensorSource.estimated,
    _ => StrideSensorSource.estimated,
  };

  String toJson() => switch (this) {
    StrideSensorSource.phoneGps => 'phone_gps',
    StrideSensorSource.phoneStepSensor => 'phone_step_sensor',
    StrideSensorSource.phoneAccelerometer => 'phone_accelerometer',
    StrideSensorSource.wearOs => 'wear_os',
    StrideSensorSource.healthConnect => 'health_connect',
    StrideSensorSource.manualEntry => 'manual_entry',
    StrideSensorSource.serverCorrected => 'server_corrected',
    StrideSensorSource.estimated => 'estimated',
  };
}

/// The metric type for dedup decisions.
enum StrideMetricType {
  heartRate,
  steps,
  distance,
  calories,
  elevation;

  static StrideMetricType fromJson(String s) => switch (s) {
    'heart_rate' => StrideMetricType.heartRate,
    'steps' => StrideMetricType.steps,
    'distance' => StrideMetricType.distance,
    'calories' => StrideMetricType.calories,
    'elevation' => StrideMetricType.elevation,
    _ => StrideMetricType.heartRate,
  };

  String toJson() => name;
}

/// A record of a source revocation.
class StrideSourceRevocationRecord {
  final StrideSensorSource source;
  final int revokedAtMs;
  final StrideRevocationReason reason;
  final String message;

  StrideSourceRevocationRecord.fromJson(Map<String, dynamic> j)
      : source = StrideSensorSource.fromJson(j['source'] as String),
        revokedAtMs = j['revoked_at_ms'] as int,
        reason = StrideRevocationReason.fromJson(j['reason'] as String),
        message = j['message'] as String;

  Map<String, dynamic> toJson() => {
    'source': source.toJson(),
    'revoked_at_ms': revokedAtMs,
    'reason': reason.toJson(),
    'message': message,
  };
}

/// The sync status of wearable data relative to the phone app.
enum StrideWearableSyncStatus {
  noWearable,
  inSync,
  syncing,
  stale,
  syncError,
  pending;

  bool get isCurrent => switch (this) {
    StrideWearableSyncStatus.inSync ||
    StrideWearableSyncStatus.syncing => true,
    _ => false,
  };

  String get label => switch (this) {
    StrideWearableSyncStatus.noWearable => 'No wearable paired',
    StrideWearableSyncStatus.inSync => 'Wearable in sync',
    StrideWearableSyncStatus.syncing => 'Syncing from wearable',
    StrideWearableSyncStatus.stale => 'Wearable data is stale',
    StrideWearableSyncStatus.syncError => 'Wearable sync error',
    StrideWearableSyncStatus.pending => 'Wearable sync pending',
  };

  static StrideWearableSyncStatus fromJson(String s) => switch (s) {
    'no_wearable' => StrideWearableSyncStatus.noWearable,
    'in_sync' => StrideWearableSyncStatus.inSync,
    'syncing' => StrideWearableSyncStatus.syncing,
    'stale' => StrideWearableSyncStatus.stale,
    'sync_error' => StrideWearableSyncStatus.syncError,
    'pending' => StrideWearableSyncStatus.pending,
    _ => StrideWearableSyncStatus.noWearable,
  };

  String toJson() => switch (this) {
    StrideWearableSyncStatus.noWearable => 'no_wearable',
    StrideWearableSyncStatus.inSync => 'in_sync',
    StrideWearableSyncStatus.syncing => 'syncing',
    StrideWearableSyncStatus.stale => 'stale',
    StrideWearableSyncStatus.syncError => 'sync_error',
    StrideWearableSyncStatus.pending => 'pending',
  };
}

/// Configuration for wearable sync staleness detection.
class StrideWearableSyncConfig {
  final int staleThresholdMs;

  StrideWearableSyncConfig({this.staleThresholdMs = 300000});

  StrideWearableSyncConfig.fromJson(Map<String, dynamic> j)
      : staleThresholdMs = j['stale_threshold_ms'] as int? ?? 300000;

  Map<String, dynamic> toJson() => {
    'stale_threshold_ms': staleThresholdMs,
  };
}

/// The consent state for Health Connect permissions.
enum StrideHealthConnectConsentState {
  notRequested,
  requesting,
  granted,
  denied,
  revoked;

  bool get canRead => this == StrideHealthConnectConsentState.granted;

  bool get shouldReRequest =>
      this == StrideHealthConnectConsentState.notRequested;

  String get label => switch (this) {
    StrideHealthConnectConsentState.notRequested => 'Health Connect not yet set up',
    StrideHealthConnectConsentState.requesting => 'Requesting Health Connect permissions',
    StrideHealthConnectConsentState.granted => 'Health Connect granted',
    StrideHealthConnectConsentState.denied => 'Health Connect denied',
    StrideHealthConnectConsentState.revoked => 'Health Connect revoked',
  };

  static StrideHealthConnectConsentState fromJson(String s) => switch (s) {
    'not_requested' => StrideHealthConnectConsentState.notRequested,
    'requesting' => StrideHealthConnectConsentState.requesting,
    'granted' => StrideHealthConnectConsentState.granted,
    'denied' => StrideHealthConnectConsentState.denied,
    'revoked' => StrideHealthConnectConsentState.revoked,
    _ => StrideHealthConnectConsentState.notRequested,
  };

  String toJson() => name;
}

/// Information about a paired wearable device.
class StrideWearableDeviceInfo {
  final String manufacturer;
  final String model;
  final String deviceId;
  final bool supportsHealthConnect;
  final int? batteryLevel;

  StrideWearableDeviceInfo.fromJson(Map<String, dynamic> j)
      : manufacturer = j['manufacturer'] as String,
        model = j['model'] as String,
        deviceId = j['device_id'] as String,
        supportsHealthConnect = j['supports_health_connect'] as bool? ?? false,
        batteryLevel = j['battery_level'] as int?;

  String get displayName => '$manufacturer $model';

  Map<String, dynamic> toJson() => {
    'manufacturer': manufacturer,
    'model': model,
    'device_id': deviceId,
    'supports_health_connect': supportsHealthConnect,
    'battery_level': batteryLevel,
  };
}

/// The result of evaluating the no-watch fallback: which mode the app
/// should operate in and what sources to use.
class StrideFallbackDecision {
  final StrideWearableConnectionState connectionState;
  final StrideSourceAvailability availableSources;
  final StrideSensorSource stepSource;
  final StrideSensorSource distanceSource;
  final StrideSensorSource? heartRateSource;
  final bool isPhoneOnly;
  final String message;

  StrideFallbackDecision.fromJson(Map<String, dynamic> j)
      : connectionState = StrideWearableConnectionState.fromJson(
          j['connection_state'] as String,
        ),
        availableSources = StrideSourceAvailability.fromJson(
          j['available_sources'] as Map<String, dynamic>,
        ),
        stepSource = StrideSensorSource.fromJson(j['step_source'] as String),
        distanceSource = StrideSensorSource.fromJson(j['distance_source'] as String),
        heartRateSource = j['heart_rate_source'] != null
            ? StrideSensorSource.fromJson(j['heart_rate_source'] as String)
            : null,
        isPhoneOnly = j['is_phone_only'] as bool,
        message = j['message'] as String;
}

/// The result of evaluating sync status.
class StrideWearableSyncStatusResult {
  final StrideWearableSyncStatus syncStatus;
  final String label;
  final bool isCurrent;

  StrideWearableSyncStatusResult.fromJson(Map<String, dynamic> j)
      : syncStatus = StrideWearableSyncStatus.fromJson(j['sync_status'] as String),
        label = j['label'] as String,
        isCurrent = j['is_current'] as bool;
}

/// The result of deduplicating a source.
class StrideDeduplicateSourceResult {
  final StrideSensorSource winner;
  final int winnerPriority;

  StrideDeduplicateSourceResult.fromJson(Map<String, dynamic> j)
      : winner = StrideSensorSource.fromJson(j['winner'] as String),
        winnerPriority = j['winner_priority'] as int;
}

/// The result of processing a consent request.
class StrideConsentResult {
  final StrideHealthConnectConsentState consentState;
  final String label;
  final bool canRead;

  StrideConsentResult.fromJson(Map<String, dynamic> j)
      : consentState = StrideHealthConnectConsentState.fromJson(
          j['consent_state'] as String,
        ),
        label = j['label'] as String,
        canRead = j['can_read'] as bool;
}

/// A full wearable status snapshot for the UI.
class StrideWearableStatus {
  final StrideWearableConnectionState connectionState;
  final StrideHealthConnectConsentState consentState;
  final StrideWearableSyncStatus syncStatus;
  final StrideSourceAvailability availability;
  final StrideFallbackDecision fallback;
  final StrideWearableDeviceInfo? device;
  final List<StrideSourceRevocationRecord> revocations;
  final int? lastSyncMs;

  StrideWearableStatus.fromJson(Map<String, dynamic> j)
      : connectionState = StrideWearableConnectionState.fromJson(
          j['connection_state'] as String,
        ),
        consentState = StrideHealthConnectConsentState.fromJson(
          j['consent_state'] as String,
        ),
        syncStatus = StrideWearableSyncStatus.fromJson(
          j['sync_status'] as String,
        ),
        availability = StrideSourceAvailability.fromJson(
          j['availability'] as Map<String, dynamic>,
        ),
        fallback = StrideFallbackDecision.fromJson(
          j['fallback'] as Map<String, dynamic>,
        ),
        device = j['device'] != null
            ? StrideWearableDeviceInfo.fromJson(j['device'] as Map<String, dynamic>)
            : null,
        revocations = (j['revocations'] as List? ?? [])
            .map((e) => StrideSourceRevocationRecord.fromJson(e as Map<String, dynamic>))
            .toList(),
        lastSyncMs = j['last_sync_ms'] as int?;
}

// ─── §11 — Music system models ─────────────────────────────────────

/// The playback state of the music player (8-state state machine).
enum StridePlaybackState {
  idle,
  ready,
  playing,
  paused,
  buffering,
  ended,
  stopped,
  error;

  static StridePlaybackState fromJson(String s) => switch (s) {
    'idle' => StridePlaybackState.idle,
    'ready' => StridePlaybackState.ready,
    'playing' => StridePlaybackState.playing,
    'paused' => StridePlaybackState.paused,
    'buffering' => StridePlaybackState.buffering,
    'ended' => StridePlaybackState.ended,
    'stopped' => StridePlaybackState.stopped,
    'error' => StridePlaybackState.error,
    _ => StridePlaybackState.idle,
  };

  String toJson() => switch (this) {
    StridePlaybackState.idle => 'idle',
    StridePlaybackState.ready => 'ready',
    StridePlaybackState.playing => 'playing',
    StridePlaybackState.paused => 'paused',
    StridePlaybackState.buffering => 'buffering',
    StridePlaybackState.ended => 'ended',
    StridePlaybackState.stopped => 'stopped',
    StridePlaybackState.error => 'error',
  };
}

/// A command to the playback state machine.
enum StridePlaybackCommand {
  play,
  pause,
  resume,
  skip,
  previous,
  stop,
  seek;

  static StridePlaybackCommand fromJson(String s) => switch (s) {
    'play' => StridePlaybackCommand.play,
    'pause' => StridePlaybackCommand.pause,
    'resume' => StridePlaybackCommand.resume,
    'skip' => StridePlaybackCommand.skip,
    'previous' => StridePlaybackCommand.previous,
    'stop' => StridePlaybackCommand.stop,
    'seek' => StridePlaybackCommand.seek,
    _ => StridePlaybackCommand.play,
  };

  String toJson() => switch (this) {
    StridePlaybackCommand.play => 'play',
    StridePlaybackCommand.pause => 'pause',
    StridePlaybackCommand.resume => 'resume',
    StridePlaybackCommand.skip => 'skip',
    StridePlaybackCommand.previous => 'previous',
    StridePlaybackCommand.stop => 'stop',
    StridePlaybackCommand.seek => 'seek',
  };
}

/// The result of a playback state transition.
class StrideTransitionResult {
  final StridePlaybackState previousState;
  final StridePlaybackState newState;
  final bool accepted;
  final String message;

  StrideTransitionResult.fromJson(Map<String, dynamic> j)
      : previousState = StridePlaybackState.fromJson(j['previous_state'] as String),
        newState = StridePlaybackState.fromJson(j['new_state'] as String),
        accepted = j['accepted'] as bool,
        message = j['message'] as String;
}

/// The audio focus state (Android AudioManager focus states).
enum StrideAudioFocusState {
  noFocus,
  focused,
  ducking,
  transientPause,
  lost;

  bool get canPlay => switch (this) {
    StrideAudioFocusState.focused => true,
    StrideAudioFocusState.ducking => true,
    _ => false,
  };

  double get volumeMultiplier => switch (this) {
    StrideAudioFocusState.focused => 1.0,
    StrideAudioFocusState.ducking => 0.3,
    _ => 0.0,
  };

  String get label => switch (this) {
    StrideAudioFocusState.noFocus => 'No audio focus',
    StrideAudioFocusState.focused => 'Audio focused',
    StrideAudioFocusState.ducking => 'Ducking (low volume)',
    StrideAudioFocusState.transientPause => 'Transient pause',
    StrideAudioFocusState.lost => 'Audio focus lost',
  };

  static StrideAudioFocusState fromJson(String s) => switch (s) {
    'no_focus' => StrideAudioFocusState.noFocus,
    'focused' => StrideAudioFocusState.focused,
    'ducking' => StrideAudioFocusState.ducking,
    'transient_pause' => StrideAudioFocusState.transientPause,
    'lost' => StrideAudioFocusState.lost,
    _ => StrideAudioFocusState.noFocus,
  };

  String toJson() => switch (this) {
    StrideAudioFocusState.noFocus => 'no_focus',
    StrideAudioFocusState.focused => 'focused',
    StrideAudioFocusState.ducking => 'ducking',
    StrideAudioFocusState.transientPause => 'transient_pause',
    StrideAudioFocusState.lost => 'lost',
  };
}

/// An audio focus event from the Android AudioManager.
enum StrideAudioFocusEvent {
  gain,
  duck,
  transientPause,
  loss;

  static StrideAudioFocusEvent fromJson(String s) => switch (s) {
    'gain' => StrideAudioFocusEvent.gain,
    'duck' => StrideAudioFocusEvent.duck,
    'transient_pause' => StrideAudioFocusEvent.transientPause,
    'loss' => StrideAudioFocusEvent.loss,
    _ => StrideAudioFocusEvent.gain,
  };

  String toJson() => switch (this) {
    StrideAudioFocusEvent.gain => 'gain',
    StrideAudioFocusEvent.duck => 'duck',
    StrideAudioFocusEvent.transientPause => 'transient_pause',
    StrideAudioFocusEvent.loss => 'loss',
  };
}

/// A playback action recommended by the audio-focus handler.
enum StridePlaybackAction {
  continuePlaying,
  resume,
  pause,
  duck,
  stop;

  static StridePlaybackAction fromJson(String s) => switch (s) {
    'continue' => StridePlaybackAction.continuePlaying,
    'resume' => StridePlaybackAction.resume,
    'pause' => StridePlaybackAction.pause,
    'duck' => StridePlaybackAction.duck,
    'stop' => StridePlaybackAction.stop,
    _ => StridePlaybackAction.continuePlaying,
  };

  String toJson() => switch (this) {
    StridePlaybackAction.continuePlaying => 'continue',
    StridePlaybackAction.resume => 'resume',
    StridePlaybackAction.pause => 'pause',
    StridePlaybackAction.duck => 'duck',
    StridePlaybackAction.stop => 'stop',
  };
}

/// The result of an audio-focus event.
class StrideAudioFocusResult {
  final StrideAudioFocusState newFocus;
  final StridePlaybackAction action;
  final bool canPlay;
  final double volumeMultiplier;
  final String focusLabel;

  StrideAudioFocusResult.fromJson(Map<String, dynamic> j)
      : newFocus = StrideAudioFocusState.fromJson(j['new_focus'] as String),
        action = StridePlaybackAction.fromJson(j['action'] as String),
        canPlay = j['can_play'] as bool,
        volumeMultiplier = (j['volume_multiplier'] as num).toDouble(),
        focusLabel = j['focus_label'] as String;
}

/// The coaching-interop state (coordinating music with coaching prompts).
enum StrideCoachingInteropState {
  inactive,
  coachingActive,
  coachingFinished;

  static StrideCoachingInteropState fromJson(String s) => switch (s) {
    'inactive' => StrideCoachingInteropState.inactive,
    'coaching_active' => StrideCoachingInteropState.coachingActive,
    'coaching_finished' => StrideCoachingInteropState.coachingFinished,
    _ => StrideCoachingInteropState.inactive,
  };

  String toJson() => switch (this) {
    StrideCoachingInteropState.inactive => 'inactive',
    StrideCoachingInteropState.coachingActive => 'coaching_active',
    StrideCoachingInteropState.coachingFinished => 'coaching_finished',
  };
}

/// A coaching-interop request.
enum StrideCoachingRequest {
  promptStarting,
  promptFinished,
  promptCancelled;

  static StrideCoachingRequest fromJson(String s) => switch (s) {
    'prompt_starting' => StrideCoachingRequest.promptStarting,
    'prompt_finished' => StrideCoachingRequest.promptFinished,
    'prompt_cancelled' => StrideCoachingRequest.promptCancelled,
    _ => StrideCoachingRequest.promptStarting,
  };

  String toJson() => switch (this) {
    StrideCoachingRequest.promptStarting => 'prompt_starting',
    StrideCoachingRequest.promptFinished => 'prompt_finished',
    StrideCoachingRequest.promptCancelled => 'prompt_cancelled',
  };
}

/// The result of a coaching-interop coordination request.
class StrideCoachingInteropResult {
  final StrideCoachingInteropState state;
  final StridePlaybackAction musicAction;
  final bool wasPlayingBefore;
  final String message;

  StrideCoachingInteropResult.fromJson(Map<String, dynamic> j)
      : state = StrideCoachingInteropState.fromJson(j['state'] as String),
        musicAction = StridePlaybackAction.fromJson(j['music_action'] as String),
        wasPlayingBefore = j['was_playing_before'] as bool,
        message = j['message'] as String;
}

/// The music source (where tracks come from).
enum StrideMusicSource {
  localDevice,
  internetRadio,
  aiCurated;

  bool get requiresNetwork => switch (this) {
    StrideMusicSource.localDevice => false,
    _ => true,
  };

  String get label => switch (this) {
    StrideMusicSource.localDevice => 'Local device',
    StrideMusicSource.internetRadio => 'Internet radio',
    StrideMusicSource.aiCurated => 'AI-curated',
  };

  static StrideMusicSource fromJson(String s) => switch (s) {
    'local_device' => StrideMusicSource.localDevice,
    'internet_radio' => StrideMusicSource.internetRadio,
    'ai_curated' => StrideMusicSource.aiCurated,
    _ => StrideMusicSource.localDevice,
  };

  String toJson() => switch (this) {
    StrideMusicSource.localDevice => 'local_device',
    StrideMusicSource.internetRadio => 'internet_radio',
    StrideMusicSource.aiCurated => 'ai_curated',
  };
}

/// The music mode (manual or AI-curated).
enum StrideMusicMode {
  manual,
  ai;

  static StrideMusicMode fromJson(String s) => switch (s) {
    'manual' => StrideMusicMode.manual,
    'ai' => StrideMusicMode.ai,
    _ => StrideMusicMode.manual,
  };

  String toJson() => switch (this) {
    StrideMusicMode.manual => 'manual',
    StrideMusicMode.ai => 'ai',
  };
}

/// The network state for music streaming.
enum StrideNetworkState {
  connected,
  weak,
  lost;

  static StrideNetworkState fromJson(String s) => switch (s) {
    'connected' => StrideNetworkState.connected,
    'weak' => StrideNetworkState.weak,
    'lost' => StrideNetworkState.lost,
    _ => StrideNetworkState.connected,
  };

  String toJson() => switch (this) {
    StrideNetworkState.connected => 'connected',
    StrideNetworkState.weak => 'weak',
    StrideNetworkState.lost => 'lost',
  };
}

/// The decision when the network is lost during streaming.
class StrideNetworkLossDecision {
  final bool shouldSwitchToLocal;
  final bool shouldBuffer;
  final bool shouldStop;
  final String message;
  final StrideMusicSource fallbackSource;

  StrideNetworkLossDecision.fromJson(Map<String, dynamic> j)
      : shouldSwitchToLocal = j['should_switch_to_local'] as bool,
        shouldBuffer = j['should_buffer'] as bool,
        shouldStop = j['should_stop'] as bool,
        message = j['message'] as String,
        fallbackSource =
            StrideMusicSource.fromJson(j['fallback_source'] as String);
}

/// A single track in a playlist.
class StrideTrack {
  final String id;
  final String title;
  final String artist;
  final String genre;
  final int durationMs;
  final StrideMusicSource source;

  StrideTrack.fromJson(Map<String, dynamic> j)
      : id = j['id'] as String,
        title = j['title'] as String,
        artist = j['artist'] as String,
        genre = j['genre'] as String,
        durationMs = j['duration_ms'] as int,
        source = StrideMusicSource.fromJson(j['source'] as String);

  Map<String, dynamic> toJson() => {
    'id': id,
    'title': title,
    'artist': artist,
    'genre': genre,
    'duration_ms': durationMs,
    'source': source.toJson(),
  };
}

/// A playlist with its tracks and metadata.
class StridePlaylist {
  final String id;
  final String name;
  final List<StrideTrack> tracks;
  final StrideMusicSource source;
  final StrideMusicMode mode;

  StridePlaylist.fromJson(Map<String, dynamic> j)
      : id = j['id'] as String,
        name = j['name'] as String,
        tracks = (j['tracks'] as List? ?? [])
            .map((e) => StrideTrack.fromJson(e as Map<String, dynamic>))
            .toList(),
        source = StrideMusicSource.fromJson(j['source'] as String),
        mode = StrideMusicMode.fromJson(j['mode'] as String);

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'tracks': tracks.map((t) => t.toJson()).toList(),
    'source': source.toJson(),
    'mode': mode.toJson(),
  };
}

/// The result of filtering blocked content from a playlist.
class StrideFilterBlockedResult {
  final StridePlaylist filteredPlaylist;
  final int removedCount;

  StrideFilterBlockedResult.fromJson(Map<String, dynamic> j)
      : filteredPlaylist =
            StridePlaylist.fromJson(j['filtered_playlist'] as Map<String, dynamic>),
        removedCount = j['removed_count'] as int;
}

/// The type of feedback a user gave for a track.
enum StrideTrackFeedback {
  liked,
  disliked,
  skipped,
  completed;

  bool get isPositive => switch (this) {
    StrideTrackFeedback.liked || StrideTrackFeedback.completed => true,
    _ => false,
  };

  static StrideTrackFeedback fromJson(String s) => switch (s) {
    'liked' => StrideTrackFeedback.liked,
    'disliked' => StrideTrackFeedback.disliked,
    'skipped' => StrideTrackFeedback.skipped,
    'completed' => StrideTrackFeedback.completed,
    _ => StrideTrackFeedback.completed,
  };

  String toJson() => switch (this) {
    StrideTrackFeedback.liked => 'liked',
    StrideTrackFeedback.disliked => 'disliked',
    StrideTrackFeedback.skipped => 'skipped',
    StrideTrackFeedback.completed => 'completed',
  };
}

/// A record of a user's feedback on a track.
class StrideFeedbackRecord {
  final String trackId;
  final String trackArtist;
  final String trackGenre;
  final StrideTrackFeedback feedback;
  final int recordedAtMs;

  StrideFeedbackRecord.fromJson(Map<String, dynamic> j)
      : trackId = j['track_id'] as String,
        trackArtist = j['track_artist'] as String,
        trackGenre = j['track_genre'] as String,
        feedback = StrideTrackFeedback.fromJson(j['feedback'] as String),
        recordedAtMs = j['recorded_at_ms'] as int;

  Map<String, dynamic> toJson() => {
    'track_id': trackId,
    'track_artist': trackArtist,
    'track_genre': trackGenre,
    'feedback': feedback.toJson(),
    'recorded_at_ms': recordedAtMs,
  };
}

/// The result of a should-recommend check.
class StrideShouldRecommendResult {
  final bool shouldRecommend;

  StrideShouldRecommendResult.fromJson(Map<String, dynamic> j)
      : shouldRecommend = j['should_recommend'] as bool;
}

/// The source of a remote control command.
enum StrideRemoteControlSource {
  inApp,
  lockScreen,
  bluetoothHeadset,
  wearOs,
  notification;

  static StrideRemoteControlSource fromJson(String s) => switch (s) {
    'in_app' => StrideRemoteControlSource.inApp,
    'lock_screen' => StrideRemoteControlSource.lockScreen,
    'bluetooth_headset' => StrideRemoteControlSource.bluetoothHeadset,
    'wear_os' => StrideRemoteControlSource.wearOs,
    'notification' => StrideRemoteControlSource.notification,
    _ => StrideRemoteControlSource.inApp,
  };

  String toJson() => switch (this) {
    StrideRemoteControlSource.inApp => 'in_app',
    StrideRemoteControlSource.lockScreen => 'lock_screen',
    StrideRemoteControlSource.bluetoothHeadset => 'bluetooth_headset',
    StrideRemoteControlSource.wearOs => 'wear_os',
    StrideRemoteControlSource.notification => 'notification',
  };
}

/// A full music status snapshot for the UI.
class StrideMusicStatus {
  final StridePlaybackState playbackState;
  final StrideAudioFocusState audioFocus;
  final StrideCoachingInteropState coachingInterop;
  final StrideMusicSource musicSource;
  final StrideMusicMode musicMode;
  final StrideNetworkState network;
  final double volumeMultiplier;
  final int? currentTrackIndex;
  final int playlistTrackCount;
  final int playlistTotalDurationMs;

  StrideMusicStatus.fromJson(Map<String, dynamic> j)
      : playbackState = StridePlaybackState.fromJson(j['playback_state'] as String),
        audioFocus = StrideAudioFocusState.fromJson(j['audio_focus'] as String),
        coachingInterop =
            StrideCoachingInteropState.fromJson(j['coaching_interop'] as String),
        musicSource = StrideMusicSource.fromJson(j['music_source'] as String),
        musicMode = StrideMusicMode.fromJson(j['music_mode'] as String),
        network = StrideNetworkState.fromJson(j['network'] as String),
        volumeMultiplier = (j['volume_multiplier'] as num).toDouble(),
        currentTrackIndex = j['current_track_index'] as int?,
        playlistTrackCount = j['playlist_track_count'] as int,
        playlistTotalDurationMs = j['playlist_total_duration_ms'] as int;
}

// ─── §12 — Background execution models ──────────────────────────────

/// The state of the Android foreground workout service.
enum StrideForegroundServiceState {
  stopped,
  starting,
  running,
  paused,
  stopping;

  bool get isAlive => switch (this) {
    StrideForegroundServiceState.starting ||
    StrideForegroundServiceState.running ||
    StrideForegroundServiceState.paused => true,
    _ => false,
  };

  bool get isTracking => switch (this) {
    StrideForegroundServiceState.starting ||
    StrideForegroundServiceState.running => true,
    _ => false,
  };

  String get label => switch (this) {
    StrideForegroundServiceState.stopped => 'Workout stopped',
    StrideForegroundServiceState.starting => 'Starting workout…',
    StrideForegroundServiceState.running => 'Workout in progress',
    StrideForegroundServiceState.paused => 'Workout paused',
    StrideForegroundServiceState.stopping => 'Stopping workout…',
  };

  static StrideForegroundServiceState fromJson(String s) => switch (s) {
    'stopped' => StrideForegroundServiceState.stopped,
    'starting' => StrideForegroundServiceState.starting,
    'running' => StrideForegroundServiceState.running,
    'paused' => StrideForegroundServiceState.paused,
    'stopping' => StrideForegroundServiceState.stopping,
    _ => StrideForegroundServiceState.stopped,
  };

  String toJson() => switch (this) {
    StrideForegroundServiceState.stopped => 'stopped',
    StrideForegroundServiceState.starting => 'starting',
    StrideForegroundServiceState.running => 'running',
    StrideForegroundServiceState.paused => 'paused',
    StrideForegroundServiceState.stopping => 'stopping',
  };
}

/// A command to the foreground service state machine.
enum StrideForegroundServiceCommand {
  start,
  pause,
  resume,
  stop;

  static StrideForegroundServiceCommand fromJson(String s) => switch (s) {
    'start' => StrideForegroundServiceCommand.start,
    'pause' => StrideForegroundServiceCommand.pause,
    'resume' => StrideForegroundServiceCommand.resume,
    'stop' => StrideForegroundServiceCommand.stop,
    _ => StrideForegroundServiceCommand.start,
  };

  String toJson() => switch (this) {
    StrideForegroundServiceCommand.start => 'start',
    StrideForegroundServiceCommand.pause => 'pause',
    StrideForegroundServiceCommand.resume => 'resume',
    StrideForegroundServiceCommand.stop => 'stop',
  };
}

/// The result of a foreground service state transition.
class StrideForegroundServiceTransition {
  final StrideForegroundServiceState previousState;
  final StrideForegroundServiceState newState;
  final bool accepted;
  final String notificationText;
  final bool shouldKeepServiceAlive;
  final String message;

  StrideForegroundServiceTransition.fromJson(Map<String, dynamic> j)
      : previousState = StrideForegroundServiceState.fromJson(j['previous_state'] as String),
        newState = StrideForegroundServiceState.fromJson(j['new_state'] as String),
        accepted = j['accepted'] as bool,
        notificationText = j['notification_text'] as String,
        shouldKeepServiceAlive = j['should_keep_service_alive'] as bool,
        message = j['message'] as String;
}

/// The phase of the workout, which affects checkpoint frequency.
enum StrideWorkoutPhase {
  active,
  paused,
  ending;

  static StrideWorkoutPhase fromJson(String s) => switch (s) {
    'active' => StrideWorkoutPhase.active,
    'paused' => StrideWorkoutPhase.paused,
    'ending' => StrideWorkoutPhase.ending,
    _ => StrideWorkoutPhase.active,
  };

  String toJson() => switch (this) {
    StrideWorkoutPhase.active => 'active',
    StrideWorkoutPhase.paused => 'paused',
    StrideWorkoutPhase.ending => 'ending',
  };
}

/// Battery state relevant to checkpoint scheduling.
class StrideCheckpointBatteryContext {
  final int batteryPercent;
  final bool batterySaverEnabled;
  final bool isCharging;

  StrideCheckpointBatteryContext({
    required this.batteryPercent,
    required this.batterySaverEnabled,
    required this.isCharging,
  });

  StrideCheckpointBatteryContext.fromJson(Map<String, dynamic> j)
      : batteryPercent = j['battery_percent'] as int,
        batterySaverEnabled = j['battery_saver_enabled'] as bool,
        isCharging = j['is_charging'] as bool;

  Map<String, dynamic> toJson() => {
    'battery_percent': batteryPercent,
    'battery_saver_enabled': batterySaverEnabled,
    'is_charging': isCharging,
  };
}

/// The checkpoint write schedule.
class StrideCheckpointSchedule {
  final int intervalMs;
  final int maxDataLossMs;
  final String reason;

  StrideCheckpointSchedule.fromJson(Map<String, dynamic> j)
      : intervalMs = j['interval_ms'] as int,
        maxDataLossMs = j['max_data_loss_ms'] as int,
        reason = j['reason'] as String;
}

/// User preference for background tracking.
enum StrideBackgroundTrackingPreference {
  unset,
  enabled,
  disabled;

  static StrideBackgroundTrackingPreference fromJson(String s) => switch (s) {
    'unset' => StrideBackgroundTrackingPreference.unset,
    'enabled' => StrideBackgroundTrackingPreference.enabled,
    'disabled' => StrideBackgroundTrackingPreference.disabled,
    _ => StrideBackgroundTrackingPreference.unset,
  };

  String toJson() => switch (this) {
    StrideBackgroundTrackingPreference.unset => 'unset',
    StrideBackgroundTrackingPreference.enabled => 'enabled',
    StrideBackgroundTrackingPreference.disabled => 'disabled',
  };
}

/// The full context for a background execution decision.
class StrideBackgroundExecutionContext {
  final StrideBackgroundTrackingPreference preference;
  final bool serviceRunning;
  final bool backgroundLocationGranted;
  final bool workoutActive;
  final StrideCheckpointBatteryContext battery;

  StrideBackgroundExecutionContext({
    required this.preference,
    required this.serviceRunning,
    required this.backgroundLocationGranted,
    required this.workoutActive,
    required this.battery,
  });

  StrideBackgroundExecutionContext.fromJson(Map<String, dynamic> j)
      : preference = StrideBackgroundTrackingPreference.fromJson(j['preference'] as String),
        serviceRunning = j['service_running'] as bool,
        backgroundLocationGranted = j['background_location_granted'] as bool,
        workoutActive = j['workout_active'] as bool,
        battery = StrideCheckpointBatteryContext.fromJson(
            j['battery'] as Map<String, dynamic>);

  Map<String, dynamic> toJson() => {
    'preference': preference.toJson(),
    'service_running': serviceRunning,
    'background_location_granted': backgroundLocationGranted,
    'workout_active': workoutActive,
    'battery': battery.toJson(),
  };
}

/// The decision on whether background execution is permitted.
class StrideBackgroundExecutionDecision {
  final bool allowed;
  final bool keepServiceAlive;
  final String userMessage;
  final bool shouldPromptUser;
  final String reason;

  StrideBackgroundExecutionDecision.fromJson(Map<String, dynamic> j)
      : allowed = j['allowed'] as bool,
        keepServiceAlive = j['keep_service_alive'] as bool,
        userMessage = j['user_message'] as String,
        shouldPromptUser = j['should_prompt_user'] as bool,
        reason = j['reason'] as String;
}

/// What to recommend to the user after a process kill.
enum StrideProcessKillRecommendation {
  resume,
  finishAndSave,
  discard,
  nothingToRecover;

  static StrideProcessKillRecommendation fromJson(String s) => switch (s) {
    'resume' => StrideProcessKillRecommendation.resume,
    'finish_and_save' => StrideProcessKillRecommendation.finishAndSave,
    'discard' => StrideProcessKillRecommendation.discard,
    'nothing_to_recover' => StrideProcessKillRecommendation.nothingToRecover,
    _ => StrideProcessKillRecommendation.nothingToRecover,
  };

  String toJson() => switch (this) {
    StrideProcessKillRecommendation.resume => 'resume',
    StrideProcessKillRecommendation.finishAndSave => 'finish_and_save',
    StrideProcessKillRecommendation.discard => 'discard',
    StrideProcessKillRecommendation.nothingToRecover => 'nothing_to_recover',
  };
}

/// The result of evaluating process-kill recovery.
class StrideProcessKillRecoveryDecision {
  final bool canResume;
  final bool checkpointIsFresh;
  final int estimatedDataLossMs;
  final StrideProcessKillRecommendation recommendation;
  final String message;

  StrideProcessKillRecoveryDecision.fromJson(Map<String, dynamic> j)
      : canResume = j['can_resume'] as bool,
        checkpointIsFresh = j['checkpoint_is_fresh'] as bool,
        estimatedDataLossMs = j['estimated_data_loss_ms'] as int,
        recommendation =
            StrideProcessKillRecommendation.fromJson(j['recommendation'] as String),
        message = j['message'] as String;
}

/// The power mode for background execution.
enum StridePowerMode {
  full,
  reduced,
  critical;

  static StridePowerMode fromJson(String s) => switch (s) {
    'full' => StridePowerMode.full,
    'reduced' => StridePowerMode.reduced,
    'critical' => StridePowerMode.critical,
    _ => StridePowerMode.full,
  };

  String toJson() => switch (this) {
    StridePowerMode.full => 'full',
    StridePowerMode.reduced => 'reduced',
    StridePowerMode.critical => 'critical',
  };
}

/// The result of a power mode decision.
class StrideBackgroundPowerModeResult {
  final StridePowerMode powerMode;
  final int notificationIntervalMs;

  StrideBackgroundPowerModeResult.fromJson(Map<String, dynamic> j)
      : powerMode = StridePowerMode.fromJson(j['power_mode'] as String),
        notificationIntervalMs = j['notification_interval_ms'] as int;
}

/// An interruption event that can affect background tracking.
enum StrideInterruptionEvent {
  screenOff,
  screenOn,
  appBackgrounded,
  appForegrounded,
  incomingCall,
  callEnded,
  lowMemory,
  deviceShutdown;

  static StrideInterruptionEvent fromJson(String s) => switch (s) {
    'screen_off' => StrideInterruptionEvent.screenOff,
    'screen_on' => StrideInterruptionEvent.screenOn,
    'app_backgrounded' => StrideInterruptionEvent.appBackgrounded,
    'app_foregrounded' => StrideInterruptionEvent.appForegrounded,
    'incoming_call' => StrideInterruptionEvent.incomingCall,
    'call_ended' => StrideInterruptionEvent.callEnded,
    'low_memory' => StrideInterruptionEvent.lowMemory,
    'device_shutdown' => StrideInterruptionEvent.deviceShutdown,
    _ => StrideInterruptionEvent.screenOff,
  };

  String toJson() => switch (this) {
    StrideInterruptionEvent.screenOff => 'screen_off',
    StrideInterruptionEvent.screenOn => 'screen_on',
    StrideInterruptionEvent.appBackgrounded => 'app_backgrounded',
    StrideInterruptionEvent.appForegrounded => 'app_foregrounded',
    StrideInterruptionEvent.incomingCall => 'incoming_call',
    StrideInterruptionEvent.callEnded => 'call_ended',
    StrideInterruptionEvent.lowMemory => 'low_memory',
    StrideInterruptionEvent.deviceShutdown => 'device_shutdown',
  };
}

/// The action to take in response to an interruption.
enum StrideInterruptionAction {
  continueTracking,
  pause,
  writeCheckpointAndContinue,
  writeCheckpointAndStop,
  reducePower;

  static StrideInterruptionAction fromJson(String s) => switch (s) {
    'continue' => StrideInterruptionAction.continueTracking,
    'pause' => StrideInterruptionAction.pause,
    'write_checkpoint_and_continue' => StrideInterruptionAction.writeCheckpointAndContinue,
    'write_checkpoint_and_stop' => StrideInterruptionAction.writeCheckpointAndStop,
    'reduce_power' => StrideInterruptionAction.reducePower,
    _ => StrideInterruptionAction.continueTracking,
  };

  String toJson() => switch (this) {
    StrideInterruptionAction.continueTracking => 'continue',
    StrideInterruptionAction.pause => 'pause',
    StrideInterruptionAction.writeCheckpointAndContinue => 'write_checkpoint_and_continue',
    StrideInterruptionAction.writeCheckpointAndStop => 'write_checkpoint_and_stop',
    StrideInterruptionAction.reducePower => 'reduce_power',
  };
}

/// The result of a battery-use assessment.
class StrideBatteryUseAssessment {
  final bool isAcceptable;
  final double estimatedDrainPerHour;
  final bool recommendBatterySaver;
  final String message;

  StrideBatteryUseAssessment.fromJson(Map<String, dynamic> j)
      : isAcceptable = j['is_acceptable'] as bool,
        estimatedDrainPerHour = (j['estimated_drain_per_hour'] as num).toDouble(),
        recommendBatterySaver = j['recommend_battery_saver'] as bool,
        message = j['message'] as String;
}

/// A full background execution status snapshot for the UI / diagnostics.
class StrideBackgroundStatus {
  final StrideForegroundServiceState serviceState;
  final StridePowerMode powerMode;
  final bool backgroundAllowed;
  final int checkpointIntervalMs;
  final int notificationIntervalMs;
  final StrideBackgroundTrackingPreference preference;
  final double estimatedDrainPerHour;
  final bool isCharging;
  final int batteryPercent;

  StrideBackgroundStatus.fromJson(Map<String, dynamic> j)
      : serviceState =
            StrideForegroundServiceState.fromJson(j['service_state'] as String),
        powerMode = StridePowerMode.fromJson(j['power_mode'] as String),
        backgroundAllowed = j['background_allowed'] as bool,
        checkpointIntervalMs = j['checkpoint_interval_ms'] as int,
        notificationIntervalMs = j['notification_interval_ms'] as int,
        preference =
            StrideBackgroundTrackingPreference.fromJson(j['preference'] as String),
        estimatedDrainPerHour = (j['estimated_drain_per_hour'] as num).toDouble(),
        isCharging = j['is_charging'] as bool,
        batteryPercent = j['battery_percent'] as int;
}

// ─────────────────────────────────────────────────────────────────────────────
// §13 — Notifications
// ─────────────────────────────────────────────────────────────────────────────

/// The category a notification belongs to.
enum StrideNotificationCategory {
  reminders,
  achievements,
  coaching,
  system,
  recovery;

  static StrideNotificationCategory fromJson(String s) => switch (s) {
    'reminders' => StrideNotificationCategory.reminders,
    'achievements' => StrideNotificationCategory.achievements,
    'coaching' => StrideNotificationCategory.coaching,
    'system' => StrideNotificationCategory.system,
    'recovery' => StrideNotificationCategory.recovery,
    _ => StrideNotificationCategory.system,
  };

  String toJson() => switch (this) {
    StrideNotificationCategory.reminders => 'reminders',
    StrideNotificationCategory.achievements => 'achievements',
    StrideNotificationCategory.coaching => 'coaching',
    StrideNotificationCategory.system => 'system',
    StrideNotificationCategory.recovery => 'recovery',
  };
}

/// The specific notification to deliver. Each type maps to a
/// [StrideNotificationCategory] and has a default priority.
enum StrideNotificationType {
  scheduledWorkout,
  planReminder,
  missedWorkout,
  goalMilestone,
  streakWarning,
  deviceDisconnected,
  syncFailed,
  workoutRecovered,
  badgeEarned,
  planUpdated,
  voiceCoaching;

  static StrideNotificationType fromJson(String s) => switch (s) {
    'scheduled_workout' => StrideNotificationType.scheduledWorkout,
    'plan_reminder' => StrideNotificationType.planReminder,
    'missed_workout' => StrideNotificationType.missedWorkout,
    'goal_milestone' => StrideNotificationType.goalMilestone,
    'streak_warning' => StrideNotificationType.streakWarning,
    'device_disconnected' => StrideNotificationType.deviceDisconnected,
    'sync_failed' => StrideNotificationType.syncFailed,
    'workout_recovered' => StrideNotificationType.workoutRecovered,
    'badge_earned' => StrideNotificationType.badgeEarned,
    'plan_updated' => StrideNotificationType.planUpdated,
    'voice_coaching' => StrideNotificationType.voiceCoaching,
    _ => StrideNotificationType.scheduledWorkout,
  };

  String toJson() => switch (this) {
    StrideNotificationType.scheduledWorkout => 'scheduled_workout',
    StrideNotificationType.planReminder => 'plan_reminder',
    StrideNotificationType.missedWorkout => 'missed_workout',
    StrideNotificationType.goalMilestone => 'goal_milestone',
    StrideNotificationType.streakWarning => 'streak_warning',
    StrideNotificationType.deviceDisconnected => 'device_disconnected',
    StrideNotificationType.syncFailed => 'sync_failed',
    StrideNotificationType.workoutRecovered => 'workout_recovered',
    StrideNotificationType.badgeEarned => 'badge_earned',
    StrideNotificationType.planUpdated => 'plan_updated',
    StrideNotificationType.voiceCoaching => 'voice_coaching',
  };
}

/// Notification channel importance level.
enum StrideNotificationPriority {
  high,
  defaultPriority,
  low;

  static StrideNotificationPriority fromJson(String s) => switch (s) {
    'high' => StrideNotificationPriority.high,
    'default' => StrideNotificationPriority.defaultPriority,
    'low' => StrideNotificationPriority.low,
    _ => StrideNotificationPriority.defaultPriority,
  };

  String toJson() => switch (this) {
    StrideNotificationPriority.high => 'high',
    StrideNotificationPriority.defaultPriority => 'default',
    StrideNotificationPriority.low => 'low',
  };
}

/// Whether the user has granted notification permission.
enum StrideNotificationPermission {
  notRequested,
  granted,
  denied,
  permanentlyDenied;

  static StrideNotificationPermission fromJson(String s) => switch (s) {
    'not_requested' => StrideNotificationPermission.notRequested,
    'granted' => StrideNotificationPermission.granted,
    'denied' => StrideNotificationPermission.denied,
    'permanently_denied' => StrideNotificationPermission.permanentlyDenied,
    _ => StrideNotificationPermission.notRequested,
  };

  String toJson() => switch (this) {
    StrideNotificationPermission.notRequested => 'not_requested',
    StrideNotificationPermission.granted => 'granted',
    StrideNotificationPermission.denied => 'denied',
    StrideNotificationPermission.permanentlyDenied => 'permanently_denied',
  };
}

/// How coaching prompts are delivered to the user.
enum StrideCoachingDeliveryMode {
  none,
  visual,
  voice,
  both;

  static StrideCoachingDeliveryMode fromJson(String s) => switch (s) {
    'none' => StrideCoachingDeliveryMode.none,
    'visual' => StrideCoachingDeliveryMode.visual,
    'voice' => StrideCoachingDeliveryMode.voice,
    'both' => StrideCoachingDeliveryMode.both,
    _ => StrideCoachingDeliveryMode.none,
  };

  String toJson() => switch (this) {
    StrideCoachingDeliveryMode.none => 'none',
    StrideCoachingDeliveryMode.visual => 'visual',
    StrideCoachingDeliveryMode.voice => 'voice',
    StrideCoachingDeliveryMode.both => 'both',
  };
}

/// The kind of voice coaching announcement being evaluated.
enum StrideCoachingAnnouncementKind {
  distance,
  time,
  pace,
  split,
  goalProgress,
  encouragement;

  static StrideCoachingAnnouncementKind fromJson(String s) => switch (s) {
    'distance' => StrideCoachingAnnouncementKind.distance,
    'time' => StrideCoachingAnnouncementKind.time,
    'pace' => StrideCoachingAnnouncementKind.pace,
    'split' => StrideCoachingAnnouncementKind.split,
    'goal_progress' => StrideCoachingAnnouncementKind.goalProgress,
    'encouragement' => StrideCoachingAnnouncementKind.encouragement,
    _ => StrideCoachingAnnouncementKind.distance,
  };

  String toJson() => switch (this) {
    StrideCoachingAnnouncementKind.distance => 'distance',
    StrideCoachingAnnouncementKind.time => 'time',
    StrideCoachingAnnouncementKind.pace => 'pace',
    StrideCoachingAnnouncementKind.split => 'split',
    StrideCoachingAnnouncementKind.goalProgress => 'goal_progress',
    StrideCoachingAnnouncementKind.encouragement => 'encouragement',
  };
}

/// The user's per-category notification preferences. All categories
/// default to `false` — the user must opt in.
class StrideNotificationPreferences {
  final bool remindersEnabled;
  final bool achievementsEnabled;
  final bool coachingEnabled;
  final bool systemEnabled;
  final bool recoveryEnabled;
  final bool criticalBypassesQuietHours;

  StrideNotificationPreferences({
    this.remindersEnabled = false,
    this.achievementsEnabled = false,
    this.coachingEnabled = false,
    this.systemEnabled = false,
    this.recoveryEnabled = false,
    this.criticalBypassesQuietHours = true,
  });

  StrideNotificationPreferences.fromJson(Map<String, dynamic> j)
      : remindersEnabled = j['reminders_enabled'] as bool,
        achievementsEnabled = j['achievements_enabled'] as bool,
        coachingEnabled = j['coaching_enabled'] as bool,
        systemEnabled = j['system_enabled'] as bool,
        recoveryEnabled = j['recovery_enabled'] as bool,
        criticalBypassesQuietHours = j['critical_bypasses_quiet_hours'] as bool;

  Map<String, dynamic> toJson() => {
    'reminders_enabled': remindersEnabled,
    'achievements_enabled': achievementsEnabled,
    'coaching_enabled': coachingEnabled,
    'system_enabled': systemEnabled,
    'recovery_enabled': recoveryEnabled,
    'critical_bypasses_quiet_hours': criticalBypassesQuietHours,
  };
}

/// The user's time-zone context for evaluating quiet hours and
/// scheduling notifications in local time.
class StrideTimezoneContext {
  final int utcOffsetSeconds;
  final bool isDst;

  StrideTimezoneContext({
    this.utcOffsetSeconds = 0,
    this.isDst = false,
  });

  StrideTimezoneContext.fromJson(Map<String, dynamic> j)
      : utcOffsetSeconds = j['utc_offset_seconds'] as int,
        isDst = j['is_dst'] as bool;

  Map<String, dynamic> toJson() => {
    'utc_offset_seconds': utcOffsetSeconds,
    'is_dst': isDst,
  };
}

/// Configuration for quiet hours — a time window during which
/// non-critical notifications are suppressed.
class StrideQuietHoursConfig {
  final bool enabled;
  final int startMinute;
  final int endMinute;

  StrideQuietHoursConfig({
    this.enabled = false,
    this.startMinute = 1320, // 22:00
    this.endMinute = 420,    // 07:00
  });

  StrideQuietHoursConfig.fromJson(Map<String, dynamic> j)
      : enabled = j['enabled'] as bool,
        startMinute = j['start_minute'] as int,
        endMinute = j['end_minute'] as int;

  Map<String, dynamic> toJson() => {
    'enabled': enabled,
    'start_minute': startMinute,
    'end_minute': endMinute,
  };
}

/// The result of a quiet-hours evaluation for a specific notification.
class StrideQuietHoursDecision {
  final bool isQuiet;
  final bool shouldDeliverNow;
  final int? rescheduleToMinute;
  final String reason;

  StrideQuietHoursDecision.fromJson(Map<String, dynamic> j)
      : isQuiet = j['is_quiet'] as bool,
        shouldDeliverNow = j['should_deliver_now'] as bool,
        rescheduleToMinute = j['reschedule_to_minute'] as int?,
        reason = j['reason'] as String;
}

/// The content of a notification to display.
class StrideNotificationContent {
  final StrideNotificationType notifType;
  final StrideNotificationCategory category;
  final StrideNotificationPriority priority;
  final String title;
  final String body;
  final String? actionLabel;
  final String? actionData;

  StrideNotificationContent.fromJson(Map<String, dynamic> j)
      : notifType =
            StrideNotificationType.fromJson(j['notif_type'] as String),
        category =
            StrideNotificationCategory.fromJson(j['category'] as String),
        priority =
            StrideNotificationPriority.fromJson(j['priority'] as String),
        title = j['title'] as String,
        body = j['body'] as String,
        actionLabel = j['action_label'] as String?,
        actionData = j['action_data'] as String?;
}

/// Optional context parameters for building notification content.
class StrideNotificationContextParams {
  final String? workoutTime;
  final String? workoutType;
  final String? goalDescription;
  final int? streakDays;
  final String? deviceName;
  final String? badgeName;
  final String? coachingMessage;
  final String? actionData;

  StrideNotificationContextParams({
    this.workoutTime,
    this.workoutType,
    this.goalDescription,
    this.streakDays,
    this.deviceName,
    this.badgeName,
    this.coachingMessage,
    this.actionData,
  });

  StrideNotificationContextParams.fromJson(Map<String, dynamic> j)
      : workoutTime = j['workout_time'] as String?,
        workoutType = j['workout_type'] as String?,
        goalDescription = j['goal_description'] as String?,
        streakDays = j['streak_days'] as int?,
        deviceName = j['device_name'] as String?,
        badgeName = j['badge_name'] as String?,
        coachingMessage = j['coaching_message'] as String?,
        actionData = j['action_data'] as String?;

  Map<String, dynamic> toJson() => {
    'workout_time': workoutTime,
    'workout_type': workoutType,
    'goal_description': goalDescription,
    'streak_days': streakDays,
    'device_name': deviceName,
    'badge_name': badgeName,
    'coaching_message': coachingMessage,
    'action_data': actionData,
  };
}

/// When a notification should be delivered.
///
/// Serialized by Rust/serde as a tagged union:
/// - `Immediate` → `"immediate"`
/// - `Scheduled { at_utc_ms }` → `{"scheduled":{"at_utc_ms":...}}`
/// - `Daily { at_local_minute }` → `{"daily":{"at_local_minute":...}}`
class StrideNotificationSchedule {
  final StrideScheduleKind kind;
  final int? atUtcMs;
  final int? atLocalMinute;

  const StrideNotificationSchedule._({
    required this.kind,
    this.atUtcMs,
    this.atLocalMinute,
  });

  const StrideNotificationSchedule.immediate()
      : kind = StrideScheduleKind.immediate,
        atUtcMs = null,
        atLocalMinute = null;

  const StrideNotificationSchedule.scheduled(int utcMs)
      : kind = StrideScheduleKind.scheduled,
        atUtcMs = utcMs,
        atLocalMinute = null;

  const StrideNotificationSchedule.daily(int localMinute)
      : kind = StrideScheduleKind.daily,
        atUtcMs = null,
        atLocalMinute = localMinute;

  StrideNotificationSchedule.fromJson(dynamic j)
      : kind = _kindFromJson(j),
        atUtcMs = _atUtcMsFromJson(j),
        atLocalMinute = _atLocalMinuteFromJson(j);

  static StrideScheduleKind _kindFromJson(dynamic j) {
    if (j is String) {
      return switch (j) {
        'immediate' => StrideScheduleKind.immediate,
        _ => StrideScheduleKind.immediate,
      };
    }
    if (j is Map<String, dynamic>) {
      if (j.containsKey('scheduled')) return StrideScheduleKind.scheduled;
      if (j.containsKey('daily')) return StrideScheduleKind.daily;
    }
    return StrideScheduleKind.immediate;
  }

  static int? _atUtcMsFromJson(dynamic j) {
    if (j is Map<String, dynamic> && j.containsKey('scheduled')) {
      final inner = j['scheduled'] as Map<String, dynamic>;
      return inner['at_utc_ms'] as int?;
    }
    return null;
  }

  static int? _atLocalMinuteFromJson(dynamic j) {
    if (j is Map<String, dynamic> && j.containsKey('daily')) {
      final inner = j['daily'] as Map<String, dynamic>;
      return inner['at_local_minute'] as int?;
    }
    return null;
  }
}

/// The kind of notification schedule.
enum StrideScheduleKind {
  immediate,
  scheduled,
  daily;
}

/// A fully specified notification request: what to show, when, and
/// with what content.
class StrideNotificationRequest {
  final StrideNotificationType notifType;
  final StrideNotificationSchedule schedule;
  final StrideNotificationContextParams context;

  StrideNotificationRequest({
    required this.notifType,
    required this.schedule,
    required this.context,
  });

  StrideNotificationRequest.fromJson(Map<String, dynamic> j)
      : notifType =
            StrideNotificationType.fromJson(j['notif_type'] as String),
        schedule = StrideNotificationSchedule.fromJson(j['schedule']),
        context = StrideNotificationContextParams.fromJson(
            j['context'] as Map<String, dynamic>);

  Map<String, dynamic> toJson() => {
    'notif_type': notifType.toJson(),
    'schedule': _scheduleToJson(),
    'context': context.toJson(),
  };

  dynamic _scheduleToJson() {
    switch (schedule.kind) {
      case StrideScheduleKind.immediate:
        return 'immediate';
      case StrideScheduleKind.scheduled:
        return {'scheduled': {'at_utc_ms': schedule.atUtcMs}};
      case StrideScheduleKind.daily:
        return {'daily': {'at_local_minute': schedule.atLocalMinute}};
    }
  }
}

/// Configuration for voice coaching announcements.
class StrideVoiceCoachingConfig {
  final StrideCoachingDeliveryMode mode;
  final bool announceDistance;
  final bool announceTime;
  final bool announcePace;
  final bool announceSplits;
  final bool announceGoalProgress;
  final int distanceIntervalM;
  final int timeIntervalS;

  StrideVoiceCoachingConfig({
    this.mode = StrideCoachingDeliveryMode.none,
    this.announceDistance = true,
    this.announceTime = true,
    this.announcePace = false,
    this.announceSplits = true,
    this.announceGoalProgress = true,
    this.distanceIntervalM = 1000,
    this.timeIntervalS = 300,
  });

  StrideVoiceCoachingConfig.fromJson(Map<String, dynamic> j)
      : mode = StrideCoachingDeliveryMode.fromJson(j['mode'] as String),
        announceDistance = j['announce_distance'] as bool,
        announceTime = j['announce_time'] as bool,
        announcePace = j['announce_pace'] as bool,
        announceSplits = j['announce_splits'] as bool,
        announceGoalProgress = j['announce_goal_progress'] as bool,
        distanceIntervalM = j['distance_interval_m'] as int,
        timeIntervalS = j['time_interval_s'] as int;

  Map<String, dynamic> toJson() => {
    'mode': mode.toJson(),
    'announce_distance': announceDistance,
    'announce_time': announceTime,
    'announce_pace': announcePace,
    'announce_splits': announceSplits,
    'announce_goal_progress': announceGoalProgress,
    'distance_interval_m': distanceIntervalM,
    'time_interval_s': timeIntervalS,
  };
}

/// The overall decision for a notification request: should it be
/// delivered, and if so, how and when?
class StrideNotificationDecision {
  final StrideNotificationType notifType;
  final bool shouldDeliver;
  final bool deliverNow;
  final StrideNotificationContent? content;
  final int? rescheduleToMinute;
  final String reason;
  final StrideNotificationPriority priority;

  StrideNotificationDecision.fromJson(Map<String, dynamic> j)
      : notifType =
            StrideNotificationType.fromJson(j['notif_type'] as String),
        shouldDeliver = j['should_deliver'] as bool,
        deliverNow = j['deliver_now'] as bool,
        content = j['content'] != null
            ? StrideNotificationContent.fromJson(
                j['content'] as Map<String, dynamic>)
            : null,
        rescheduleToMinute = j['reschedule_to_minute'] as int?,
        reason = j['reason'] as String,
        priority =
            StrideNotificationPriority.fromJson(j['priority'] as String);
}

/// The full context for making a notification decision.
class StrideNotificationDecisionContext {
  final StrideNotificationType notifType;
  final StrideNotificationPreferences preferences;
  final StrideNotificationPermission permission;
  final StrideQuietHoursConfig quietHours;
  final int nowUtcMs;
  final StrideTimezoneContext timezone;
  final StrideNotificationContextParams contextParams;

  StrideNotificationDecisionContext({
    required this.notifType,
    required this.preferences,
    required this.permission,
    required this.quietHours,
    required this.nowUtcMs,
    required this.timezone,
    required this.contextParams,
  });

  Map<String, dynamic> toJson() => {
    'notif_type': notifType.toJson(),
    'preferences': preferences.toJson(),
    'permission': permission.toJson(),
    'quiet_hours': quietHours.toJson(),
    'now_utc_ms': nowUtcMs,
    'timezone': timezone.toJson(),
    'context_params': contextParams.toJson(),
  };
}

/// A full status snapshot of the notification system for the UI or
/// diagnostics screen.
class StrideNotificationStatus {
  final bool permissionGranted;
  final StrideNotificationPermission permission;
  final int enabledCategoryCount;
  final bool quietHoursEnabled;
  final String quietHoursWindow;
  final bool voiceCoachingEnabled;
  final StrideCoachingDeliveryMode coachingMode;
  final String timezoneLabel;

  StrideNotificationStatus.fromJson(Map<String, dynamic> j)
      : permissionGranted = j['permission_granted'] as bool,
        permission =
            StrideNotificationPermission.fromJson(j['permission'] as String),
        enabledCategoryCount = j['enabled_category_count'] as int,
        quietHoursEnabled = j['quiet_hours_enabled'] as bool,
        quietHoursWindow = j['quiet_hours_window'] as String,
        voiceCoachingEnabled = j['voice_coaching_enabled'] as bool,
        coachingMode =
            StrideCoachingDeliveryMode.fromJson(j['coaching_mode'] as String),
        timezoneLabel = j['timezone_label'] as String;
}

// ---------------------------------------------------------------------------
// §14 — Error / Recovery States
// ---------------------------------------------------------------------------

/// The subsystem or layer that an error originated from.
enum StrideErrorCategory {
  gps,
  sensors,
  sync,
  network,
  storage,
  bluetooth,
  permissions,
  engine,
  ffi,
  music,
  coaching,
  notifications,
  background,
  unknown;

  static StrideErrorCategory fromJson(String s) => switch (s) {
    'gps' => StrideErrorCategory.gps,
    'sensors' => StrideErrorCategory.sensors,
    'sync' => StrideErrorCategory.sync,
    'network' => StrideErrorCategory.network,
    'storage' => StrideErrorCategory.storage,
    'bluetooth' => StrideErrorCategory.bluetooth,
    'permissions' => StrideErrorCategory.permissions,
    'engine' => StrideErrorCategory.engine,
    'ffi' => StrideErrorCategory.ffi,
    'music' => StrideErrorCategory.music,
    'coaching' => StrideErrorCategory.coaching,
    'notifications' => StrideErrorCategory.notifications,
    'background' => StrideErrorCategory.background,
    'unknown' => StrideErrorCategory.unknown,
    _ => StrideErrorCategory.unknown,
  };

  String toJson() => switch (this) {
    StrideErrorCategory.gps => 'gps',
    StrideErrorCategory.sensors => 'sensors',
    StrideErrorCategory.sync => 'sync',
    StrideErrorCategory.network => 'network',
    StrideErrorCategory.storage => 'storage',
    StrideErrorCategory.bluetooth => 'bluetooth',
    StrideErrorCategory.permissions => 'permissions',
    StrideErrorCategory.engine => 'engine',
    StrideErrorCategory.ffi => 'ffi',
    StrideErrorCategory.music => 'music',
    StrideErrorCategory.coaching => 'coaching',
    StrideErrorCategory.notifications => 'notifications',
    StrideErrorCategory.background => 'background',
    StrideErrorCategory.unknown => 'unknown',
  };
}

/// How serious an error is.
enum StrideErrorSeverity {
  info,
  low,
  medium,
  high,
  critical;

  static StrideErrorSeverity fromJson(String s) => switch (s) {
    'info' => StrideErrorSeverity.info,
    'low' => StrideErrorSeverity.low,
    'medium' => StrideErrorSeverity.medium,
    'high' => StrideErrorSeverity.high,
    'critical' => StrideErrorSeverity.critical,
    _ => StrideErrorSeverity.medium,
  };

  String toJson() => switch (this) {
    StrideErrorSeverity.info => 'info',
    StrideErrorSeverity.low => 'low',
    StrideErrorSeverity.medium => 'medium',
    StrideErrorSeverity.high => 'high',
    StrideErrorSeverity.critical => 'critical',
  };
}

/// The lifecycle state of an error.
enum StrideErrorState {
  detected,
  reported,
  recovering,
  resolved,
  abandoned,
  escalated;

  static StrideErrorState fromJson(String s) => switch (s) {
    'detected' => StrideErrorState.detected,
    'reported' => StrideErrorState.reported,
    'recovering' => StrideErrorState.recovering,
    'resolved' => StrideErrorState.resolved,
    'abandoned' => StrideErrorState.abandoned,
    'escalated' => StrideErrorState.escalated,
    _ => StrideErrorState.detected,
  };

  String toJson() => switch (this) {
    StrideErrorState.detected => 'detected',
    StrideErrorState.reported => 'reported',
    StrideErrorState.recovering => 'recovering',
    StrideErrorState.resolved => 'resolved',
    StrideErrorState.abandoned => 'abandoned',
    StrideErrorState.escalated => 'escalated',
  };
}

/// What to do in response to an error.
enum StrideRecoveryStrategy {
  retry,
  fallback,
  ignore,
  abort,
  escalate,
  manual;

  static StrideRecoveryStrategy fromJson(String s) => switch (s) {
    'retry' => StrideRecoveryStrategy.retry,
    'fallback' => StrideRecoveryStrategy.fallback,
    'ignore' => StrideRecoveryStrategy.ignore,
    'abort' => StrideRecoveryStrategy.abort,
    'escalate' => StrideRecoveryStrategy.escalate,
    'manual' => StrideRecoveryStrategy.manual,
    _ => StrideRecoveryStrategy.ignore,
  };

  String toJson() => switch (this) {
    StrideRecoveryStrategy.retry => 'retry',
    StrideRecoveryStrategy.fallback => 'fallback',
    StrideRecoveryStrategy.ignore => 'ignore',
    StrideRecoveryStrategy.abort => 'abort',
    StrideRecoveryStrategy.escalate => 'escalate',
    StrideRecoveryStrategy.manual => 'manual',
  };
}

/// Overall health label for the engine.
enum StrideEngineHealthLabel {
  healthy,
  operational,
  degraded,
  warning,
  critical;

  static StrideEngineHealthLabel fromJson(String s) => switch (s) {
    'healthy' => StrideEngineHealthLabel.healthy,
    'operational' => StrideEngineHealthLabel.operational,
    'degraded' => StrideEngineHealthLabel.degraded,
    'warning' => StrideEngineHealthLabel.warning,
    'critical' => StrideEngineHealthLabel.critical,
    _ => StrideEngineHealthLabel.healthy,
  };

  String toJson() => switch (this) {
    StrideEngineHealthLabel.healthy => 'healthy',
    StrideEngineHealthLabel.operational => 'operational',
    StrideEngineHealthLabel.degraded => 'degraded',
    StrideEngineHealthLabel.warning => 'warning',
    StrideEngineHealthLabel.critical => 'critical',
  };
}

/// Configuration for retry behavior with exponential backoff + jitter.
class StrideRetryPolicy {
  final int maxAttempts;
  final int baseDelayMs;
  final int maxDelayMs;
  final double backoffMultiplier;
  final double jitterFraction;

  StrideRetryPolicy({
    this.maxAttempts = 5,
    this.baseDelayMs = 1000,
    this.maxDelayMs = 600000,
    this.backoffMultiplier = 2.0,
    this.jitterFraction = 0.25,
  });

  /// Default policy.
  factory StrideRetryPolicy.defaultPolicy() => StrideRetryPolicy();

  /// No-retry policy (max_attempts = 0).
  factory StrideRetryPolicy.noRetry() => StrideRetryPolicy(
        maxAttempts: 0,
        baseDelayMs: 0,
        maxDelayMs: 0,
        backoffMultiplier: 1.0,
        jitterFraction: 0.0,
      );

  /// Aggressive retry policy (more attempts, shorter delays).
  factory StrideRetryPolicy.aggressive() => StrideRetryPolicy(
        maxAttempts: 10,
        baseDelayMs: 500,
        maxDelayMs: 60000,
        backoffMultiplier: 1.5,
        jitterFraction: 0.3,
      );

  /// Gentle retry policy (fewer attempts, longer delays).
  factory StrideRetryPolicy.gentle() => StrideRetryPolicy(
        maxAttempts: 3,
        baseDelayMs: 5000,
        maxDelayMs: 1800000,
        backoffMultiplier: 2.5,
        jitterFraction: 0.2,
      );

  Map<String, dynamic> toJson() => {
        'max_attempts': maxAttempts,
        'base_delay_ms': baseDelayMs,
        'max_delay_ms': maxDelayMs,
        'backoff_multiplier': backoffMultiplier,
        'jitter_fraction': jitterFraction,
      };
}

/// Full context for a single error occurrence.
class StrideErrorContext {
  final StrideErrorCategory category;
  final StrideErrorSeverity severity;
  final String code;
  final String message;
  final String subsystem;
  final int timestampMs;
  final int retryCount;
  final List<List<String>> metadata;

  StrideErrorContext({
    required this.category,
    required this.severity,
    required this.code,
    required this.message,
    required this.subsystem,
    this.timestampMs = 0,
    this.retryCount = 0,
    this.metadata = const [],
  });

  Map<String, dynamic> toJson() => {
        'category': category.toJson(),
        'severity': severity.toJson(),
        'code': code,
        'message': message,
        'subsystem': subsystem,
        'timestamp_ms': timestampMs,
        'retry_count': retryCount,
        'metadata': metadata,
      };

  StrideErrorContext.fromJson(Map<String, dynamic> j)
      : category = StrideErrorCategory.fromJson(j['category'] as String),
        severity = StrideErrorSeverity.fromJson(j['severity'] as String),
        code = j['code'] as String,
        message = j['message'] as String,
        subsystem = j['subsystem'] as String,
        timestampMs = j['timestamp_ms'] as int,
        retryCount = j['retry_count'] as int,
        metadata = (j['metadata'] as List)
            .map((e) => (e as List).map((v) => v as String).toList())
            .toList();
}

/// The action to take in response to an error (result of decide_recovery).
class StrideRecoveryAction {
  final StrideRecoveryStrategy strategy;
  final int delayMs;
  final String? userMessage;
  final bool shouldReport;
  final bool shouldPersist;
  final StrideErrorState nextState;
  final String reason;

  StrideRecoveryAction({
    required this.strategy,
    required this.delayMs,
    this.userMessage,
    required this.shouldReport,
    required this.shouldPersist,
    required this.nextState,
    required this.reason,
  });

  StrideRecoveryAction.fromJson(Map<String, dynamic> j)
      : strategy = StrideRecoveryStrategy.fromJson(j['strategy'] as String),
        delayMs = j['delay_ms'] as int,
        userMessage = j['user_message'] as String?,
        shouldReport = j['should_report'] as bool,
        shouldPersist = j['should_persist'] as bool,
        nextState = StrideErrorState.fromJson(j['next_state'] as String),
        reason = j['reason'] as String;
}

/// A validated state transition for an error.
class StrideErrorStateTransition {
  final StrideErrorState from;
  final StrideErrorState to;
  final bool isValid;
  final String message;

  StrideErrorStateTransition({
    required this.from,
    required this.to,
    required this.isValid,
    required this.message,
  });

  StrideErrorStateTransition.fromJson(Map<String, dynamic> j)
      : from = StrideErrorState.fromJson(j['from'] as String),
        to = StrideErrorState.fromJson(j['to'] as String),
        isValid = j['is_valid'] as bool,
        message = j['message'] as String;
}

/// An entry in the error registry.
class StrideErrorRegistryEntry {
  final StrideErrorContext error;
  final StrideErrorState state;
  final StrideRecoveryAction? recoveryAction;

  StrideErrorRegistryEntry({
    required this.error,
    required this.state,
    this.recoveryAction,
  });

  StrideErrorRegistryEntry.fromJson(Map<String, dynamic> j)
      : error = StrideErrorContext.fromJson(j['error'] as Map<String, dynamic>),
        state = StrideErrorState.fromJson(j['state'] as String),
        recoveryAction = j['recovery_action'] != null
            ? StrideRecoveryAction.fromJson(
                j['recovery_action'] as Map<String, dynamic>)
            : null;
}

/// An in-memory registry of recent errors for diagnostics.
class StrideErrorRegistry {
  final List<StrideErrorRegistryEntry> entries;
  final int maxEntries;

  StrideErrorRegistry({
    this.entries = const [],
    this.maxEntries = 100,
  });

  StrideErrorRegistry.fromJson(Map<String, dynamic> j)
      : entries = (j['entries'] as List)
            .map((e) => StrideErrorRegistryEntry.fromJson(
                e as Map<String, dynamic>))
            .toList(),
        maxEntries = j['max_entries'] as int;

  Map<String, dynamic> toJson() => {
        'entries': entries.map((e) {
          final m = <String, dynamic>{
            'error': e.error.toJson(),
            'state': e.state.toJson(),
          };
          if (e.recoveryAction != null) {
            m['recovery_action'] = {
              'strategy': e.recoveryAction!.strategy.toJson(),
              'delay_ms': e.recoveryAction!.delayMs,
              'user_message': e.recoveryAction!.userMessage,
              'should_report': e.recoveryAction!.shouldReport,
              'should_persist': e.recoveryAction!.shouldPersist,
              'next_state': e.recoveryAction!.nextState.toJson(),
              'reason': e.recoveryAction!.reason,
            };
          }
          return m;
        }).toList(),
        'max_entries': maxEntries,
      };
}

/// The overall health status of the engine for the UI / diagnostics.
class StrideEngineHealthStatus {
  final StrideEngineHealthLabel status;
  final int totalErrors;
  final int activeErrors;
  final int resolvedErrors;
  final int abandonedErrors;
  final int criticalErrors;
  final bool isDegraded;
  final String summary;
  final List<List<dynamic>> categoryCounts;

  StrideEngineHealthStatus({
    required this.status,
    required this.totalErrors,
    required this.activeErrors,
    required this.resolvedErrors,
    required this.abandonedErrors,
    required this.criticalErrors,
    required this.isDegraded,
    required this.summary,
    required this.categoryCounts,
  });

  StrideEngineHealthStatus.fromJson(Map<String, dynamic> j)
      : status = StrideEngineHealthLabel.fromJson(j['status'] as String),
        totalErrors = j['total_errors'] as int,
        activeErrors = j['active_errors'] as int,
        resolvedErrors = j['resolved_errors'] as int,
        abandonedErrors = j['abandoned_errors'] as int,
        criticalErrors = j['critical_errors'] as int,
        isDegraded = j['is_degraded'] as bool,
        summary = j['summary'] as String,
        categoryCounts = (j['category_counts'] as List)
            .map((e) {
              final arr = e as List;
              return <dynamic>[
                StrideErrorCategory.fromJson(arr[0] as String),
                arr[1] as int,
              ];
            })
            .toList();
}
