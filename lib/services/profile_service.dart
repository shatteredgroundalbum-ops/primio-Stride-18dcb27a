import '../models/profile_data.dart';

class ProfileService {
  Future<UserProfile> getProfile() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    return UserProfile(
      name: 'Alex Rivera',
      age: 28,
      weightKg: 78.5,
      heightCm: 178,
      goal: 'Build Muscle',
      activityLevel: 'Very Active',
      memberSince: DateTime(2025, 3, 15),
    );
  }

  Future<BodyStats> getBodyStats() async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    final now = DateTime.now();
    return BodyStats(
      weightKg: 78.5,
      bodyFatPercent: 14.2,
      muscleMassKg: 35.8,
      weightHistory: [
        WeightEntry(date: now.subtract(const Duration(days: 30)), weightKg: 81.0),
        WeightEntry(date: now.subtract(const Duration(days: 25)), weightKg: 80.5),
        WeightEntry(date: now.subtract(const Duration(days: 20)), weightKg: 80.1),
        WeightEntry(date: now.subtract(const Duration(days: 15)), weightKg: 79.6),
        WeightEntry(date: now.subtract(const Duration(days: 10)), weightKg: 79.2),
        WeightEntry(date: now.subtract(const Duration(days: 5)), weightKg: 78.8),
        WeightEntry(date: now, weightKg: 78.5),
      ],
    );
  }

  Future<List<Achievement>> getAchievements() async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    return [
      Achievement(title: 'First Steps', description: 'Complete your first 10,000 step day', iconName: 'steps', earned: true, earnedDate: DateTime(2025, 3, 18)),
      Achievement(title: 'Iron Will', description: 'Complete 7 consecutive workout days', iconName: 'exercise', earned: true, earnedDate: DateTime(2025, 4, 2)),
      Achievement(title: 'Fasting Pro', description: 'Complete 30 fasts', iconName: 'fasting', earned: true, earnedDate: DateTime(2025, 5, 20)),
      Achievement(title: 'Hydration Hero', description: 'Hit water goal 14 days in a row', iconName: 'nutrition', earned: false),
      Achievement(title: 'Marathon Month', description: 'Walk 300,000 steps in a month', iconName: 'steps', earned: false),
      Achievement(title: 'Century Club', description: 'Log 100 workouts', iconName: 'exercise', earned: false),
    ];
  }
}
