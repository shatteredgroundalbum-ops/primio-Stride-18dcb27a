import 'package:flutter/foundation.dart';
import '../models/profile_data.dart';
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

  Future<void> loadProfile() async {
    _isLoading = true;
    _error = null;
    notifyListeners();
    try {
      final results = await Future.wait([
        _service.getProfile(),
        _service.getBodyStats(),
        _service.getAchievements(),
      ]);
      _profile = results[0] as UserProfile;
      _bodyStats = results[1] as BodyStats;
      _achievements = results[2] as List<Achievement>;
    } catch (e) {
      _error = 'Failed to load profile data';
    }
    _isLoading = false;
    notifyListeners();
  }
}
