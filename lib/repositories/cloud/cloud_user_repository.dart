import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

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

/// Real user repository backed by SharedPreferences for local
/// persistence. Each user is stored as a JSON document keyed by
/// user ID. When Firebase is connected, swap for
/// `FirestoreUserRepository` — the abstract interface is identical.
class LocalCloudUserRepository implements CloudUserRepository {
  static const _keyPrefix = 'cloud_user_';
  static const _keyAchievementsPrefix = 'cloud_user_achievements_';

  Future<SharedPreferences> get _prefs async =>
      await SharedPreferences.getInstance();

  @override
  Future<UserModel?> getUser(String userId) async {
    final prefs = await _prefs;
    final json = prefs.getString('$_keyPrefix$userId');
    if (json == null) return null;
    return UserModel.fromMap(jsonDecode(json) as Map<String, dynamic>);
  }

  @override
  Future<void> createUser(UserModel user) async {
    final prefs = await _prefs;
    await prefs.setString('$_keyPrefix${user.id}', jsonEncode(user.toMap()));
    // Initialize empty achievement list for the new user
    final achKey = '$_keyAchievementsPrefix${user.id}';
    if (prefs.getString(achKey) == null) {
      await prefs.setString(achKey, jsonEncode([]));
    }
  }

  @override
  Future<void> updateUser(UserModel user) async {
    final prefs = await _prefs;
    await prefs.setString('$_keyPrefix${user.id}', jsonEncode(user.toMap()));
  }

  @override
  Future<void> updateGoals(String userId, UserGoals goals) async {
    final user = await getUser(userId);
    if (user != null) {
      await updateUser(user.copyWith(goals: goals));
    }
  }

  @override
  Future<void> updateSettings(String userId, AppSettings settings) async {
    // Settings are managed by PreferencesService — no-op here.
  }

  @override
  Future<void> deleteUser(String userId) async {
    final prefs = await _prefs;
    await prefs.remove('$_keyPrefix$userId');
    await prefs.remove('$_keyAchievementsPrefix$userId');
  }

  @override
  Future<List<Achievement>> getAchievements(String userId) async {
    final prefs = await _prefs;
    final json = prefs.getString('$_keyAchievementsPrefix$userId');
    if (json == null) return [];
    final list = jsonDecode(json) as List<dynamic>;
    return list
        .map((a) => Achievement(
              title: a['title'] as String,
              description: a['description'] as String,
              iconName: a['iconName'] as String,
              earned: a['earned'] as bool? ?? false,
              earnedDate: a['earnedDate'] != null
                  ? DateTime.parse(a['earnedDate'] as String)
                  : null,
            ))
        .toList();
  }

  @override
  Future<void> awardAchievement(
      String userId, Achievement achievement) async {
    final current = await getAchievements(userId);
    current.add(achievement);
    final prefs = await _prefs;
    await prefs.setString(
      '$_keyAchievementsPrefix$userId',
      jsonEncode(current.map((a) => {
            'title': a.title;
            'description': a.description;
            'iconName': a.iconName;
            'earned': a.earned;
            'earnedDate': a.earnedDate?.toIso8601String();
          }).toList()),
    );
  }
}
