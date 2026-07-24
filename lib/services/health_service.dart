import '../models/health_data.dart';

class HealthService {
  Future<DailyHealth> getDailyHealth() async {
    await Future<void>.delayed(const Duration(milliseconds: 600));
    return DailyHealth(
      steps: 7842,
      stepGoal: 10000,
      caloriesBurned: 487,
      caloriesConsumed: 1580,
      calorieGoal: 2200,
      proteinGrams: 98,
      proteinGoal: 150,
      carbsGrams: 145,
      carbsGoal: 250,
      fatGrams: 52,
      fatGoal: 73,
      waterLiters: 1.8,
      waterGoal: 3.0,
      activeMinutes: 42,
      activeMinuteGoal: 60,
      recoveryScore: 82,
      fastingState: const FastingState(
        isActive: true,
        targetDuration: Duration(hours: 16),
        elapsed: Duration(hours: 12, minutes: 34),
        protocol: '16:8',
      ),
      recentWorkouts: [
        WorkoutSummary(
          name: 'Upper Body Strength',
          type: 'Weightlifting',
          durationMinutes: 55,
          caloriesBurned: 320,
          date: DateTime.now().subtract(const Duration(hours: 4)),
          isCompleted: true,
        ),
        WorkoutSummary(
          name: 'Morning Walk',
          type: 'Cardio',
          durationMinutes: 30,
          caloriesBurned: 145,
          date: DateTime.now().subtract(const Duration(hours: 8)),
          isCompleted: true,
        ),
        WorkoutSummary(
          name: 'Core & Abs',
          type: 'Bodyweight',
          durationMinutes: 20,
          caloriesBurned: 0,
          date: DateTime.now(),
          isCompleted: false,
        ),
      ],
      weeklySteps: [8200, 6500, 9100, 7842, 0, 0, 0],
    );
  }

  List<AiInsight> getInsights(DailyHealth data) {
    final insights = <AiInsight>[];

    if (data.proteinProgress < 0.5) {
      insights.add(const AiInsight(
        title: 'Protein Intake Low',
        message: 'You\'re behind on protein. Consider a protein shake or chicken breast to hit your target.',
        type: InsightType.nutrition,
      ));
    }

    if (data.recoveryScore > 75) {
      insights.add(const AiInsight(
        title: 'Great Recovery',
        message: 'Your recovery score is excellent. Today is a good day for high-intensity training.',
        type: InsightType.recovery,
      ));
    }

    if (data.fastingState.isActive && data.fastingState.progress > 0.7) {
      insights.add(const AiInsight(
        title: 'Fast Almost Complete',
        message: 'You\'re close to completing your fast! Plan a balanced meal with protein and healthy fats.',
        type: InsightType.fasting,
      ));
    }

    if (data.stepProgress < 0.6) {
      insights.add(const AiInsight(
        title: 'Step Up Your Walk',
        message: 'A 20-minute evening walk would get you close to your daily step goal.',
        type: InsightType.steps,
      ));
    }

    return insights;
  }
}
