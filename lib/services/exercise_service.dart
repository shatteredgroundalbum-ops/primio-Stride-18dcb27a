import 'package:flutter/material.dart';
import '../models/exercise_data.dart';

class ExerciseService {
  Future<List<ExerciseCategory>> getCategories() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    return const [
      ExerciseCategory(name: 'Strength', icon: Icons.fitness_center, exerciseCount: 24, colorKey: 'exercise'),
      ExerciseCategory(name: 'Cardio', icon: Icons.directions_run, exerciseCount: 12, colorKey: 'steps'),
      ExerciseCategory(name: 'Flexibility', icon: Icons.self_improvement, exerciseCount: 8, colorKey: 'recovery'),
      ExerciseCategory(name: 'HIIT', icon: Icons.flash_on, exerciseCount: 10, colorKey: 'fasting'),
      ExerciseCategory(name: 'Bodyweight', icon: Icons.sports_gymnastics, exerciseCount: 16, colorKey: 'nutrition'),
      ExerciseCategory(name: 'Recovery', icon: Icons.spa, exerciseCount: 6, colorKey: 'recovery'),
    ];
  }

  Future<WorkoutPlan> getTodayWorkout() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    return WorkoutPlan(
      name: 'Push Day — Chest & Shoulders',
      category: 'Strength',
      estimatedMinutes: 55,
      difficulty: 'Intermediate',
      isActive: true,
      exercises: [
        Exercise(
          name: 'Barbell Bench Press',
          muscleGroup: 'Chest',
          sets: [
            const ExerciseSet(setNumber: 1, reps: 10, weight: 60, isCompleted: true),
            const ExerciseSet(setNumber: 2, reps: 8, weight: 70, isCompleted: true),
            const ExerciseSet(setNumber: 3, reps: 6, weight: 80, isCompleted: false),
            const ExerciseSet(setNumber: 4, reps: 6, weight: 80, isCompleted: false),
          ],
        ),
        Exercise(
          name: 'Overhead Press',
          muscleGroup: 'Shoulders',
          sets: [
            const ExerciseSet(setNumber: 1, reps: 10, weight: 30),
            const ExerciseSet(setNumber: 2, reps: 8, weight: 35),
            const ExerciseSet(setNumber: 3, reps: 8, weight: 35),
          ],
        ),
        Exercise(
          name: 'Incline Dumbbell Press',
          muscleGroup: 'Upper Chest',
          sets: [
            const ExerciseSet(setNumber: 1, reps: 12, weight: 22),
            const ExerciseSet(setNumber: 2, reps: 10, weight: 24),
            const ExerciseSet(setNumber: 3, reps: 10, weight: 24),
          ],
        ),
        Exercise(
          name: 'Lateral Raises',
          muscleGroup: 'Side Delts',
          sets: [
            const ExerciseSet(setNumber: 1, reps: 15, weight: 10),
            const ExerciseSet(setNumber: 2, reps: 12, weight: 12),
            const ExerciseSet(setNumber: 3, reps: 12, weight: 12),
          ],
        ),
        Exercise(
          name: 'Cable Flyes',
          muscleGroup: 'Chest',
          sets: [
            const ExerciseSet(setNumber: 1, reps: 12, weight: 15),
            const ExerciseSet(setNumber: 2, reps: 12, weight: 15),
            const ExerciseSet(setNumber: 3, reps: 10, weight: 17.5),
          ],
        ),
      ],
    );
  }

  Future<List<WorkoutPlan>> getRecentWorkouts() async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    return [
      const WorkoutPlan(
        name: 'Pull Day — Back & Biceps',
        category: 'Strength',
        exercises: [],
        estimatedMinutes: 50,
        difficulty: 'Intermediate',
      ),
      const WorkoutPlan(
        name: 'Leg Day — Quads & Glutes',
        category: 'Strength',
        exercises: [],
        estimatedMinutes: 60,
        difficulty: 'Advanced',
      ),
      const WorkoutPlan(
        name: 'Morning HIIT Session',
        category: 'HIIT',
        exercises: [],
        estimatedMinutes: 25,
        difficulty: 'Beginner',
      ),
    ];
  }
}
