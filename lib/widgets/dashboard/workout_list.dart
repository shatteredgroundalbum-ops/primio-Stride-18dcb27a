import 'package:flutter/material.dart';
import '../../models/health_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class WorkoutList extends StatelessWidget {
  final List<WorkoutSummary> workouts;

  const WorkoutList({super.key, required this.workouts});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        for (int i = 0; i < workouts.length; i++) ...[
          if (i > 0) const SizedBox(height: AppTheme.spacingSm),
          WorkoutTile(workout: workouts[i]),
        ],
      ],
    );
  }
}

class WorkoutTile extends StatelessWidget {
  final WorkoutSummary workout;

  const WorkoutTile({super.key, required this.workout});

  IconData get _icon {
    switch (workout.type) {
      case 'Weightlifting':
        return Icons.fitness_center;
      case 'Cardio':
        return Icons.directions_walk;
      default:
        return Icons.sports_gymnastics;
    }
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return GlassCard(
      padding: const EdgeInsets.symmetric(
        horizontal: AppTheme.spacingMd,
        vertical: AppTheme.spacingSm + 4,
      ),
      child: Row(
        children: [
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
              color: appColors.exerciseAccent.withOpacity(AppTheme.opacitySubtle),
            ),
            child: Icon(_icon, color: appColors.exerciseAccent, size: AppTheme.iconMd),
          ),
          const SizedBox(width: AppTheme.spacingSm + 4),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(workout.name, style: text.titleSmall),
                const SizedBox(height: 2),
                Text(
                  '${workout.type} · ${workout.durationMinutes} min',
                  style: text.labelSmall,
                ),
              ],
            ),
          ),
          if (workout.isCompleted)
            Container(
              padding: const EdgeInsets.symmetric(
                horizontal: AppTheme.spacingSm,
                vertical: AppTheme.spacingXs,
              ),
              decoration: BoxDecoration(
                color: appColors.success.withOpacity(AppTheme.opacityLight),
                borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
              ),
              child: Text(
                'Done',
                style: text.labelSmall?.copyWith(
                  color: appColors.success,
                  fontWeight: FontWeight.w700,
                ),
              ),
            )
          else
            Container(
              padding: const EdgeInsets.symmetric(
                horizontal: AppTheme.spacingSm,
                vertical: AppTheme.spacingXs,
              ),
              decoration: BoxDecoration(
                color: colors.primary.withOpacity(AppTheme.opacityLight),
                borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
              ),
              child: Text(
                'Upcoming',
                style: text.labelSmall?.copyWith(
                  color: colors.primary,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ),
        ],
      ),
    );
  }
}
