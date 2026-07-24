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

/// In-memory implementation. Swap with `FirestoreSummaryRepository`.
class MockCloudSummaryRepository implements CloudSummaryRepository {
  final Map<String, DailySummary> _store = {};

  String _key(String userId, String dateId) => '$userId/$dateId';

  @override
  Future<void> saveSummary(DailySummary summary) async {
    await Future.delayed(const Duration(milliseconds: 50));
    _store[_key(summary.userId, summary.dateId)] = summary;
  }

  @override
  Future<DailySummary?> getSummary(String userId, String dateId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    return _store[_key(userId, dateId)];
  }

  @override
  Future<List<DailySummary>> getSummariesInRange(
      String userId, DateTime start, DateTime end) async {
    await Future.delayed(const Duration(milliseconds: 80));
    return _store.values
        .where((s) {
          if (s.userId != userId) return false;
          final date = DateTime.tryParse(s.dateId);
          if (date == null) return false;
          return date.isAfter(start.subtract(const Duration(days: 1))) &&
              date.isBefore(end.add(const Duration(days: 1)));
        })
        .toList()
      ..sort((a, b) => b.dateId.compareTo(a.dateId));
  }
}
