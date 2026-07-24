import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class MacrosCard extends StatelessWidget {
  final double caloriesConsumed;
  final double calorieGoal;
  final double protein;
  final double proteinGoal;
  final double carbs;
  final double carbsGoal;
  final double fat;
  final double fatGoal;

  const MacrosCard({
    super.key,
    required this.caloriesConsumed,
    required this.calorieGoal,
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
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final colors = Theme.of(context).colorScheme;
    final remaining = (calorieGoal - caloriesConsumed).clamp(0.0, calorieGoal);

    return GlassCard(
      child: Column(
        children: [
          Row(
            children: [
              Icon(Icons.restaurant, size: AppTheme.iconSm, color: appColors.nutritionAccent),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Nutrition', style: text.titleSmall),
              const Spacer(),
              Text(
                '${remaining.toInt()} kcal left',
                style: text.labelMedium?.copyWith(color: appColors.nutritionAccent),
              ),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          Row(
            children: [
              Text(
                '${caloriesConsumed.toInt()}',
                style: text.headlineMedium?.copyWith(color: appColors.nutritionAccent),
              ),
              Text(
                ' / ${calorieGoal.toInt()} kcal',
                style: text.bodyMedium?.copyWith(
                  color: colors.onSurface.withOpacity(AppTheme.opacityHint),
                ),
              ),
            ],
          ),
          const SizedBox(height: AppTheme.spacingSm),
          ClipRRect(
            borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
            child: LinearProgressIndicator(
              value: (caloriesConsumed / calorieGoal).clamp(0.0, 1.0),
              minHeight: 6,
              backgroundColor: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
              valueColor: AlwaysStoppedAnimation(appColors.nutritionAccent),
            ),
          ),
          const SizedBox(height: AppTheme.spacingMd),
          IntrinsicHeight(
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Expanded(
                  child: _MacroTile(
                    label: 'Protein',
                    value: '${protein.toInt()}g',
                    goal: '${proteinGoal.toInt()}g',
                    progress: (protein / proteinGoal).clamp(0.0, 1.0),
                    color: const Color(0xFF60A5FA),
                  ),
                ),
                const SizedBox(width: AppTheme.spacingSm),
                Expanded(
                  child: _MacroTile(
                    label: 'Carbs',
                    value: '${carbs.toInt()}g',
                    goal: '${carbsGoal.toInt()}g',
                    progress: (carbs / carbsGoal).clamp(0.0, 1.0),
                    color: appColors.warning,
                  ),
                ),
                const SizedBox(width: AppTheme.spacingSm),
                Expanded(
                  child: _MacroTile(
                    label: 'Fat',
                    value: '${fat.toInt()}g',
                    goal: '${fatGoal.toInt()}g',
                    progress: (fat / fatGoal).clamp(0.0, 1.0),
                    color: appColors.exerciseAccent,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _MacroTile extends StatelessWidget {
  final String label;
  final String value;
  final String goal;
  final double progress;
  final Color color;

  const _MacroTile({
    required this.label,
    required this.value,
    required this.goal,
    required this.progress,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;

    return Container(
      padding: const EdgeInsets.all(AppTheme.spacingSm + 4),
      decoration: BoxDecoration(
        color: color.withOpacity(AppTheme.opacitySubtle),
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
      ),
      child: Column(
        children: [
          Text(label, style: text.labelSmall?.copyWith(color: color)),
          const SizedBox(height: AppTheme.spacingXs),
          Text(value, style: text.titleSmall),
          Text(
            '/ $goal',
            style: text.labelSmall?.copyWith(
              color: colors.onSurface.withOpacity(AppTheme.opacityHint),
            ),
          ),
          const SizedBox(height: AppTheme.spacingSm),
          ClipRRect(
            borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
            child: LinearProgressIndicator(
              value: progress,
              minHeight: 4,
              backgroundColor: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
              valueColor: AlwaysStoppedAnimation(color),
            ),
          ),
        ],
      ),
    );
  }
}
