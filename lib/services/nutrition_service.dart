import '../models/nutrition_data.dart';

/// Nutrition service — returns empty state by default. No mock or
/// hardcoded data. When the user logs meals through the UI, the data
/// will be persisted and read back from storage.
class NutritionService {
  /// Returns empty list — no meals logged until the user adds them.
  Future<List<Meal>> getTodayMeals() async {
    return const [];
  }

  /// Returns empty list — no micronutrient data until the user has meals.
  Future<List<MicroNutrient>> getMicroNutrients() async {
    return const [];
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
