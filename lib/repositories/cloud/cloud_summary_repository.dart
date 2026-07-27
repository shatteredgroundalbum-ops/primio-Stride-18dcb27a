import '../../database/local_database.dart';
import '../../models/walking_session.dart';

/// Daily summary matching Firestore path:
/// `users/{userId}/dailySummaries/{dateId}`
///
/// Aggregated at the end of each day or after each workout.
class DailySummary {
  final String dateId; // 'yyyy-MM-dd'
  final String userId;
  final int totalSteps;
  final double totalDistanceMeters;
  final int totalActiveMinutes;
  final double totalCaloriesBurned;
  final int workoutCount;
  final double averagePaceMinPerKm;
  final double? averageHeartRate;

  const DailySummary({
    required this.dateId,
    required this.userId,
    this.totalSteps = 0,
    this.totalDistanceMeters = 0,
    this.totalActiveMinutes = 0,
    this.totalCaloriesBurned = 0,
    this.workoutCount = 0,
    this.averagePaceMinPerKm = 0,
    this.averageHeartRate,
  });

  Map<String, dynamic> toMap() => {
        'dateId': dateId,
        'userId': userId,
        'totalSteps': totalSteps,
        'totalDistanceMeters': totalDistanceMeters,
        'totalActiveMinutes': totalActiveMinutes,
        'totalCaloriesBurned': totalCaloriesBurned,
        'workoutCount': workoutCount,
        'averagePaceMinPerKm': averagePaceMinPerKm,
        'averageHeartRate': averageHeartRate,
      };

  factory DailySummary.fromMap(Map<String, dynamic> map) => DailySummary(
        dateId: map['dateId'] as String,
        userId: map['userId'] as String,
        totalSteps: map['totalSteps'] as int? ?? 0,
        totalDistanceMeters:
            (map['totalDistanceMeters'] as num?)?.toDouble() ?? 0,
        totalActiveMinutes: map['totalActiveMinutes'] as int? ?? 0,
        totalCaloriesBurned:
            (map['totalCaloriesBurned'] as num?)?.toDouble() ?? 0,
        workoutCount: map['workoutCount'] as int? ?? 0,
        averagePaceMinPerKm:
            (map['averagePaceMinPerKm'] as num?)?.toDouble() ?? 0,
        averageHeartRate: (map['averageHeartRate'] as num?)?.toDouble(),
      );
}

/// Abstract interface for daily summaries in Firestore.
abstract class CloudSummaryRepository {
  Future<void> saveSummary(DailySummary summary);
  Future<DailySummary?> getSummary(String userId, String dateId);
  Future<List<DailySummary>> getSummariesInRange(
      String userId, DateTime start, DateTime end);
}

/// Real summary repository backed by the local SQLite database.
/// Daily summaries are computed on-the-fly from real cached sessions
/// rather than stored separately — this ensures the summary always
/// reflects the actual session data. When Firebase is connected,
/// swap for `FirestoreSummaryRepository`.
class LocalCloudSummaryRepository implements CloudSummaryRepository {
  final LocalDatabase _localDb;

  LocalCloudSummaryRepository({required LocalDatabase localDb})
      : _localDb = localDb;

  @override
  Future<void> saveSummary(DailySummary summary) async {
    // No-op: summaries are derived from cached sessions, not stored
    // separately. This method exists for interface symmetry with the
    // future Firestore implementation.
  }

  @override
  Future<DailySummary?> getSummary(String userId, String dateId) async {
    final sessions = await _localDb.getCachedSessions(userId);
    final daySessions = sessions.where((s) {
      final d = DateTime(s.startTime.year, s.startTime.month, s.startTime.day);
      final target = DateTime.parse(dateId);
      return d.isAtSameMomentAs(target);
    }).toList();

    if (daySessions.isEmpty) return null;

    return _aggregate(daySessions, userId, dateId);
  }

  @override
  Future<List<DailySummary>> getSummariesInRange(
      String userId, DateTime start, DateTime end) async {
    final sessions = await _localDb.getCachedSessions(userId);
    final summaries = <DailySummary>[];

    // Group sessions by day
    final byDay = <String, List<WalkingSession>>{};
    for (final s in sessions) {
      if (s.startTime.isAfter(start.subtract(const Duration(days: 1))) &&
          s.startTime.isBefore(end.add(const Duration(days: 1)))) {
        final dateId =
            '${s.startTime.year}-${s.startTime.month.toString().padLeft(2, '0')}-${s.startTime.day.toString().padLeft(2, '0')}';
        byDay.putIfAbsent(dateId, () => []).add(s);
      }
    }

    for (final entry in byDay.entries) {
      summaries.add(_aggregate(entry.value, userId, entry.key));
    }

    summaries.sort((a, b) => b.dateId.compareTo(a.dateId));
    return summaries;
  }

  DailySummary _aggregate(
      List<WalkingSession> sessions, String userId, String dateId) {
    int totalSteps = 0;
    double totalDistance = 0;
    int totalMinutes = 0;
    double totalCalories = 0;
    double totalPaceWeighted = 0;
    int heartRateCount = 0;
    double heartRateSum = 0;

    for (final s in sessions) {
      totalSteps += s.stepCount;
      totalDistance += s.distanceMeters;
      totalMinutes += s.duration.inMinutes;
      totalCalories += s.caloriesBurned;
      if (s.averagePaceMinPerKm > 0 && s.distanceMeters > 0) {
        totalPaceWeighted += s.averagePaceMinPerKm * s.distanceMeters;
      }
      if (s.averageHeartRate != null && s.averageHeartRate! > 0) {
        heartRateSum += s.averageHeartRate!;
        heartRateCount++;
      }
    }

    return DailySummary(
      dateId: dateId,
      userId: userId,
      totalSteps: totalSteps,
      totalDistanceMeters: totalDistance,
      totalActiveMinutes: totalMinutes,
      totalCaloriesBurned: totalCalories,
      workoutCount: sessions.length,
      averagePaceMinPerKm:
          totalDistance > 0 ? totalPaceWeighted / totalDistance : 0,
      averageHeartRate: heartRateCount > 0
          ? heartRateSum / heartRateCount
          : null,
    );
  }
}
