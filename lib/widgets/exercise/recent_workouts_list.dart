import 'package:flutter/material.dart';
import '../../models/exercise_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class RecentWorkoutsList extends StatelessWidget {
  final List<WorkoutPlan> workouts;

  const RecentWorkoutsList({super.key, required this.workouts});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Column(
      children: [
        for (int i = 0; i < workouts.length; i++) ...[
          if (i > 0) const SizedBox(height: AppTheme.spacingSm),
          GlassCard(
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
                  child: Icon(
                    workouts[i].category == 'HIIT' ? Icons.flash_on : Icons.fitness_center,
                    color: appColors.exerciseAccent,
                    size: AppTheme.iconMd,
                  ),
                ),
                const SizedBox(width: AppTheme.spacingSm + 4),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(workouts[i].name, style: text.titleSmall, maxLines: 1, overflow: TextOverflow.ellipsis),
                      const SizedBox(height: 2),
                      Text(
                        '${workouts[i].category} · ${workouts[i].estimatedMinutes} min · ${workouts[i].difficulty}',
                        style: text.labelSmall,
                      ),
                    ],
                  ),
                ),
                Icon(Icons.chevron_right, size: AppTheme.iconMd, color: colors.onSurface.withOpacity(AppTheme.opacityHint)),
              ],
            ),
          ),
        ],
      ],
    );
  }
}
