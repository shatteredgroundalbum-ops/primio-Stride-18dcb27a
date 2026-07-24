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

    var user = await cloudUserRepository.getUser(result.userId!);
    if (user == null && cloudUserRepository is MockCloudUserRepository) {
      user = (cloudUserRepository as MockCloudUserRepository)
          .createDefaultUser(
        id: result.userId!,
        email: result.email!,
        displayName: result.email!.split('@').first,
      );
    }
    if (user != null) {
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

    UserModel? user;
    if (cloudUserRepository is MockCloudUserRepository) {
      user = (cloudUserRepository as MockCloudUserRepository)
          .createDefaultUser(
        id: result.userId!,
        email: result.email!,
        displayName: displayName,
      );
    } else {
      user = UserModel(
        id: result.userId!,
        email: result.email!,
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
      await cloudUserRepository.createUser(user);
    }
    return (success: true, error: null, user: user);
  }

  Future<void> logout() => authRepository.signOut();

  Future<void> resetPassword(String email) =>
      authRepository.sendPasswordReset(email);
}
