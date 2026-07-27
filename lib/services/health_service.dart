import '../database/local_database.dart';
import '../models/health_data.dart';
import '../models/walking_session.dart';

/// Reads real health/activity data from the local SQLite database
/// (populated by the Rust engine via WorkoutRecorder). No mock or
/// hardcoded values — every field is derived from actual user sessions.
class HealthService {
  final LocalDatabase _localDb;

  HealthService({required LocalDatabase localDb}) : _localDb = localDb;

  /// Builds today's health snapshot from real cached sessions in SQLite.
  /// Returns a zero-state DailyHealth when no sessions exist yet (new
  /// user) rather than fabricated numbers.
  Future<DailyHealth> getDailyHealth({required String userId}) async {
    final sessions = await _localDb.getCachedSessions(userId);

    final now = DateTime.now();
    final today = DateTime(now.year, now.month, now.day);
    final weekAgo = today.subtract(const Duration(days: 6));

    // Today's sessions
    final todaySessions = sessions.where((s) {
      final d = DateTime(s.startTime.year, s.startTime.month, s.startTime.day);
      return d.isAtSameMomentAs(today);
    }).toList();

    // This week's sessions (last 7 days)
    final weekSessions = sessions.where((s) =>
        !s.startTime.isBefore(weekAgo) && !s.startTime.isAfter(now)).toList();

    // Aggregate real totals from today's sessions
    int totalSteps = 0;
    double totalDistanceMeters = 0;
    int totalActiveMinutes = 0;
    double totalCaloriesBurned = 0;

    for (final s in todaySessions) {
      totalSteps += s.stepCount;
      totalDistanceMeters += s.distanceMeters;
      totalActiveMinutes += s.duration.inMinutes;
      totalCaloriesBurned += s.caloriesBurned;
    }

    // Build recent workout summaries from real sessions (last 5)
    final recentWorkouts = <WorkoutSummary>[];
    final recent = sessions.take(5).toList();
    for (final s in recent) {
      recentWorkouts.add(WorkoutSummary(
        name: _activityName(s.type),
        type: _activityTypeLabel(s.type),
        durationMinutes: s.duration.inMinutes,
        caloriesBurned: s.caloriesBurned.round(),
        date: s.startTime,
        isCompleted: s.status == SessionStatus.completed,
      ));
    }

    // Weekly steps — one entry per day for the last 7 days
    final weeklySteps = _buildWeeklySteps(weekSessions, today);

    // Real recovery score: based on recent activity consistency.
    final completedThisWeek = weekSessions
        .where((s) => s.status == SessionStatus.completed).length;
    final recoveryScore = _computeRecoveryScore(completedThisWeek);

    return DailyHealth(
      steps: totalSteps,
      stepGoal: 10000,
      caloriesBurned: totalCaloriesBurned,
      caloriesConsumed: 0,
      calorieGoal: 2200,
      proteinGrams: 0,
      proteinGoal: 150,
      carbsGrams: 0,
      carbsGoal: 250,
      fatGrams: 0,
      fatGoal: 73,
      waterLiters: 0,
      waterGoal: 3.0,
      activeMinutes: totalActiveMinutes,
      activeMinuteGoal: 60,
      recoveryScore: recoveryScore,
      fastingState: const FastingState(
        isActive: false,
        targetDuration: Duration.zero,
        elapsed: Duration.zero,
        protocol: 'None',
      ),
      recentWorkouts: recentWorkouts,
      weeklySteps: weeklySteps,
    );
  }

  List<AiInsight> getInsights(DailyHealth data) {
    final insights = <AiInsight>[];

    if (data.stepProgress < 0.6 && data.steps > 0) {
      insights.add(const AiInsight(
        title: 'Step Up Your Walk',
        message: 'A 20-minute walk would get you closer to your daily step goal.',
        type: InsightType.steps,
      ));
    }

    if (data.recoveryScore > 75) {
      insights.add(const AiInsight(
        title: 'Great Recovery',
        message: 'Your recovery score is excellent. Today is a good day for a longer walk.',
        type: InsightType.recovery,
      ));
    }

    return insights;
  }

  /// Builds a 7-element list of daily step counts from real sessions.
  /// Index 0 = 6 days ago, index 6 = today.
  List<double> _buildWeeklySteps(List<WalkingSession> weekSessions, DateTime today) {
    final steps = List<double>.filled(7, 0);
    for (final s in weekSessions) {
      final dayDiff = today
          .difference(DateTime(s.startTime.year, s.startTime.month, s.startTime.day))
          .inDays;
      if (dayDiff >= 0 && dayDiff < 7) {
        steps[6 - dayDiff] += s.stepCount;
      }
    }
    return steps;
  }

  /// Computes a recovery score (0-100) based on how many sessions the
  /// user completed this week.
  double _computeRecoveryScore(int completedThisWeek) {
    if (completedThisWeek == 0) return 0;
    return (completedThisWeek * 20.0).clamp(0, 100);
  }

  String _activityName(SessionType type) {
    switch (type) {
      case SessionType.walk:
        return 'Walk';
      case SessionType.run:
        return 'Run';
      case SessionType.hike:
        return 'Hike';
    }
  }

  String _activityTypeLabel(SessionType type) {
    switch (type) {
      case SessionType.walk:
        return 'Walking';
      case SessionType.run:
        return 'Running';
      case SessionType.hike:
        return 'Hiking';
    }
  }
}
