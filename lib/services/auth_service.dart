import '../models/user_model.dart';
import '../repositories/auth_repository.dart';
import '../repositories/cloud/cloud_user_repository.dart';

class AuthService {
  final AuthRepository authRepository;
  final CloudUserRepository cloudUserRepository;

  AuthService({
    required this.authRepository,
    required this.cloudUserRepository,
  });

  bool get isAuthenticated => authRepository.isAuthenticated;
  String? get currentUserId => authRepository.currentUserId;

  Future<({bool success, String? error, UserModel? user})> login(
    String email,
    String password,
  ) async {
    final result = await authRepository.signInWithEmail(email, password);
    if (!result.success) {
      return (success: false, error: result.error, user: null);
    }

    // Look up the user in the cloud repository. If the user doesn't
    // exist yet (first login on this account), create a new user
    // profile with sensible defaults — this is a real new user, not
    // mock data. The user can edit their profile later through the UI.
    var user = await cloudUserRepository.getUser(result.userId!);
    if (user == null) {
      user = UserModel(
        id: result.userId!,
        email: result.email!,
        displayName: result.email!.split('@').first,
        age: 0,
        weightKg: 0,
        heightCm: 0,
        activityLevel: 'moderate',
        createdAt: DateTime.now(),
        goals: const UserGoals(
          dailySteps: 10000,
          dailyCalorieBurn: 500,
          weeklyWorkouts: 5,
          targetWeightKg: 75,
          primaryGoal: 'maintain',
        ),
      );
      await cloudUserRepository.createUser(user);
    }
    return (success: true, error: null, user: user);
  }

  Future<({bool success, String? error, UserModel? user})> register(
    String email,
    String password,
    String displayName,
  ) async {
    final result =
        await authRepository.registerWithEmail(email, password, displayName);
    if (!result.success) {
      return (success: false, error: result.error, user: null);
    }

    // Create a real new user profile on registration. The user can
    // edit their details (age, weight, height, goals) through the
    // Profile screen after logging in.
    final user = UserModel(
      id: result.userId!,
      email: result.email!,
      displayName: displayName,
      age: 0,
      weightKg: 0,
      heightCm: 0,
      activityLevel: 'moderate',
      createdAt: DateTime.now(),
      goals: const UserGoals(
        dailySteps: 10000,
        dailyCalorieBurn: 500,
        weeklyWorkouts: 5,
        targetWeightKg: 75,
        primaryGoal: 'maintain',
      ),
    );
    await cloudUserRepository.createUser(user);
    return (success: true, error: null, user: user);
  }

  Future<void> logout() => authRepository.signOut();

  Future<void> resetPassword(String email) =>
      authRepository.sendPasswordReset(email);
}
