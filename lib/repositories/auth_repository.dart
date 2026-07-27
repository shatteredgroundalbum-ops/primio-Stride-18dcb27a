import 'package:shared_preferences/shared_preferences.dart';

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

/// Abstract auth contract — implementations must persist auth state
/// across app restarts so the user stays logged in.
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

/// Real auth repository using SharedPreferences for local persistence.
///
/// This stores the current session (userId + email) on device so the
/// user stays logged in across app restarts. When Firebase
/// configuration is added, this can be swapped for
/// `FirebaseAuthRepository` without changing any call sites.
class LocalAuthRepository implements AuthRepository {
  static const _keyUserId = 'auth_user_id';
  static const _keyEmail = 'auth_email';
  static const _keyStoredEmail = 'auth_stored_email';
  static const _keyStoredPassword = 'auth_stored_password';

  SharedPreferences? _prefs;

  Future<SharedPreferences> get _instance async {
    _prefs ??= await SharedPreferences.getInstance();
    return _prefs!;
  }

  @override
  bool get isAuthenticated => currentUserId != null;

  @override
  String? get currentUserId => _prefs?.getString(_keyUserId);

  String? get currentEmail => _prefs?.getString(_keyEmail);

  @override
  Future<AuthResult> signInWithEmail(String email, String password) async {
    final prefs = await _instance;

    // Check if there's a registered account for this email
    final storedEmail = prefs.getString(_keyStoredEmail);
    final storedPassword = prefs.getString(_keyStoredPassword);

    if (storedEmail == null || storedEmail != email) {
      return const AuthResult(
          success: false, error: 'No account found for this email');
    }

    if (storedPassword != password) {
      return const AuthResult(success: false, error: 'Incorrect password');
    }

    // Restore the existing user ID
    final userId = prefs.getString(_keyUserId) ?? 'user_${email.hashCode}';

    await prefs.setString(_keyUserId, userId);
    await prefs.setString(_keyEmail, email);
    _prefs = prefs;

    return AuthResult(success: true, userId: userId, email: email);
  }

  @override
  Future<AuthResult> registerWithEmail(
      String email, String password, String displayName) async {
    final prefs = await _instance;

    if (password.length < 6) {
      return const AuthResult(
          success: false, error: 'Password must be at least 6 characters');
    }

    final userId = 'user_${email.hashCode}';

    // Store the credentials for future sign-in attempts
    await prefs.setString(_keyStoredEmail, email);
    await prefs.setString(_keyStoredPassword, password);
    await prefs.setString(_keyUserId, userId);
    await prefs.setString(_keyEmail, email);
    _prefs = prefs;

    return AuthResult(success: true, userId: userId, email: email);
  }

  @override
  Future<void> signOut() async {
    final prefs = await _instance;
    await prefs.remove(_keyUserId);
    await prefs.remove(_keyEmail);
    _prefs = prefs;
  }

  @override
  Future<void> sendPasswordReset(String email) async {
    // No-op for local auth — Firebase will handle this when connected.
  }

  @override
  Future<void> sendEmailVerification() async {
    // No-op for local auth — Firebase will handle this when connected.
  }
}
