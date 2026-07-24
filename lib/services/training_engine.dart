import 'package:uuid/uuid.dart';

import '../models/user_model.dart';
import '../models/walking_plan.dart';
import '../models/walking_session.dart';

/// Client-side training engine that generates progressive walking/running
/// plans. When Firebase Cloud Functions are added, plan generation can be
/// delegated to Gemini via a server call — this class serves as the local
/// fallback and offline planner.
class TrainingEngine {
  static const _uuid = Uuid();

  WalkingPlan generatePlan({
    required UserModel user,
    required List<WalkingSession> recentSessions,
  }) {
    final fitnessLevel = _assessFitnessLevel(recentSessions);
    final weeks = _planDurationWeeks(user.goals.primaryGoal);
    final days = <PlanDay>[];

    for (int week = 1; week <= weeks; week++) {
      for (int dayInWeek = 1; dayInWeek <= 7; dayInWeek++) {
        final dayNumber = (week - 1) * 7 + dayInWeek;
        days.add(_createPlanDay(
          dayNumber: dayNumber,
          weekNumber: week,
          dayInWeek: dayInWeek,
          fitnessLevel: fitnessLevel,
          week: week,
          goal: user.goals.primaryGoal,
        ));
      }
    }

    return WalkingPlan(
      id: _uuid.v4(),
      userId: user.id,
      name: _planName(user.goals.primaryGoal),
      goal: user.goals.primaryGoal,
      durationWeeks: weeks,
      days: days,
      createdAt: DateTime.now(),
    );
  }

  double _assessFitnessLevel(List<WalkingSession> sessions) {
    if (sessions.isEmpty) return 0.3;
    final completed =
        sessions.where((s) => s.status == SessionStatus.completed).toList();
    if (completed.isEmpty) return 0.3;

    final avgDistance = completed.fold<double>(
            0, (sum, s) => sum + s.distanceMeters) /
        completed.length;
    final avgDuration = completed.fold<int>(
            0, (sum, s) => sum + s.duration.inMinutes) /
        completed.length;

    final distanceScore = (avgDistance / 5000).clamp(0.0, 1.0);
    final durationScore = (avgDuration / 60).clamp(0.0, 1.0);
    return (distanceScore * 0.6 + durationScore * 0.4).clamp(0.2, 1.0);
  }

  int _planDurationWeeks(String goal) => switch (goal) {
        'lose_weight' => 8,
        'endurance' => 12,
        '5k' => 8,
        '10k' => 12,
        _ => 6,
      };

  String _planName(String goal) => switch (goal) {
        'lose_weight' => 'Fat Burn Walking Plan',
        'endurance' => 'Endurance Builder',
        '5k' => 'Couch to 5K',
        '10k' => '10K Training Plan',
        _ => 'Stay Active Plan',
      };

  PlanDay _createPlanDay({
    required int dayNumber,
    required int weekNumber,
    required int dayInWeek,
    required double fitnessLevel,
    required int week,
    required String goal,
  }) {
    if (dayInWeek == 7) {
      return PlanDay(
        dayNumber: dayNumber,
        weekNumber: weekNumber,
        activityType: PlanActivityType.rest,
        targetMinutes: 0,
        notes: 'Rest day — recover and hydrate',
      );
    }

    if (dayInWeek == 3) {
      return PlanDay(
        dayNumber: dayNumber,
        weekNumber: weekNumber,
        activityType: PlanActivityType.recovery,
        targetMinutes: 15 + (fitnessLevel * 10).round(),
        notes: 'Light recovery walk — easy pace',
      );
    }

    final progressMultiplier = 1.0 + (week - 1) * 0.1;
    final baseMinutes = 20 + (fitnessLevel * 20).round();
    final targetMinutes =
        (baseMinutes * progressMultiplier).round().clamp(15, 90);

    PlanActivityType type;
    String notes;
    if (dayInWeek == 2 || dayInWeek == 5) {
      type = PlanActivityType.interval;
      notes =
          'Interval training — alternate fast/easy pace every 2 minutes';
    } else {
      type = goal == '5k' || goal == '10k'
          ? PlanActivityType.run
          : PlanActivityType.walk;
      notes = type == PlanActivityType.run
          ? 'Steady-state run — maintain comfortable pace'
          : 'Brisk walk — keep heart rate elevated';
    }

    final baseDistanceKm = type == PlanActivityType.run ? 3.0 : 2.0;
    final targetDistance = baseDistanceKm * fitnessLevel * progressMultiplier;

    return PlanDay(
      dayNumber: dayNumber,
      weekNumber: weekNumber,
      activityType: type,
      targetMinutes: targetMinutes,
      targetDistanceKm: double.parse(targetDistance.toStringAsFixed(1)),
      notes: notes,
    );
  }

  /// Whether the user should take a recovery day based on recent activity.
  bool shouldRecover(List<WalkingSession> recentSessions) {
    final last3Days = recentSessions
        .where((s) =>
            s.startTime
                .isAfter(DateTime.now().subtract(const Duration(days: 3))) &&
            s.status == SessionStatus.completed)
        .toList();
    return last3Days.length >= 3;
  }

  /// Daily calorie-burn target using Mifflin-St Jeor BMR.
  double dailyCalorieBurnTarget(UserModel user) {
    final bmr =
        10 * user.weightKg + 6.25 * user.heightCm - 5 * user.age + 5;
    final activityMultiplier = switch (user.activityLevel) {
      'sedentary' => 1.2,
      'light' => 1.375,
      'moderate' => 1.55,
      'active' => 1.725,
      'very_active' => 1.9,
      _ => 1.55,
    };
    final tdee = bmr * activityMultiplier;
    return user.goals.primaryGoal == 'lose_weight' ? tdee * 0.2 : tdee * 0.15;
  }
}
