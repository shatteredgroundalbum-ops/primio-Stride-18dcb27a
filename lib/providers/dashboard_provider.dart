import 'package:flutter/foundation.dart';
import '../models/health_data.dart';
import '../services/health_service.dart';

class DashboardProvider extends ChangeNotifier {
  final HealthService _service;
  final String _userId;

  DashboardProvider({required HealthService service, required String userId})
      : _service = service,
        _userId = userId;

  DailyHealth? _data;
  DailyHealth? get data => _data;

  List<AiInsight> _insights = [];
  List<AiInsight> get insights => _insights;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  Future<void> loadDashboard() async {
    _isLoading = true;
    _error = null;
    notifyListeners();
    try {
      _data = await _service.getDailyHealth(userId: _userId);
      _insights = _service.getInsights(_data!);
    } catch (e) {
      _error = 'Failed to load health data';
    }
    _isLoading = false;
    notifyListeners();
  }
}
