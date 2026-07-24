import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';
import '../common/progress_ring.dart';

class CalorieRingCard extends StatelessWidget {
  final double consumed;
  final double goal;
  final double protein;
  final double proteinGoal;
  final double carbs;
  final double carbsGoal;
  final double fat;
  final double fatGoal;

  const CalorieRingCard({
    super.key,
    required this.consumed,
    required this.goal,
    required this.protein,
    required this.proteinGoal,
    required this.carbs,
    required this.carbsGoal,
    required this.fat,
    required this.fatGoal,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final remaining = (goal - consumed).clamp(0.0, goal);

    return GlassCard(
      child: Column(
        children: [
          Row(
            children: [
              ProgressRing(
                progress: (consumed / goal).clamp(0.0, 1.0),
                size: 110,
                strokeWidth: 9,
                activeColor: appColors.nutritionAccent,
                center: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      '${remaining.toInt()}',
                      style: text.headlineMedium?.copyWith(color: appColors.nutritionAccent),
                    ),
                    Text('kcal left', style: text.labelSmall),
                  ],
                ),
              ),
              const SizedBox(width: AppTheme.spacingMd),
              Expanded(
                child: Column(
                  children: [
                    _MacroRow(label: 'Protein', value: protein, goal: proteinGoal, color: const Color(0xFF60A5FA)),
                    const SizedBox(height: AppTheme.spacingSm + 2),
                    _MacroRow(label: 'Carbs', value: carbs, goal: carbsGoal, color: appColors.warning),
                    const SizedBox(height: AppTheme.spacingSm + 2),
                    _MacroRow(label: 'Fat', value: fat, goal: fatGoal, color: appColors.exerciseAccent),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: AppTheme.spacingSm),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                '${consumed.toInt()} consumed',
                style: text.labelMedium?.copyWith(color: appColors.nutritionAccent),
              ),
              Text(
                'Goal: ${goal.toInt()} kcal',
                style: text.labelMedium?.copyWith(
                  color: colors.onSurface.withOpacity(AppTheme.opacityHint),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _MacroRow extends StatelessWidget {
  final String label;
  final double value;
  final double goal;
  final Color color;

  const _MacroRow({
    required this.label,
    required this.value,
    required this.goal,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(label, style: text.labelSmall?.copyWith(color: color)),
            Text(
              '${value.toInt()}/${goal.toInt()}g',
              style: text.labelSmall,
            ),
          ],
        ),
        const SizedBox(height: AppTheme.spacingXs),
        ClipRRect(
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          child: LinearProgressIndicator(
            value: (value / goal).clamp(0.0, 1.0),
            minHeight: 5,
            backgroundColor: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
            valueColor: AlwaysStoppedAnimation(color),
          ),
        ),
      ],
    );
  }
}
