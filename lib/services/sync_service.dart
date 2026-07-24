import 'dart:async';
import 'dart:convert';

import '../database/local_database.dart';
import '../models/sync_status.dart';
import '../models/walking_session.dart';
import '../repositories/cloud/cloud_route_repository.dart';
import '../repositories/cloud/cloud_summary_repository.dart';
import '../repositories/cloud/cloud_workout_repository.dart';
import 'preferences_service.dart';

/// Manages the local → cloud synchronization pipeline:
///
/// 1. Detect unsynced workouts in SQLite (pending_uploads table).
/// 2. Compress route points and upload to Cloud Storage.
/// 3. Write the finalized workout summary to Firestore.
/// 4. Update or create the daily summary document.
/// 5. Mark the local record as synced and clean up.
///
/// Runs automatically on a periodic timer and can be triggered manually.
class SyncService {
  final LocalDatabase _localDb;
  final CloudWorkoutRepository _workoutRepo;
  final CloudRouteRepository _routeRepo;
  final CloudSummaryRepository _summaryRepo;
  final PreferencesService _prefs;

  Timer? _periodicSync;
  bool _isSyncing = false;

  static const _syncInterval = Duration(minutes: 5);
  static const _maxRetries = 5;

  SyncService({
    required LocalDatabase localDb,
    required CloudWorkoutRepository workoutRepo,
    required CloudRouteRepository routeRepo,
    required CloudSummaryRepository summaryRepo,
    required PreferencesService prefs,
  })  : _localDb = localDb,
        _workoutRepo = workoutRepo,
        _routeRepo = routeRepo,
        _summaryRepo = summaryRepo,
        _prefs = prefs;

  /// Starts the periodic background sync.
  void startPeriodicSync() {
    _periodicSync?.cancel();
    _periodicSync = Timer.periodic(_syncInterval, (_) => syncPending());
  }

  /// Stops the periodic sync (e.g. on logout).
  void stopPeriodicSync() {
    _periodicSync?.cancel();
    _periodicSync = null;
  }

  /// Manually trigger a sync of all pending uploads.
  /// Returns the number of successfully synced workouts.
  Future<int> syncPending() async {
    if (_isSyncing) return 0;
    _isSyncing = true;

    int syncedCount = 0;

    try {
      final pendingRows = await _localDb.getPendingUploads();

      for (final row in pendingRows) {
        final workoutId = row['id'] as String;
        final retryCount = row['retry_count'] as int? ?? 0;

        // Skip workouts that have exceeded max retries
        if (retryCount >= _maxRetries) continue;

        try {
          // Mark as syncing
          await _localDb.updateUploadState(workoutId, SyncState.syncing);

          // Decode the stored data
          final sessionMap = jsonDecode(row['workout_json'] as String)
              as Map<String, dynamic>;
          final pointsList =
              (jsonDecode(row['route_points_json'] as String) as List)
                  .map((p) =>
                      RoutePoint.fromMap(p as Map<String, dynamic>))
                  .toList();

          final session = WalkingSession.fromMap(sessionMap);
          final userId = row['user_id'] as String;

          // Step 1: Upload route file to Cloud Storage
          String? routeFilePath;
          if (pointsList.isNotEmpty) {
            routeFilePath = await _routeRepo.uploadRouteFile(
                userId, workoutId, pointsList);
          }

          // Step 2: Write workout summary to Firestore
          final syncedSession = session.copyWith(
            routeFilePath: routeFilePath,
            syncMetadata: SyncMetadata(
              state: SyncState.synced,
              lastAttempt: DateTime.now(),
            ),
          );
          await _workoutRepo.saveWorkout(syncedSession);

          // Step 3: Update/create daily summary
          await _updateDailySummary(userId, syncedSession);

          // Step 4: Mark as synced and remove from queue
          await _localDb.updateUploadState(workoutId, SyncState.synced);
          await _localDb.removeFromUploadQueue(workoutId);

          syncedCount++;
        } catch (e) {
          // Mark failed — will retry on next sync cycle
          await _localDb.updateUploadState(
            workoutId,
            SyncState.failed,
            error: e.toString(),
          );
        }
      }

      if (syncedCount > 0) {
        await _prefs.setLastSyncTime(DateTime.now());
      }
    } finally {
      _isSyncing = false;
    }

    return syncedCount;
  }

  /// Updates the daily summary for the day of the given workout.
  Future<void> _updateDailySummary(
      String userId, WalkingSession session) async {
    final dateId =
        '${session.startTime.year}-${session.startTime.month.toString().padLeft(2, '0')}-${session.startTime.day.toString().padLeft(2, '0')}';

    final existing = await _summaryRepo.getSummary(userId, dateId);

    final updated = DailySummary(
      dateId: dateId,
      userId: userId,
      totalSteps: (existing?.totalSteps ?? 0) + session.stepCount,
      totalDistanceMeters:
          (existing?.totalDistanceMeters ?? 0) + session.distanceMeters,
      totalActiveMinutes:
          (existing?.totalActiveMinutes ?? 0) + session.duration.inMinutes,
      totalCaloriesBurned:
          (existing?.totalCaloriesBurned ?? 0) + session.caloriesBurned,
      workoutCount: (existing?.workoutCount ?? 0) + 1,
      averagePaceMinPerKm: session.averagePaceMinPerKm,
      averageHeartRate: session.averageHeartRate?.toDouble(),
    );

    await _summaryRepo.saveSummary(updated);
  }

  /// Disposes the sync timer.
  void dispose() {
    _periodicSync?.cancel();
  }
}
