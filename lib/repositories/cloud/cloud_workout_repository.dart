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

/// In-memory implementation for development. Swap with
/// `FirestoreWorkoutRepository` when Firebase is connected.
class MockCloudWorkoutRepository implements CloudWorkoutRepository {
  final List<WalkingSession> _store = [];

  @override
  Future<void> saveWorkout(WalkingSession session) async {
    await Future.delayed(const Duration(milliseconds: 100));
    _store.removeWhere((s) => s.id == session.id);
    _store.add(session);
  }

  @override
  Future<WalkingSession?> getWorkout(
      String userId, String workoutId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    try {
      return _store
          .firstWhere((s) => s.id == workoutId && s.userId == userId);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<List<WalkingSession>> getWorkouts(String userId) async {
    await Future.delayed(const Duration(milliseconds: 80));
    final userSessions =
        _store.where((s) => s.userId == userId).toList()
          ..sort((a, b) => b.startTime.compareTo(a.startTime));
    return userSessions;
  }

  @override
  Future<List<WalkingSession>> getWorkoutsInRange(
      String userId, DateTime start, DateTime end) async {
    await Future.delayed(const Duration(milliseconds: 80));
    return _store
        .where((s) =>
            s.userId == userId &&
            s.startTime.isAfter(start) &&
            s.startTime.isBefore(end))
        .toList()
      ..sort((a, b) => b.startTime.compareTo(a.startTime));
  }

  @override
  Future<void> deleteWorkout(String userId, String workoutId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    _store.removeWhere(
        (s) => s.id == workoutId && s.userId == userId);
  }
}
