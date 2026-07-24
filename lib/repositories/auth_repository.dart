class AuthResult {
  final bool success;
  final String? userId;
  final String? email;
  final String? error;

  const AuthResult({
    required this.success,
    this.userId,
    this.email,
    this.error,
  });
}

/// Abstract auth contract — swap MockAuthRepository for FirebaseAuthRepository
/// once Firebase config files are provided.
abstract class AuthRepository {
  Future<AuthResult> signInWithEmail(String email, String password);
  Future<AuthResult> registerWithEmail(
      String email, String password, String displayName);
  Future<void> signOut();
  Future<void> sendPasswordReset(String email);
  Future<void> sendEmailVerification();
  bool get isAuthenticated;
  String? get currentUserId;
}

class MockAuthRepository implements AuthRepository {
  String? _currentUserId;

  @override
  bool get isAuthenticated => _currentUserId != null;

  @override
  String? get currentUserId => _currentUserId;

  @override
  Future<AuthResult> signInWithEmail(String email, String password) async {
    await Future<void>.delayed(const Duration(milliseconds: 800));
    if (password.length < 6) {
      return const AuthResult(
          success: false, error: 'Invalid email or password');
    }
    _currentUserId = 'user_${email.hashCode}';
    return AuthResult(success: true, userId: _currentUserId, email: email);
  }

  @override
  Future<AuthResult> registerWithEmail(
      String email, String password, String displayName) async {
    await Future<void>.delayed(const Duration(milliseconds: 800));
    if (password.length < 6) {
      return const AuthResult(
          success: false, error: 'Password must be at least 6 characters');
    }
    _currentUserId = 'user_${email.hashCode}';
    return AuthResult(success: true, userId: _currentUserId, email: email);
  }

  @override
  Future<void> signOut() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    _currentUserId = null;
  }

  @override
  Future<void> sendPasswordReset(String email) async {
    await Future<void>.delayed(const Duration(milliseconds: 500));
  }

  @override
  Future<void> sendEmailVerification() async {
    await Future<void>.delayed(const Duration(milliseconds: 500));
  }
}
