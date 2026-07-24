import 'package:flutter/material.dart';
import '../../models/exercise_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';
import '../common/progress_ring.dart';

class TodayWorkoutCard extends StatelessWidget {
  final WorkoutPlan workout;
  final void Function(int exerciseIndex, int setIndex) onToggleSet;

  const TodayWorkoutCard({
    super.key,
    required this.workout,
    required this.onToggleSet,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return GlassCard(
      borderColor: appColors.exerciseAccent.withOpacity(AppTheme.opacityLight),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              ProgressRing(
                progress: workout.progress,
                size: 52,
                strokeWidth: 5,
                activeColor: appColors.exerciseAccent,
                center: Text(
                  '${(workout.progress * 100).toInt()}%',
                  style: text.labelSmall?.copyWith(
                    color: appColors.exerciseAccent,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
              const SizedBox(width: AppTheme.spacingSm + 4),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(workout.name, style: text.titleSmall, maxLines: 1, overflow: TextOverflow.ellipsis),
                    const SizedBox(height: AppTheme.spacingXs),
                    Row(
                      children: [
                        Icon(Icons.timer_outlined, size: AppTheme.iconSm, color: appColors.subtleText),
                        const SizedBox(width: AppTheme.spacingXs),
                        Text('${workout.estimatedMinutes} min', style: text.labelSmall),
                        const SizedBox(width: AppTheme.spacingSm),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: colors.primary.withOpacity(AppTheme.opacityLight),
                            borderRadius: BorderRadius.circular(AppTheme.spacingXs),
                          ),
                          child: Text(workout.difficulty, style: text.labelSmall?.copyWith(color: colors.primary, fontWeight: FontWeight.w600)),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          ...List.generate(workout.exercises.length, (ei) {
            final exercise = workout.exercises[ei];
            final completedSets = exercise.sets.where((s) => s.isCompleted).length;
            return Padding(
              padding: EdgeInsets.only(top: ei > 0 ? AppTheme.spacingSm : 0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(exercise.name, style: text.titleSmall),
                            Text(exercise.muscleGroup, style: text.labelSmall),
                          ],
                        ),
                      ),
                      Text(
                        '$completedSets/${exercise.sets.length}',
                        style: text.labelMedium?.copyWith(
                          color: completedSets == exercise.sets.length
                              ? appColors.success
                              : appColors.subtleText,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: AppTheme.spacingSm),
                  Wrap(
                    spacing: AppTheme.spacingSm,
                    runSpacing: AppTheme.spacingSm,
                    children: List.generate(exercise.sets.length, (si) {
                      final s = exercise.sets[si];
                      return GestureDetector(
                        onTap: () => onToggleSet(ei, si),
                        child: Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: AppTheme.spacingSm + 2,
                            vertical: AppTheme.spacingXs + 2,
                          ),
                          decoration: BoxDecoration(
                            color: s.isCompleted
                                ? appColors.success.withOpacity(AppTheme.opacityLight)
                                : colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                            borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                            border: Border.all(
                              color: s.isCompleted
                                  ? appColors.success.withOpacity(0.3)
                                  : colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                              width: AppTheme.borderDefault,
                            ),
                          ),
                          child: Text(
                            '${s.reps}×${s.weight.toInt()}kg',
                            style: text.labelSmall?.copyWith(
                              color: s.isCompleted ? appColors.success : colors.onSurface,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ),
                      );
                    }),
                  ),
                  if (ei < workout.exercises.length - 1)
                    Divider(
                      height: AppTheme.spacingMd * 2,
                      color: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                    ),
                ],
              ),
            );
          }),
        ],
      ),
    );
  }
}
