import 'dart:async';

import 'package:flutter/foundation.dart';

import '../database/local_database.dart';
import '../models/walking_session.dart';
import '../services/workout_recorder.dart';

/// Provides real walking/running/hiking session data to the Walking
/// screen. Reads completed sessions from the local SQLite cache
/// (`LocalDatabase.getCachedSessions`) and the live in-progress session
/// from `WorkoutRecorder` via its `sessionStream`.
///
/// No mock or hardcoded data — every value comes from real recorded
/// sessions. When the user has no sessions yet, `sessions` is empty and
/// the screen renders its empty state.
class WalkingProvider extends ChangeNotifier {
  final LocalDatabase _localDb;
  final WorkoutRecorder _recorder;
  final String _userId;

  WalkingProvider({
    required LocalDatabase localDb,
    required WorkoutRecorder recorder,
    required String userId,
  })  : _localDb = localDb,
        _recorder = recorder,
        _userId = userId;

  List<WalkingSession> _sessions = [];
  List<WalkingSession> get sessions => _sessions;

  /// Sessions from the cache filtered to today only.
  List<WalkingSession> get todaySessions {
    final now = DateTime.now();
    final today = DateTime(now.year, now.month, now.day);
    return _sessions
        .where((s) =>
            DateTime(s.startTime.year, s.startTime.month, s.startTime.day)
                .isAtSameMomentAs(today))
        .toList();
  }

  /// The live in-progress session (if any), straight from the recorder.
  WalkingSession? get activeSession => _recorder.activeSession;
  bool get isRecording => _recorder.isRecording;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  StreamSubscription<WalkingSession>? _sessionSub;

  Future<void> loadSessions() async {
    _isLoading = true;
    _error = null;
    notifyListeners();
    try {
      _sessions = await _localDb.getCachedSessions(_userId);
    } catch (e) {
      _error = 'Failed to load sessions';
    }
    _isLoading = false;
    notifyListeners();
  }

  /// Subscribes to the recorder's live session stream so the UI
  /// updates in real time while a workout is in progress.
  void subscribeToLiveSession() {
    _sessionSub?.cancel();
    _sessionSub = _recorder.sessionStream.listen((_) => notifyListeners());
  }

  // ── Live workout controls (delegated to WorkoutRecorder) ───────────

  Future<bool> startWorkout({SessionType type = SessionType.walk}) async {
    final ok = await _recorder.startWorkout(userId: _userId, type: type);
    if (ok) notifyListeners();
    return ok;
  }

  Future<void> pauseWorkout() async {
    await _recorder.pauseWorkout();
    notifyListeners();
  }

  Future<void> resumeWorkout() async {
    await _recorder.resumeWorkout();
    notifyListeners();
  }

  Future<WalkingSession?> stopWorkout() async {
    final finalized = await _recorder.stopWorkout();
    await loadSessions();
    return finalized;
  }

  Future<void> cancelWorkout() async {
    await _recorder.cancelWorkout();
    notifyListeners();
  }

  /// Aggregate stats for the summary card. All values are derived
  /// from real cached sessions; zero when no sessions exist.
  double get totalDistanceKm =>
      _sessions.fold(0.0, (sum, s) => sum + s.distanceKm);

  Duration get totalDuration => _sessions.fold(
      Duration.zero, (sum, s) => sum + s.duration);

  int get totalSteps =>
      _sessions.fold(0, (sum, s) => sum + s.stepCount);

  double get totalCalories =>
      _sessions.fold(0.0, (sum, s) => sum + s.caloriesBurned);

  /// This week's distance (last 7 days) for the weekly progress ring.
  double get thisWeekDistanceKm {
    final now = DateTime.now();
    final weekAgo = now.subtract(const Duration(days: 7));
    return _sessions
        .where((s) => s.startTime.isAfter(weekAgo))
        .fold(0.0, (sum, s) => sum + s.distanceKm);
  }

  @override
  void dispose() {
    _sessionSub?.cancel();
    super.dispose();
  }
}
