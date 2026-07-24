class DailyHealth {
  final int steps;
  final int stepGoal;
  final double caloriesBurned;
  final double caloriesConsumed;
  final double calorieGoal;
  final double proteinGrams;
  final double proteinGoal;
  final double carbsGrams;
  final double carbsGoal;
  final double fatGrams;
  final double fatGoal;
  final double waterLiters;
  final double waterGoal;
  final int activeMinutes;
  final int activeMinuteGoal;
  final double recoveryScore;
  final FastingState fastingState;
  final List<WorkoutSummary> recentWorkouts;
  final List<double> weeklySteps;

  const DailyHealth({
    required this.steps,
    required this.stepGoal,
    required this.caloriesBurned,
    required this.caloriesConsumed,
    required this.calorieGoal,
    required this.proteinGrams,
    required this.proteinGoal,
    required this.carbsGrams,
    required this.carbsGoal,
    required this.fatGrams,
    required this.fatGoal,
    required this.waterLiters,
    required this.waterGoal,
    required this.activeMinutes,
    required this.activeMinuteGoal,
    required this.recoveryScore,
    required this.fastingState,
    required this.recentWorkouts,
    required this.weeklySteps,
  });

  double get stepProgress => (steps / stepGoal).clamp(0.0, 1.0);
  double get calorieProgress => (caloriesConsumed / calorieGoal).clamp(0.0, 1.0);
  double get proteinProgress => (proteinGrams / proteinGoal).clamp(0.0, 1.0);
  double get carbsProgress => (carbsGrams / carbsGoal).clamp(0.0, 1.0);
  double get fatProgress => (fatGrams / fatGoal).clamp(0.0, 1.0);
  double get waterProgress => (waterLiters / waterGoal).clamp(0.0, 1.0);
  double get activeProgress => (activeMinutes / activeMinuteGoal).clamp(0.0, 1.0);
}

class FastingState {
  final bool isActive;
  final DateTime? startTime;
  final Duration targetDuration;
  final Duration elapsed;
  final String protocol;

  const FastingState({
    required this.isActive,
    this.startTime,
    required this.targetDuration,
    required this.elapsed,
    required this.protocol,
  });

  double get progress => targetDuration.inMinutes > 0
      ? (elapsed.inMinutes / targetDuration.inMinutes).clamp(0.0, 1.0)
      : 0.0;

  String get remainingFormatted {
    final remaining = targetDuration - elapsed;
    if (remaining.isNegative) return '0h 0m';
    final hours = remaining.inHours;
    final minutes = remaining.inMinutes.remainder(60);
    return '${hours}h ${minutes}m';
  }

  String get elapsedFormatted {
    final hours = elapsed.inHours;
    final minutes = elapsed.inMinutes.remainder(60);
    return '${hours}h ${minutes}m';
  }
}

class WorkoutSummary {
  final String name;
  final String type;
  final int durationMinutes;
  final double caloriesBurned;
  final DateTime date;
  final bool isCompleted;

  const WorkoutSummary({
    required this.name,
    required this.type,
    required this.durationMinutes,
    required this.caloriesBurned,
    required this.date,
    required this.isCompleted,
  });
}

class AiInsight {
  final String title;
  final String message;
  final InsightType type;

  const AiInsight({
    required this.title,
    required this.message,
    required this.type,
  });
}

enum InsightType { nutrition, exercise, recovery, fasting, steps }
