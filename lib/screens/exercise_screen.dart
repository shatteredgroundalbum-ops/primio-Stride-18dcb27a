import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:provider/provider.dart';
import '../providers/exercise_provider.dart';
import '../theme/theme.dart';
import '../widgets/common/section_header.dart';
import '../widgets/exercise/exercise_category_grid.dart';
import '../widgets/exercise/recent_workouts_list.dart';
import '../widgets/exercise/today_workout_card.dart';

class ExerciseScreen extends StatelessWidget {
  const ExerciseScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<ExerciseProvider>();
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    if (provider.isLoading) {
      return Scaffold(
        body: Center(
          child: CircularProgressIndicator(color: appColors.exerciseAccent),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: Text('Exercise', style: text.titleLarge?.copyWith(color: appColors.exerciseAccent)),
        actions: [
          IconButton(
            icon: Icon(Icons.calendar_today, color: colors.onSurface.withOpacity(AppTheme.opacityHint)),
            onPressed: () {},
          ),
        ],
      ),
      body: SafeArea(
        child: RefreshIndicator(
          onRefresh: provider.loadExerciseData,
          color: appColors.exerciseAccent,
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              AppTheme.spacingMd, AppTheme.spacingSm, AppTheme.spacingMd, AppTheme.spacingLg,
            ),
            children: [
              const SectionHeader(
                title: 'Today\'s Workout',
                icon: Icons.fitness_center,
              ).animate().fadeIn(duration: 400.ms),
              if (provider.todayWorkout != null)
                TodayWorkoutCard(
                  workout: provider.todayWorkout!,
                  onToggleSet: provider.toggleSetCompletion,
                ).animate().fadeIn(duration: 500.ms, delay: 100.ms).slideY(begin: 0.05, end: 0),
              const SizedBox(height: AppTheme.spacingLg),
              const SectionHeader(
                title: 'Categories',
                icon: Icons.category,
              ).animate().fadeIn(duration: 400.ms, delay: 200.ms),
              ExerciseCategoryGrid(categories: provider.categories)
                  .animate().fadeIn(duration: 500.ms, delay: 300.ms),
              const SizedBox(height: AppTheme.spacingLg),
              const SectionHeader(
                title: 'Recent Workouts',
                icon: Icons.history,
                actionLabel: 'View All',
              ).animate().fadeIn(duration: 400.ms, delay: 400.ms),
              RecentWorkoutsList(workouts: provider.recentWorkouts)
                  .animate().fadeIn(duration: 500.ms, delay: 500.ms),
            ],
          ),
        ),
      ),
    );
  }
}
