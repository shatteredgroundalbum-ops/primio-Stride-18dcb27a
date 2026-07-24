import 'dart:async';
import 'package:flutter/foundation.dart';
import '../models/fasting_data.dart';
import '../services/fasting_service.dart';

class FastingProvider extends ChangeNotifier {
  final FastingService _service;
  Timer? _timer;

  FastingProvider({required FastingService service}) : _service = service;

  FastingLog? _activeFast;
  FastingLog? get activeFast => _activeFast;

  List<FastingLog> _recentLogs = [];
  List<FastingLog> get recentLogs => _recentLogs;

  FastingStats? _stats;
  FastingStats? get stats => _stats;

  int _selectedProtocolIndex = 0;
  int get selectedProtocolIndex => _selectedProtocolIndex;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  Duration get elapsed {
    if (_activeFast == null) return Duration.zero;
    return DateTime.now().difference(_activeFast!.startTime);
  }

  Duration get remaining {
    if (_activeFast == null) return Duration.zero;
    final r = _activeFast!.targetDuration - elapsed;
    return r.isNegative ? Duration.zero : r;
  }

  double get progress {
    if (_activeFast == null) return 0.0;
    return (_activeFast!.targetDuration.inSeconds > 0)
        ? (elapsed.inSeconds / _activeFast!.targetDuration.inSeconds).clamp(0.0, 1.0)
        : 0.0;
  }

  Future<void> loadFastingData() async {
    _isLoading = true;
    _error = null;
    notifyListeners();
    try {
      final results = await Future.wait([
        _service.getActiveFast(),
        _service.getRecentLogs(),
        _service.getStats(),
      ]);
      _activeFast = results[0] as FastingLog?;
      _recentLogs = results[1] as List<FastingLog>;
      _stats = results[2] as FastingStats;
      if (_activeFast != null) _startTimer();
    } catch (e) {
      _error = 'Failed to load fasting data';
    }
    _isLoading = false;
    notifyListeners();
  }

  void selectProtocol(int index) {
    _selectedProtocolIndex = index;
    notifyListeners();
  }

  void startFast() {
    final protocol = FastingService.protocols[_selectedProtocolIndex];
    _activeFast = FastingLog(
      date: DateTime.now(),
      protocol: protocol,
      startTime: DateTime.now(),
      completed: false,
      targetDuration: Duration(hours: protocol.fastHours),
    );
    _startTimer();
    notifyListeners();
  }

  void endFast() {
    _timer?.cancel();
    _timer = null;
    _activeFast = null;
    notifyListeners();
  }

  void _startTimer() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(seconds: 30), (_) {
      notifyListeners();
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }
}
