class Meal {
  final String name;
  final MealType type;
  final List<FoodItem> foods;
  final DateTime time;

  const Meal({
    required this.name,
    required this.type,
    required this.foods,
    required this.time,
  });

  double get totalCalories => foods.fold(0, (sum, f) => sum + f.calories);
  double get totalProtein => foods.fold(0, (sum, f) => sum + f.protein);
  double get totalCarbs => foods.fold(0, (sum, f) => sum + f.carbs);
  double get totalFat => foods.fold(0, (sum, f) => sum + f.fat);
}

enum MealType { breakfast, lunch, dinner, snack }

class FoodItem {
  final String name;
  final double calories;
  final double protein;
  final double carbs;
  final double fat;
  final String servingSize;
  final double fiber;

  const FoodItem({
    required this.name,
    required this.calories,
    required this.protein,
    required this.carbs,
    required this.fat,
    required this.servingSize,
    this.fiber = 0,
  });
}

class MicroNutrient {
  final String name;
  final double amount;
  final String unit;
  final double dailyPercent;

  const MicroNutrient({
    required this.name,
    required this.amount,
    required this.unit,
    required this.dailyPercent,
  });
}
