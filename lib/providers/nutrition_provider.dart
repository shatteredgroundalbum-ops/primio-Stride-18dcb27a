import 'package:flutter/foundation.dart';
import '../models/nutrition_data.dart';
import '../services/nutrition_service.dart';

class NutritionProvider extends ChangeNotifier {
  final NutritionService _service;

  NutritionProvider({required NutritionService service}) : _service = service;

  List<Meal> _meals = [];
  List<Meal> get meals => _meals;

  List<MicroNutrient> _microNutrients = [];
  List<MicroNutrient> get microNutrients => _microNutrients;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  double get totalCalories => _service.getTotalCalories(_meals);
  double get totalProtein => _service.getTotalProtein(_meals);
  double get totalCarbs => _service.getTotalCarbs(_meals);
  double get totalFat => _service.getTotalFat(_meals);

  static const double calorieGoal = 2200;
  static const double proteinGoal = 150;
  static const double carbsGoal = 250;
  static const double fatGoal = 73;

  double get calorieProgress => (totalCalories / calorieGoal).clamp(0.0, 1.0);
  double get proteinProgress => (totalProtein / proteinGoal).clamp(0.0, 1.0);

  Future<void> loadNutritionData() async {
    _isLoading = true;
    _error = null;
    notifyListeners();
    try {
      final results = await Future.wait([
        _service.getTodayMeals(),
        _service.getMicroNutrients(),
      ]);
      _meals = results[0] as List<Meal>;
      _microNutrients = results[1] as List<MicroNutrient>;
    } catch (e) {
      _error = 'Failed to load nutrition data';
    }
    _isLoading = false;
    notifyListeners();
  }
}
