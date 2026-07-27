import 'package:flutter/foundation.dart';
import '../models/profile_data.dart';
import '../models/user_model.dart';
import '../services/profile_service.dart';

class ProfileProvider extends ChangeNotifier {
  final ProfileService _service;

  ProfileProvider({required ProfileService service}) : _service = service;

  UserProfile? _profile;
  UserProfile? get profile => _profile;

  BodyStats? _bodyStats;
  BodyStats? get bodyStats => _bodyStats;

  List<Achievement> _achievements = [];
  List<Achievement> get achievements => _achievements;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  int get earnedCount => _achievements.where((a) => a.earned).length;

  /// Loads profile from the real authenticated user model. The user
  /// model comes from AuthProvider and is passed in from the screen.
  Future<void> loadProfile(UserModel? user) async {
    _isLoading = true;
    _error = null;
    notifyListeners();
    try {
      _profile = _service.profileFromUser(user);
      final results = await Future.wait([
        _service.getBodyStats(),
        _service.getAchievements(),
      ]);
      _bodyStats = results[0] as BodyStats?;
      _achievements = results[1] as List<Achievement>;
    } catch (e) {
      _error = 'Failed to load profile data';
    }
    _isLoading = false;
    notifyListeners();
  }
}
