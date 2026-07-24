import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:provider/provider.dart';
import '../providers/dashboard_provider.dart';
import '../theme/theme.dart';
import '../widgets/common/section_header.dart';
import '../widgets/dashboard/ai_insights_card.dart';
import '../widgets/dashboard/fasting_card.dart';
import '../widgets/dashboard/greeting_header.dart';
import '../widgets/dashboard/macros_card.dart';
import '../widgets/dashboard/steps_card.dart';
import '../widgets/dashboard/water_card.dart';
import '../widgets/dashboard/weekly_chart.dart';
import '../widgets/dashboard/workout_list.dart';

class DashboardScreen extends StatelessWidget {
  const DashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<DashboardProvider>();

    if (provider.isLoading) {
      return Scaffold(
        body: Center(
          child: CircularProgressIndicator(
            color: Theme.of(context).colorScheme.primary,
          ),
        ),
      );
    }

    if (provider.error != null || provider.data == null) {
      return Scaffold(
        body: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.error_outline, size: AppTheme.iconXl,
                  color: Theme.of(context).colorScheme.error),
              const SizedBox(height: AppTheme.spacingMd),
              Text(provider.error ?? 'Something went wrong',
                  style: Theme.of(context).textTheme.bodyLarge),
              const SizedBox(height: AppTheme.spacingMd),
              ElevatedButton(
                onPressed: provider.loadDashboard,
                child: const Text('Retry'),
              ),
            ],
          ),
        ),
      );
    }

    final data = provider.data!;

    return Scaffold(
      body: SafeArea(
        child: RefreshIndicator(
          onRefresh: provider.loadDashboard,
          color: Theme.of(context).colorScheme.primary,
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              AppTheme.spacingMd, AppTheme.spacingSm, AppTheme.spacingMd, AppTheme.spacingLg,
            ),
            children: [
              GreetingHeader(recoveryScore: data.recoveryScore)
                  .animate().fadeIn(duration: 400.ms).slideY(begin: -0.1, end: 0),
              const SizedBox(height: AppTheme.spacingLg),
              StepsCard(
                steps: data.steps,
                goal: data.stepGoal,
                caloriesBurned: data.caloriesBurned,
                activeMinutes: data.activeMinutes,
                activeProgress: data.activeProgress,
              ).animate().fadeIn(duration: 500.ms, delay: 100.ms).slideY(begin: 0.05, end: 0),
              const SizedBox(height: AppTheme.spacingMd),
              FastingCard(fasting: data.fastingState)
                  .animate().fadeIn(duration: 500.ms, delay: 200.ms).slideY(begin: 0.05, end: 0),
              const SizedBox(height: AppTheme.spacingMd),
              MacrosCard(
                caloriesConsumed: data.caloriesConsumed,
                calorieGoal: data.calorieGoal,
                protein: data.proteinGrams,
                proteinGoal: data.proteinGoal,
                carbs: data.carbsGrams,
                carbsGoal: data.carbsGoal,
                fat: data.fatGrams,
                fatGoal: data.fatGoal,
              ).animate().fadeIn(duration: 500.ms, delay: 300.ms).slideY(begin: 0.05, end: 0),
              const SizedBox(height: AppTheme.spacingMd),
              WaterCard(liters: data.waterLiters, goal: data.waterGoal)
                  .animate().fadeIn(duration: 500.ms, delay: 350.ms).slideY(begin: 0.05, end: 0),
              const SizedBox(height: AppTheme.spacingLg),
              const SectionHeader(
                title: 'AI Insights',
                icon: Icons.auto_awesome,
              ).animate().fadeIn(duration: 400.ms, delay: 400.ms),
              AiInsightsCard(insights: provider.insights)
                  .animate().fadeIn(duration: 500.ms, delay: 450.ms),
              const SizedBox(height: AppTheme.spacingLg),
              const SectionHeader(
                title: 'Weekly Steps',
                icon: Icons.bar_chart,
              ).animate().fadeIn(duration: 400.ms, delay: 500.ms),
              WeeklyChart(weeklySteps: data.weeklySteps)
                  .animate().fadeIn(duration: 500.ms, delay: 550.ms),
              const SizedBox(height: AppTheme.spacingLg),
              SectionHeader(
                title: 'Today\'s Workouts',
                icon: Icons.fitness_center,
                actionLabel: 'See All',
                onAction: () {},
              ).animate().fadeIn(duration: 400.ms, delay: 600.ms),
              WorkoutList(workouts: data.recentWorkouts)
                  .animate().fadeIn(duration: 500.ms, delay: 650.ms),
            ],
          ),
        ),
      ),
    );
  }
}
