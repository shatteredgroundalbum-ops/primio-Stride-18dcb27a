import 'package:flutter/material.dart';
import '../../models/nutrition_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class MealList extends StatelessWidget {
  final List<Meal> meals;

  const MealList({super.key, required this.meals});

  IconData _mealIcon(MealType type) {
    switch (type) {
      case MealType.breakfast:
        return Icons.free_breakfast;
      case MealType.lunch:
        return Icons.lunch_dining;
      case MealType.dinner:
        return Icons.dinner_dining;
      case MealType.snack:
        return Icons.local_cafe;
    }
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Column(
      children: [
        for (int i = 0; i < meals.length; i++) ...[
          if (i > 0) const SizedBox(height: AppTheme.spacingSm),
          GlassCard(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Container(
                      width: 40,
                      height: 40,
                      decoration: BoxDecoration(
                        color: appColors.nutritionAccent.withOpacity(AppTheme.opacityLight),
                        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                      ),
                      child: Icon(_mealIcon(meals[i].type), color: appColors.nutritionAccent, size: AppTheme.iconSm + 2),
                    ),
                    const SizedBox(width: AppTheme.spacingSm + 4),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(meals[i].name, style: text.titleSmall),
                          Text(
                            '${meals[i].totalCalories.toInt()} kcal · ${meals[i].foods.length} items',
                            style: text.labelSmall,
                          ),
                        ],
                      ),
                    ),
                    Text(
                      '${meals[i].time.hour.toString().padLeft(2, '0')}:${meals[i].time.minute.toString().padLeft(2, '0')}',
                      style: text.labelMedium?.copyWith(color: colors.onSurface.withOpacity(AppTheme.opacityHint)),
                    ),
                  ],
                ),
                const SizedBox(height: AppTheme.spacingSm),
                ...meals[i].foods.map((food) => Padding(
                  padding: const EdgeInsets.only(top: AppTheme.spacingXs),
                  child: Row(
                    children: [
                      const SizedBox(width: AppTheme.iconXl + AppTheme.spacingSm),
                      Expanded(
                        child: Text(food.name, style: text.bodySmall, maxLines: 1, overflow: TextOverflow.ellipsis),
                      ),
                      Text(
                        '${food.calories.toInt()} kcal',
                        style: text.labelSmall?.copyWith(color: appColors.nutritionAccent),
                      ),
                    ],
                  ),
                )),
              ],
            ),
          ),
        ],
      ],
    );
  }
}
