import 'package:flutter/material.dart';
import '../../models/exercise_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class ExerciseCategoryGrid extends StatelessWidget {
  final List<ExerciseCategory> categories;

  const ExerciseCategoryGrid({super.key, required this.categories});

  Color _colorForKey(String key, AppColorsExtension appColors, ColorScheme colors) {
    switch (key) {
      case 'exercise':
        return appColors.exerciseAccent;
      case 'steps':
        return appColors.stepsAccent;
      case 'recovery':
        return appColors.recoveryAccent;
      case 'fasting':
        return appColors.fastingActive;
      case 'nutrition':
        return appColors.nutritionAccent;
      default:
        return colors.primary;
    }
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        mainAxisSpacing: AppTheme.spacingSm,
        crossAxisSpacing: AppTheme.spacingSm,
        childAspectRatio: 1.0,
      ),
      itemCount: categories.length,
      itemBuilder: (context, index) {
        final cat = categories[index];
        final color = _colorForKey(cat.colorKey, appColors, colors);
        return GlassCard(
          padding: const EdgeInsets.all(AppTheme.spacingSm + 4),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                width: 40,
                height: 40,
                decoration: BoxDecoration(
                  color: color.withOpacity(AppTheme.opacityLight),
                  borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                ),
                child: Icon(cat.icon, color: color, size: AppTheme.iconMd),
              ),
              const SizedBox(height: AppTheme.spacingSm),
              Text(cat.name, style: text.labelSmall?.copyWith(fontWeight: FontWeight.w600), maxLines: 1, overflow: TextOverflow.ellipsis),
              Text('${cat.exerciseCount}', style: text.labelSmall),
            ],
          ),
        );
      },
    );
  }
}
