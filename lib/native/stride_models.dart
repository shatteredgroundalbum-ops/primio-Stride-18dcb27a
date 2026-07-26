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
