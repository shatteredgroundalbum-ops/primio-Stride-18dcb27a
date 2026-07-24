import 'package:flutter/material.dart';

class ExerciseCategory {
  final String name;
  final IconData icon;
  final int exerciseCount;
  final String colorKey;

  const ExerciseCategory({
    required this.name,
    required this.icon,
    required this.exerciseCount,
    required this.colorKey,
  });
}

class Exercise {
  final String name;
  final String muscleGroup;
  final List<ExerciseSet> sets;
  final String? notes;

  const Exercise({
    required this.name,
    required this.muscleGroup,
    required this.sets,
    this.notes,
  });

  int get totalVolume {
    var vol = 0.0;
    for (final s in sets) {
      vol += s.reps * s.weight;
    }
    return vol.toInt();
  }
}

class ExerciseSet {
  final int setNumber;
  final int reps;
  final double weight;
  final bool isCompleted;

  const ExerciseSet({
    required this.setNumber,
    required this.reps,
    required this.weight,
    this.isCompleted = false,
  });
}

class WorkoutPlan {
  final String name;
  final String category;
  final List<Exercise> exercises;
  final int estimatedMinutes;
  final String difficulty;
  final bool isActive;

  const WorkoutPlan({
    required this.name,
    required this.category,
    required this.exercises,
    required this.estimatedMinutes,
    required this.difficulty,
    this.isActive = false,
  });

  int get completedExercises =>
      exercises.where((e) => e.sets.every((s) => s.isCompleted)).length;

  double get progress =>
      exercises.isEmpty ? 0.0 : completedExercises / exercises.length;
}
