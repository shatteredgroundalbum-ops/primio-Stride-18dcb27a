import 'package:flutter/foundation.dart';
import '../models/exercise_data.dart';
import '../services/exercise_service.dart';

class ExerciseProvider extends ChangeNotifier {
  final ExerciseService _service;

  ExerciseProvider({required ExerciseService service}) : _service = service;

  List<ExerciseCategory> _categories = [];
  List<ExerciseCategory> get categories => _categories;

  WorkoutPlan? _todayWorkout;
  WorkoutPlan? get todayWorkout => _todayWorkout;

  List<WorkoutPlan> _recentWorkouts = [];
  List<WorkoutPlan> get recentWorkouts => _recentWorkouts;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  Future<void> loadExerciseData() async {
    _isLoading = true;
    _error = null;
    notifyListeners();
    try {
      final results = await Future.wait([
        _service.getCategories(),
        _service.getTodayWorkout(),
        _service.getRecentWorkouts(),
      ]);
      _categories = results[0] as List<ExerciseCategory>;
      _todayWorkout = results[1] as WorkoutPlan;
      _recentWorkouts = results[2] as List<WorkoutPlan>;
    } catch (e) {
      _error = 'Failed to load exercise data';
    }
    _isLoading = false;
    notifyListeners();
  }

  void toggleSetCompletion(int exerciseIndex, int setIndex) {
    if (_todayWorkout == null) return;
    final exercises = List<Exercise>.from(_todayWorkout!.exercises);
    final exercise = exercises[exerciseIndex];
    final sets = List<ExerciseSet>.from(exercise.sets);
    final s = sets[setIndex];
    sets[setIndex] = ExerciseSet(
      setNumber: s.setNumber,
      reps: s.reps,
      weight: s.weight,
      isCompleted: !s.isCompleted,
    );
    exercises[exerciseIndex] = Exercise(
      name: exercise.name,
      muscleGroup: exercise.muscleGroup,
      sets: sets,
      notes: exercise.notes,
    );
    _todayWorkout = WorkoutPlan(
      name: _todayWorkout!.name,
      category: _todayWorkout!.category,
      exercises: exercises,
      estimatedMinutes: _todayWorkout!.estimatedMinutes,
      difficulty: _todayWorkout!.difficulty,
      isActive: _todayWorkout!.isActive,
    );
    notifyListeners();
  }
}
