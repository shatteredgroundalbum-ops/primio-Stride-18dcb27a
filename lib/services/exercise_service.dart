import 'package:flutter/material.dart';
import '../models/exercise_data.dart';

/// Exercise service — returns empty state by default. No mock or
/// hardcoded data. When the user logs gym workouts through the UI,
/// the data will be persisted and read back from storage.
class ExerciseService {
  /// Returns empty list until the user has real exercise data.
  Future<List<ExerciseCategory>> getCategories() async {
    return const [];
  }

  /// Returns null until the user has a real workout assigned for today.
  Future<WorkoutPlan?> getTodayWorkout() async {
    return null;
  }

  /// Returns empty list until the user has real workout history.
  Future<List<WorkoutPlan>> getRecentWorkouts() async {
    return const [];
  }
}
