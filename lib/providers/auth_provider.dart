import 'package:flutter/material.dart';

import '../models/user_model.dart';
import '../services/auth_service.dart';

class AuthProvider extends ChangeNotifier {
  final AuthService _authService;

  AuthProvider({required AuthService authService})
      : _authService = authService;

  UserModel? _currentUser;
  UserModel? get currentUser => _currentUser;

  bool get isAuthenticated => _currentUser != null;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  Future<bool> login(String email, String password) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    final result = await _authService.login(email, password);
    _isLoading = false;

    if (result.success) {
      _currentUser = result.user;
    } else {
      _error = result.error;
    }
    notifyListeners();
    return result.success;
  }

  Future<bool> register(
      String email, String password, String displayName) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    final result = await _authService.register(email, password, displayName);
    _isLoading = false;

    if (result.success) {
      _currentUser = result.user;
    } else {
      _error = result.error;
    }
    notifyListeners();
    return result.success;
  }

  Future<void> logout() async {
    await _authService.logout();
    _currentUser = null;
    notifyListeners();
  }

  Future<bool> resetPassword(String email) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      await _authService.resetPassword(email);
      _isLoading = false;
      notifyListeners();
      return true;
    } catch (_) {
      _error = 'Failed to send reset email';
      _isLoading = false;
      notifyListeners();
      return false;
    }
  }

  void clearError() {
    _error = null;
    notifyListeners();
  }
}
