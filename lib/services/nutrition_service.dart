import '../models/nutrition_data.dart';

class NutritionService {
  Future<List<Meal>> getTodayMeals() async {
    await Future<void>.delayed(const Duration(milliseconds: 400));
    return [
      Meal(
        name: 'Breakfast',
        type: MealType.breakfast,
        time: DateTime.now().copyWith(hour: 8, minute: 0),
        foods: const [
          FoodItem(name: 'Greek Yogurt', calories: 130, protein: 15, carbs: 8, fat: 4, servingSize: '170g', fiber: 0),
          FoodItem(name: 'Granola', calories: 220, protein: 5, carbs: 38, fat: 7, servingSize: '50g', fiber: 3),
          FoodItem(name: 'Blueberries', calories: 42, protein: 0.5, carbs: 11, fat: 0.2, servingSize: '75g', fiber: 1.8),
        ],
      ),
      Meal(
        name: 'Lunch',
        type: MealType.lunch,
        time: DateTime.now().copyWith(hour: 12, minute: 30),
        foods: const [
          FoodItem(name: 'Grilled Chicken Breast', calories: 284, protein: 53, carbs: 0, fat: 6, servingSize: '200g', fiber: 0),
          FoodItem(name: 'Brown Rice', calories: 216, protein: 5, carbs: 45, fat: 1.8, servingSize: '200g', fiber: 3.5),
          FoodItem(name: 'Mixed Salad', calories: 45, protein: 2, carbs: 8, fat: 0.5, servingSize: '150g', fiber: 3),
        ],
      ),
      Meal(
        name: 'Protein Shake',
        type: MealType.snack,
        time: DateTime.now().copyWith(hour: 15, minute: 0),
        foods: const [
          FoodItem(name: 'Whey Protein', calories: 120, protein: 24, carbs: 3, fat: 1, servingSize: '1 scoop', fiber: 0),
          FoodItem(name: 'Banana', calories: 105, protein: 1.3, carbs: 27, fat: 0.4, servingSize: '1 medium', fiber: 3.1),
          FoodItem(name: 'Almond Milk', calories: 30, protein: 1, carbs: 1, fat: 2.5, servingSize: '240ml', fiber: 0),
        ],
      ),
    ];
  }

  Future<List<MicroNutrient>> getMicroNutrients() async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    return const [
      MicroNutrient(name: 'Vitamin D', amount: 15, unit: 'mcg', dailyPercent: 0.75),
      MicroNutrient(name: 'Iron', amount: 12, unit: 'mg', dailyPercent: 0.67),
      MicroNutrient(name: 'Calcium', amount: 800, unit: 'mg', dailyPercent: 0.80),
      MicroNutrient(name: 'Vitamin C', amount: 65, unit: 'mg', dailyPercent: 0.72),
      MicroNutrient(name: 'Potassium', amount: 2800, unit: 'mg', dailyPercent: 0.60),
      MicroNutrient(name: 'Fiber', amount: 18, unit: 'g', dailyPercent: 0.64),
      MicroNutrient(name: 'Vitamin B12', amount: 2.0, unit: 'mcg', dailyPercent: 0.83),
      MicroNutrient(name: 'Zinc', amount: 8, unit: 'mg', dailyPercent: 0.73),
    ];
  }

  double getTotalCalories(List<Meal> meals) =>
      meals.fold(0, (sum, m) => sum + m.totalCalories);

  double getTotalProtein(List<Meal> meals) =>
      meals.fold(0, (sum, m) => sum + m.totalProtein);

  double getTotalCarbs(List<Meal> meals) =>
      meals.fold(0, (sum, m) => sum + m.totalCarbs);

  double getTotalFat(List<Meal> meals) =>
      meals.fold(0, (sum, m) => sum + m.totalFat);
}
