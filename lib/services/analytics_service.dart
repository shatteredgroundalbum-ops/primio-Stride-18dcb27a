import '../models/walking_session.dart';

class WalkingStats {
  final double totalDistanceKm;
  final double weeklyDistanceKm;
  final double monthlyDistanceKm;
  final int totalWorkouts;
  final double averagePaceMinPerKm;
  final double fastestPaceMinPerKm;
  final double longestDistanceKm;
  final int currentStreak;
  final int longestStreak;
  final Duration totalDuration;
  final double totalCaloriesBurned;

  const WalkingStats({
    this.totalDistanceKm = 0,
    this.weeklyDistanceKm = 0,
    this.monthlyDistanceKm = 0,
    this.totalWorkouts = 0,
    this.averagePaceMinPerKm = 0,
    this.fastestPaceMinPerKm = 0,
    this.longestDistanceKm = 0,
    this.currentStreak = 0,
    this.longestStreak = 0,
    this.totalDuration = Duration.zero,
    this.totalCaloriesBurned = 0,
  });
}

class AnalyticsService {
  WalkingStats calculateStats(List<WalkingSession> sessions) {
    final completed = sessions
        .where((s) => s.status == SessionStatus.completed)
        .toList()
      ..sort((a, b) => b.startTime.compareTo(a.startTime));

    if (completed.isEmpty) return const WalkingStats();

    final now = DateTime.now();
    final weekAgo = now.subtract(const Duration(days: 7));
    final monthAgo = now.subtract(const Duration(days: 30));

    final totalDist =
        completed.fold<double>(0, (s, e) => s + e.distanceMeters) / 1000;
    final weeklyDist = completed
            .where((s) => s.startTime.isAfter(weekAgo))
            .fold<double>(0, (s, e) => s + e.distanceMeters) /
        1000;
    final monthlyDist = completed
            .where((s) => s.startTime.isAfter(monthAgo))
            .fold<double>(0, (s, e) => s + e.distanceMeters) /
        1000;

    final paces = completed
        .where((s) => s.averagePaceMinPerKm > 0)
        .map((s) => s.averagePaceMinPerKm)
        .toList();
    final avgPace =
        paces.isEmpty ? 0.0 : paces.reduce((a, b) => a + b) / paces.length;
    final fastestPace =
        paces.isEmpty ? 0.0 : paces.reduce((a, b) => a < b ? a : b);
    final longestDist = completed
        .map((s) => s.distanceMeters / 1000)
        .reduce((a, b) => a > b ? a : b);

    final totalDuration = completed.fold<Duration>(
        Duration.zero, (sum, s) => sum + s.duration);
    final totalCalories =
        completed.fold<double>(0, (s, e) => s + e.caloriesBurned);

    final streaks = _calculateStreaks(completed);

    return WalkingStats(
      totalDistanceKm: totalDist,
      weeklyDistanceKm: weeklyDist,
      monthlyDistanceKm: monthlyDist,
      totalWorkouts: completed.length,
      averagePaceMinPerKm: avgPace,
      fastestPaceMinPerKm: fastestPace,
      longestDistanceKm: longestDist,
      currentStreak: streaks.$1,
      longestStreak: streaks.$2,
      totalDuration: totalDuration,
      totalCaloriesBurned: totalCalories,
    );
  }

  (int current, int longest) _calculateStreaks(List<WalkingSession> sorted) {
    if (sorted.isEmpty) return (0, 0);

    final days = sorted
        .map((s) =>
            DateTime(s.startTime.year, s.startTime.month, s.startTime.day))
        .toSet()
        .toList()
      ..sort((a, b) => b.compareTo(a));

    if (days.length <= 1) return (1, 1);

    int current = 1;
    int longest = 1;
    int streak = 1;

    for (int i = 1; i < days.length; i++) {
      final diff = days[i - 1].difference(days[i]).inDays;
      if (diff == 1) {
        streak++;
        if (streak > longest) longest = streak;
      } else {
        if (i == 1) current = streak;
        streak = 1;
      }
    }

    final today = DateTime.now();
    final todayDate = DateTime(today.year, today.month, today.day);
    if (days.first == todayDate ||
        todayDate.difference(days.first).inDays == 1) {
      current = streak;
    } else {
      current = 0;
    }

    return (current, longest);
  }
}
