import 'dart:async';

import 'package:flutter/foundation.dart' show kIsWeb;

import '../database/local_database.dart';
import '../models/walking_session.dart';
import '../native/stride_engine_client.dart';
import '../native/stride_models.dart';
import 'sync_service.dart';
import 'workout_recorder.dart';

/// Outcome of an app-startup crash-recovery check, for the UI layer to
/// react to (e.g. show a "resume your walk?" dialog).
enum RecoveryOutcome {
  /// No leftover in-progress workout was found — nothing to do.
  nothingToRecover,

  /// A workout was found and successfully attached to [WorkoutRecorder]
  /// in the paused state, ready for the caller to resume/finish/discard
  /// based on [RecoveryService.lastDecision].
  recovered,

  /// A leftover workout was found but could not be safely recovered
  /// (e.g. corrupted checkpoint) — it was finalized/discarded
  /// automatically as a safe fallback.
  recoveredWithFallback,
}

/// Runs once at app startup (before `runApp`) to detect and recover an
/// in-progress workout left behind by a crash, force-kill, or OS restart
/// of the app process (spec section 23).
///
/// Flow:
/// 1. Look for a leftover `active_workout` row in SQLite.
/// 2. Load its most recent checkpoint from `workout_checkpoints`.
/// 3. Ask the Rust engine (`stride_evaluate_recovery`) what to do with it.
/// 4. Either rebuild a live, paused native controller
///    (`stride_restore_session`, attached via
///    [WorkoutRecorder.restoreFromCheckpoint]) so the user can resume,
///    or safely finalize/discard the leftover workout automatically.
class RecoveryService {
  RecoveryService({
    required LocalDatabase localDb,
    required WorkoutRecorder workoutRecorder,
    required SyncService syncService,
  })  : _localDb = localDb,
        _workoutRecorder = workoutRecorder,
        _syncService = syncService;

  final LocalDatabase _localDb;
  final WorkoutRecorder _workoutRecorder;
  final SyncService _syncService;

  /// The recovery decision the engine returned for the most recent
  /// [run], if any — the UI can inspect this to decide what dialog/
  /// options to present when [run] returns [RecoveryOutcome.recovered].
  StrideRecoveryDecision? lastDecision;

  /// The recovered session (in `paused` state), if any, matching
  /// [WorkoutRecorder.activeSession] after a [RecoveryOutcome.recovered]
  /// result.
  WalkingSession? recoveredSession;

  /// Performs the startup recovery check. Safe to call unconditionally
  /// on every launch — it's a fast no-op when there is nothing to
  /// recover.
  Future<RecoveryOutcome> run({required String currentUserId}) async {
    if (kIsWeb) return RecoveryOutcome.nothingToRecover;

    final activeRow = await _localDb.getActiveWorkout();
    if (activeRow == null) return RecoveryOutcome.nothingToRecover;

    final workoutId = activeRow['id'] as String;
    final checkpointJson = await _localDb.getCheckpoint(workoutId);

    // No checkpoint was ever written for this workout (e.g. killed
    // before the first GPS fix) — nothing safe to resume from; clean up
    // the orphaned row and stop.
    if (checkpointJson == null) {
      await _discardOrphanedRow(activeRow);
      return RecoveryOutcome.recoveredWithFallback;
    }

    final now = DateTime.now();
    late final StrideRecoveryDecision decision;
    try {
      decision = StrideEngineClient.evaluateRecovery(
        checkpointJson: checkpointJson,
        now: now,
      );
    } catch (_) {
      // Corrupted checkpoint JSON or engine error — fail safe by
      // discarding rather than risking a stuck app.
      await _discardOrphanedRow(activeRow);
      return RecoveryOutcome.recoveredWithFallback;
    }

    lastDecision = decision;

    switch (decision.recommendedOption) {
      case StrideRecoveryOption.resumeWorkout:
        return _attemptResume(activeRow, checkpointJson, currentUserId);
      case StrideRecoveryOption.finishAndSave:
        return _finishOrphanedRow(activeRow, checkpointJson, currentUserId);
      case StrideRecoveryOption.discardWorkout:
        await _discardOrphanedRow(activeRow);
        return RecoveryOutcome.recoveredWithFallback;
    }
  }

  Future<RecoveryOutcome> _attemptResume(
    Map<String, dynamic> activeRow,
    Map<String, dynamic> checkpointJson,
    String currentUserId,
  ) async {
    final workoutId = activeRow['id'] as String;
    final userId = activeRow['user_id'] as String? ?? currentUserId;
    final activityType = _sessionTypeFromDb(activeRow['activity_type']);

    final sessionSeed = WalkingSession(
      id: workoutId,
      userId: userId,
      startTime: DateTime.parse(activeRow['started_at'] as String),
      distanceMeters: (activeRow['distance_meters'] as num?)?.toDouble() ?? 0,
      duration:
          Duration(seconds: activeRow['duration_seconds'] as int? ?? 0),
      stepCount: activeRow['step_count'] as int? ?? 0,
      caloriesBurned:
          (activeRow['calories_estimated'] as num?)?.toDouble() ?? 0,
      type: activityType,
      status: SessionStatus.paused,
    );

    try {
      final restored = _workoutRecorder.restoreFromCheckpoint(
        checkpointJson: checkpointJson,
        sessionSeed: sessionSeed,
        userId: userId,
        activityType: activityType,
      );
      if (!restored) {
        await _discardOrphanedRow(activeRow);
        return RecoveryOutcome.recoveredWithFallback;
      }
      recoveredSession = _workoutRecorder.activeSession;
      return RecoveryOutcome.recovered;
    } catch (_) {
      await _discardOrphanedRow(activeRow);
      return RecoveryOutcome.recoveredWithFallback;
    }
  }

  /// Finalizes a leftover workout using only the last-known checkpoint
  /// totals (no live engine involved) — used when the engine recommends
  /// "finish and save" rather than resuming live tracking (e.g. a stale
  /// checkpoint).
  Future<RecoveryOutcome> _finishOrphanedRow(
    Map<String, dynamic> activeRow,
    Map<String, dynamic> checkpointJson,
    String currentUserId,
  ) async {
    final workoutId = activeRow['id'] as String;
    final userId = activeRow['user_id'] as String? ?? currentUserId;
    final activityType = _sessionTypeFromDb(activeRow['activity_type']);

    final distanceMeters =
        (checkpointJson['total_distance_meters'] as num?)?.toDouble() ??
            (activeRow['distance_meters'] as num?)?.toDouble() ??
            0.0;
    final activeMs = (checkpointJson['active_ms'] as num?)?.toInt() ??
        ((activeRow['duration_seconds'] as int? ?? 0) * 1000);

    final finalized = WalkingSession(
      id: workoutId,
      userId: userId,
      startTime: DateTime.parse(activeRow['started_at'] as String),
      endTime: DateTime.now(),
      distanceMeters: distanceMeters,
      duration: Duration(milliseconds: activeMs),
      stepCount: activeRow['step_count'] as int? ?? 0,
      caloriesBurned:
          (activeRow['calories_estimated'] as num?)?.toDouble() ?? 0,
      type: activityType,
      status: SessionStatus.completed,
    );

    await _localDb.completeWorkout(workoutId);
    final routePoints = await _localDb.getRoutePoints(workoutId);
    await _localDb.queueForUpload(finalized, routePoints);
    await _localDb.cacheSession(finalized);
    await _localDb.deleteRoutePoints(workoutId);
    await _localDb.deleteStepSamples(workoutId);
    await _localDb.deleteCheckpoint(workoutId);
    await _localDb.deleteActiveWorkout(workoutId);

    // Give the newly-queued upload an immediate chance to sync.
    unawaited(_syncService.syncPending());

    return RecoveryOutcome.recoveredWithFallback;
  }

  Future<void> _discardOrphanedRow(Map<String, dynamic> activeRow) async {
    final workoutId = activeRow['id'] as String;
    await _localDb.deleteRoutePoints(workoutId);
    await _localDb.deleteStepSamples(workoutId);
    await _localDb.deleteCheckpoint(workoutId);
    await _localDb.deleteActiveWorkout(workoutId);
  }

  SessionType _sessionTypeFromDb(Object? raw) {
    final s = raw as String? ?? 'walk';
    return SessionType.values.firstWhere(
      (t) => t.name == s,
      orElse: () => SessionType.walk,
    );
  }
}
