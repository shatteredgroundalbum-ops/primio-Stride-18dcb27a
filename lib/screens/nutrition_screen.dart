import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:provider/provider.dart';
import '../providers/nutrition_provider.dart';
import '../theme/theme.dart';
import '../widgets/common/section_header.dart';
import '../widgets/nutrition/calorie_ring_card.dart';
import '../widgets/nutrition/meal_list.dart';
import '../widgets/nutrition/micro_nutrients_card.dart';

class NutritionScreen extends StatelessWidget {
  const NutritionScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<NutritionProvider>();
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    if (provider.isLoading) {
      return Scaffold(
        body: Center(
          child: CircularProgressIndicator(color: appColors.nutritionAccent),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: Text('Nutrition', style: text.titleLarge?.copyWith(color: appColors.nutritionAccent)),
        actions: [
          IconButton(
            icon: const Icon(Icons.add_circle_outline),
            color: appColors.nutritionAccent,
            onPressed: () {},
          ),
        ],
      ),
      body: SafeArea(
        child: RefreshIndicator(
          onRefresh: provider.loadNutritionData,
          color: appColors.nutritionAccent,
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              AppTheme.spacingMd, AppTheme.spacingSm, AppTheme.spacingMd, AppTheme.spacingLg,
            ),
            children: [
              const SectionHeader(
                title: 'Daily Summary',
                icon: Icons.restaurant,
              ).animate().fadeIn(duration: 400.ms),
              CalorieRingCard(
                consumed: provider.totalCalories,
                goal: NutritionProvider.calorieGoal,
                protein: provider.totalProtein,
                proteinGoal: NutritionProvider.proteinGoal,
                carbs: provider.totalCarbs,
                carbsGoal: NutritionProvider.carbsGoal,
                fat: provider.totalFat,
                fatGoal: NutritionProvider.fatGoal,
              ).animate().fadeIn(duration: 500.ms, delay: 100.ms).slideY(begin: 0.05, end: 0),
              const SizedBox(height: AppTheme.spacingLg),
              SectionHeader(
                title: 'Today\'s Meals',
                icon: Icons.lunch_dining,
                actionLabel: 'Add Meal',
                onAction: () {},
              ).animate().fadeIn(duration: 400.ms, delay: 200.ms),
              MealList(meals: provider.meals)
                  .animate().fadeIn(duration: 500.ms, delay: 300.ms),
              const SizedBox(height: AppTheme.spacingLg),
              const SectionHeader(
                title: 'Micronutrients',
                icon: Icons.science_outlined,
              ).animate().fadeIn(duration: 400.ms, delay: 400.ms),
              MicroNutrientsCard(nutrients: provider.microNutrients)
                  .animate().fadeIn(duration: 500.ms, delay: 500.ms),
            ],
          ),
        ),
      ),
    );
  }
}
