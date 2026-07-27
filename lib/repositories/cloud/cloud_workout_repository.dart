import '../../database/local_database.dart';
import '../../models/walking_session.dart';

/// Abstract interface matching Firestore path:
/// `users/{userId}/workouts/{workoutId}`
///
/// Summary documents only — GPS route points are stored in Cloud Storage.
abstract class CloudWorkoutRepository {
  /// Writes a completed workout summary to Firestore.
  Future<void> saveWorkout(WalkingSession session);

  /// Reads a single workout by ID.
  Future<WalkingSession?> getWorkout(String userId, String workoutId);

  /// Lists all workouts for a user, most recent first.
  Future<List<WalkingSession>> getWorkouts(String userId);

  /// Lists workouts within a date range (for weekly/monthly views).
  Future<List<WalkingSession>> getWorkoutsInRange(
      String userId, DateTime start, DateTime end);

  /// Deletes a workout document (and its Cloud Storage route file).
  Future<void> deleteWorkout(String userId, String workoutId);
}

/// Real workout repository backed by the local SQLite database
/// (LocalDatabase). Completed sessions are cached in the
/// `cached_sessions` table by `WorkoutRecorder.stopWorkout()` and read
/// back here. When Firebase is connected, swap for
/// `FirestoreWorkoutRepository` — the abstract interface is identical.
class LocalCloudWorkoutRepository implements CloudWorkoutRepository {
  final LocalDatabase _localDb;

  LocalCloudWorkoutRepository({required LocalDatabase localDb})
      : _localDb = localDb;

  @override
  Future<void> saveWorkout(WalkingSession session) async {
    await _localDb.cacheSession(session);
  }

  @override
  Future<WalkingSession?> getWorkout(
      String userId, String workoutId) async {
    final sessions = await _localDb.getCachedSessions(userId);
    try {
      return sessions.firstWhere((s) => s.id == workoutId);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<List<WalkingSession>> getWorkouts(String userId) async {
    return _localDb.getCachedSessions(userId);
  }

  @override
  Future<List<WalkingSession>> getWorkoutsInRange(
      String userId, DateTime start, DateTime end) async {
    final sessions = await _localDb.getCachedSessions(userId);
    return sessions
        .where((s) =>
            s.startTime.isAfter(start.subtract(const Duration(days: 1))) &&
            s.startTime.isBefore(end.add(const Duration(days: 1))))
        .toList()
      ..sort((a, b) => b.startTime.compareTo(a.startTime));
  }

  @override
  Future<void> deleteWorkout(String userId, String workoutId) async {
    // The local database doesn't expose a per-session delete from
    // cached_sessions yet — use queueDeletion for sync tombstone flow.
    await _localDb.queueDeletion(workoutId, userId);
  }
}
