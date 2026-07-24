import '../models/user_model.dart';
import '../repositories/auth_repository.dart';
import '../repositories/user_repository.dart';

class AuthService {
  final AuthRepository authRepository;
  final UserRepository userRepository;

  AuthService({required this.authRepository, required this.userRepository});

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

    var user = await userRepository.getUser(result.userId!);
    user ??= userRepository.createDefaultUser(
      id: result.userId!,
      email: result.email!,
      displayName: result.email!.split('@').first,
    );
    await userRepository.createUser(user);
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

    final user = userRepository.createDefaultUser(
      id: result.userId!,
      email: result.email!,
      displayName: displayName,
    );
    await userRepository.createUser(user);
    return (success: true, error: null, user: user);
  }

  Future<void> logout() => authRepository.signOut();

  Future<void> resetPassword(String email) =>
      authRepository.sendPasswordReset(email);
}
