import '../../models/profile_data.dart';
import '../../models/user_model.dart';

/// Abstract interface matching Firestore path:
/// `users/{userId}` — with subcollection-like fields for
/// profile, goals, settings, musicProfile, deviceConnections.
abstract class CloudUserRepository {
  Future<UserModel?> getUser(String userId);
  Future<void> createUser(UserModel user);
  Future<void> updateUser(UserModel user);
  Future<void> updateGoals(String userId, UserGoals goals);
  Future<void> updateSettings(String userId, AppSettings settings);
  Future<void> deleteUser(String userId);

  /// Achievement subcollection: `users/{userId}/achievements/{id}`
  Future<List<Achievement>> getAchievements(String userId);
  Future<void> awardAchievement(String userId, Achievement achievement);
}

/// In-memory implementation. Swap with `FirestoreUserRepository`
/// when Firebase is connected.
class MockCloudUserRepository implements CloudUserRepository {
  final Map<String, UserModel> _users = {};
  final Map<String, List<Achievement>> _achievements = {};

  @override
  Future<UserModel?> getUser(String userId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    return _users[userId];
  }

  @override
  Future<void> createUser(UserModel user) async {
    await Future.delayed(const Duration(milliseconds: 80));
    _users[user.id] = user;
    _achievements[user.id] = _defaultAchievements();
  }

  @override
  Future<void> updateUser(UserModel user) async {
    await Future.delayed(const Duration(milliseconds: 50));
    _users[user.id] = user;
  }

  @override
  Future<void> updateGoals(String userId, UserGoals goals) async {
    await Future.delayed(const Duration(milliseconds: 50));
    final user = _users[userId];
    if (user != null) {
      _users[userId] = user.copyWith(goals: goals);
    }
  }

  @override
  Future<void> updateSettings(String userId, AppSettings settings) async {
    await Future.delayed(const Duration(milliseconds: 50));
  }

  @override
  Future<void> deleteUser(String userId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    _users.remove(userId);
    _achievements.remove(userId);
  }

  @override
  Future<List<Achievement>> getAchievements(String userId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    return _achievements[userId] ?? _defaultAchievements();
  }

  @override
  Future<void> awardAchievement(
      String userId, Achievement achievement) async {
    await Future.delayed(const Duration(milliseconds: 50));
    final list = _achievements[userId] ?? [];
    list.add(achievement);
    _achievements[userId] = list;
  }

  UserModel createDefaultUser({
    required String id,
    required String email,
    required String displayName,
  }) {
    final user = UserModel(
      id: id,
      email: email,
      displayName: displayName,
      age: 30,
      weightKg: 70,
      heightCm: 170,
      activityLevel: 'moderate',
      createdAt: DateTime.now(),
      goals: const UserGoals(
        dailySteps: 10000,
        dailyCalorieBurn: 500,
        weeklyWorkouts: 4,
        targetWeightKg: 68,
        primaryGoal: 'fitness',
      ),
    );
    _users[id] = user;
    _achievements[id] = _defaultAchievements();
    return user;
  }

  List<Achievement> _defaultAchievements() => [
        Achievement(
          title: 'First Steps',
          description: 'Complete your first walk',
          iconName: 'directions_walk',
          earned: false,
        ),
        Achievement(
          title: '5K Club',
          description: 'Walk 5 kilometers in one session',
          iconName: 'emoji_events',
          earned: false,
        ),
        Achievement(
          title: 'Week Warrior',
          description: 'Work out 7 days in a row',
          iconName: 'local_fire_department',
          earned: false,
        ),
      ];
}
