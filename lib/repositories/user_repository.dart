import '../models/user_model.dart';

/// In-memory user store — swap internals for Firestore calls
/// (e.g. FirebaseFirestore.instance.collection('users')) when ready.
class UserRepository {
  final Map<String, UserModel> _users = {};

  Future<UserModel?> getUser(String userId) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    return _users[userId];
  }

  Future<void> createUser(UserModel user) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    _users[user.id] = user;
  }

  Future<void> updateUser(UserModel user) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    _users[user.id] = user;
  }

  Future<void> deleteUser(String userId) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    _users.remove(userId);
  }

  UserModel createDefaultUser({
    required String id,
    required String email,
    required String displayName,
  }) =>
      UserModel(
        id: id,
        email: email,
        displayName: displayName,
        age: 30,
        weightKg: 75,
        heightCm: 175,
        activityLevel: 'moderate',
        createdAt: DateTime.now(),
        goals: const UserGoals(),
      );
}
