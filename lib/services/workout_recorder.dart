import 'dart:async';

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:uuid/uuid.dart';

import '../database/local_database.dart';
import '../models/sync_status.dart';
import '../models/walking_session.dart';
import 'gps_service.dart';

/// Live workout recording engine following the production data flow:
///
/// ```
/// GPS sensors
///      ↓
/// LocalDatabase (SQLite)
///      ↓
/// Workout calculations
///      ↓
/// Completed session summary
///      ↓
/// Queue for sync → Firestore + Cloud Storage
/// ```
///
/// The live workout never depends on a network connection.
class WorkoutRecorder {
  final LocalDatabase _localDb;
  final GpsService _gpsService;

  WorkoutRecorder({
    required LocalDatabase localDb,
    required GpsService gpsService,
  })  : _localDb = localDb,
        _gpsService = gpsService;

  WalkingSession? _activeSession;
  WalkingSession? get activeSession => _activeSession;
  bool get isRecording => _activeSession != null;

  final List<RoutePoint> _currentPoints = [];
  List<RoutePoint> get currentPoints => List.unmodifiable(_currentPoints);

  StreamSubscription<RoutePoint>? _gpsSubscription;
  Timer? _statsTimer;
  DateTime? _lastPauseTime;
  Duration _pausedDuration = Duration.zero;

  final _sessionController =
      StreamController<WalkingSession>.broadcast();

  /// Emits the active session with updated stats every second.
  Stream<WalkingSession> get sessionStream => _sessionController.stream;

  // ─── Start ─────────────────────────────────────────────────────

  /// Starts a new workout recording.
  Future<bool> startWorkout({
    required String userId,
    SessionType type = SessionType.walk,
    double weightKg = 70,
  }) async {
    if (kIsWeb || isRecording) return false;

    final hasPermission = await _gpsService.checkPermission();
    if (!hasPermission) return false;

    final session = WalkingSession(
      id: const Uuid().v4(),
      userId: userId,
      startTime: DateTime.now(),
      type: type,
      status: SessionStatus.active,
    );

    _activeSession = session;
    _currentPoints.clear();
    _pausedDuration = Duration.zero;
    _lastPauseTime = null;

    // Write to SQLite immediately
    await _localDb.startWorkout(session);

    // Start GPS stream → SQLite
    await _gpsService.startTracking();
    _gpsSubscription = _gpsService.pointStream.listen((point) {
      _currentPoints.add(point);
      _localDb.insertRoutePoint(session.id, point);
    });

    // Update stats every second
    _statsTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      _recalculateStats(weightKg);
    });

    return true;
  }

  // ─── Pause / Resume ────────────────────────────────────────────

  Future<void> pauseWorkout() async {
    if (_activeSession == null ||
        _activeSession!.status != SessionStatus.active) return;

    _lastPauseTime = DateTime.now();
    _gpsService.stopTracking();
    _gpsSubscription?.cancel();
    _statsTimer?.cancel();

    _activeSession = _activeSession!.copyWith(status: SessionStatus.paused);
    await _localDb.insertPauseEvent(_activeSession!.id, 'pause');
    _sessionController.add(_activeSession!);
  }

  Future<void> resumeWorkout({double weightKg = 70}) async {
    if (_activeSession == null ||
        _activeSession!.status != SessionStatus.paused) return;

    if (_lastPauseTime != null) {
      _pausedDuration += DateTime.now().difference(_lastPauseTime!);
      _lastPauseTime = null;
    }

    await _localDb.insertPauseEvent(_activeSession!.id, 'resume');

    _activeSession = _activeSession!.copyWith(status: SessionStatus.active);

    await _gpsService.startTracking();
    _gpsSubscription = _gpsService.pointStream.listen((point) {
      _currentPoints.add(point);
      _localDb.insertRoutePoint(_activeSession!.id, point);
    });

    _statsTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      _recalculateStats(weightKg);
    });
  }

  // ─── Stop & Finalize ───────────────────────────────────────────

  /// Stops the workout, computes final stats, queues for cloud sync.
  /// Returns the finalized session, or null if nothing was active.
  Future<WalkingSession?> stopWorkout({double weightKg = 70}) async {
    if (_activeSession == null) return null;

    _gpsService.stopTracking();
    _gpsSubscription?.cancel();
    _statsTimer?.cancel();

    // Compute final stats
    final now = DateTime.now();
    final elapsed = now.difference(_activeSession!.startTime) - _pausedDuration;
    final distanceM = GpsService.totalDistance(_currentPoints);
    final speedKmh =
        GpsService.averageSpeed(distanceM, elapsed);
    final paceMinKm =
        GpsService.averagePace(distanceM, elapsed);
    final calories = GpsService.estimateCalories(
      weightKg: weightKg,
      duration: elapsed,
      speedKmh: speedKmh,
    );
    final elevation = GpsService.elevationGain(_currentPoints);

    GeoLocation? startLoc;
    GeoLocation? endLoc;
    if (_currentPoints.isNotEmpty) {
      startLoc = GeoLocation(
        latitude: _currentPoints.first.latitude,
        longitude: _currentPoints.first.longitude,
      );
      endLoc = GeoLocation(
        latitude: _currentPoints.last.latitude,
        longitude: _currentPoints.last.longitude,
      );
    }

    final finalized = _activeSession!.copyWith(
      endTime: now,
      distanceMeters: distanceM,
      duration: elapsed,
      averagePaceMinPerKm: paceMinKm,
      caloriesBurned: calories,
      averageSpeedKmh: speedKmh,
      elevationGainMeters: elevation,
      status: SessionStatus.completed,
      startLocation: startLoc,
      endLocation: endLoc,
      syncMetadata: SyncMetadata(
        state: SyncState.pending,
        lastAttempt: now,
      ),
    );

    // Mark completed in SQLite
    await _localDb.completeWorkout(finalized.id);

    // Queue the finished workout + route points for cloud upload
    final routePoints =
        await _localDb.getRoutePoints(finalized.id);
    await _localDb.queueForUpload(finalized, routePoints);

    // Cache for offline history
    await _localDb.cacheSession(finalized);

    // Clean up active workout data from SQLite
    await _localDb.deleteRoutePoints(finalized.id);
    await _localDb.deleteActiveWorkout(finalized.id);

    _sessionController.add(finalized);
    _activeSession = null;
    _currentPoints.clear();

    return finalized;
  }

  /// Cancels the active workout without saving.
  Future<void> cancelWorkout() async {
    if (_activeSession == null) return;

    _gpsService.stopTracking();
    _gpsSubscription?.cancel();
    _statsTimer?.cancel();

    final id = _activeSession!.id;
    await _localDb.deleteRoutePoints(id);
    await _localDb.deleteActiveWorkout(id);

    _activeSession = null;
    _currentPoints.clear();
  }

  // ─── Stats Recalculation ───────────────────────────────────────

  void _recalculateStats(double weightKg) {
    if (_activeSession == null) return;

    final now = DateTime.now();
    final elapsed =
        now.difference(_activeSession!.startTime) - _pausedDuration;
    final distanceM = GpsService.totalDistance(_currentPoints);
    final speedKmh = GpsService.averageSpeed(distanceM, elapsed);
    final paceMinKm = GpsService.averagePace(distanceM, elapsed);
    final calories = GpsService.estimateCalories(
      weightKg: weightKg,
      duration: elapsed,
      speedKmh: speedKmh,
    );

    _activeSession = _activeSession!.copyWith(
      distanceMeters: distanceM,
      duration: elapsed,
      averagePaceMinPerKm: paceMinKm,
      caloriesBurned: calories,
      averageSpeedKmh: speedKmh,
    );

    // Persist running totals to SQLite (crash recovery)
    _localDb.updateWorkoutStats(
      workoutId: _activeSession!.id,
      distanceMeters: distanceM,
      durationSeconds: elapsed.inSeconds,
      stepCount: _activeSession!.stepCount,
      calories: calories,
    );

    _sessionController.add(_activeSession!);
  }

  /// Disposes streams and timers.
  void dispose() {
    _gpsSubscription?.cancel();
    _statsTimer?.cancel();
    _sessionController.close();
  }
}
