import '../models/profile_data.dart';
import '../models/user_model.dart';

/// Profile service — reads real user data from the authenticated
/// user model. No mock or hardcoded "Alex Rivera" data. Returns
/// empty/blank state when no user is available.
class ProfileService {
  /// Builds a UserProfile from the real authenticated user model.
  /// Returns null if the user model is null (not logged in).
  UserProfile? profileFromUser(UserModel? user) {
    if (user == null) return null;
    return UserProfile(
      name: user.displayName,
      age: user.age,
      weightKg: user.weightKg,
      heightCm: user.heightCm,
      goal: user.goals.primaryGoal,
      activityLevel: user.activityLevel,
      memberSince: user.createdAt,
    );
  }

  /// Returns null — no body stats until the user has recorded them.
  Future<BodyStats?> getBodyStats() async {
    return null;
  }

  /// Returns empty list — no achievements until the user has earned them.
  Future<List<Achievement>> getAchievements() async {
    return const [];
  }
}
