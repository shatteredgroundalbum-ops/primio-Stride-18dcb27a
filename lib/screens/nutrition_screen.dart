import 'package:flutter/material.dart';
import '../theme/theme.dart';
import '../widgets/common/placeholder_screen.dart';

class NutritionScreen extends StatelessWidget {
  const NutritionScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    return PlaceholderScreen(
      title: 'Nutrition',
      icon: Icons.restaurant,
      accentColor: appColors.nutritionAccent,
      description: 'Meal planning, calorie tracking, and macro analysis to fuel your fitness journey.',
    );
  }
}
