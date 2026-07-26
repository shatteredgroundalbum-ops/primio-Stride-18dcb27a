import 'dart:async';

import 'package:flutter/foundation.dart' show kIsWeb;

// `unawaited` is re-exported from `dart:async` (Dart >= 2.15); imported
// explicitly here for clarity since it's used to intentionally
// fire-and-forget best-effort SQLite writes off the GPS/sensor hot path.

import '../database/local_database.dart';
import '../models/sync_status.dart';
import '../models/walking_session.dart';
import '../native/stride_engine_client.dart';
import '../native/stride_models.dart';
import 'gps_service.dart';

/// Live workout recording engine following the production data flow:
///
/// ```
/// GPS sensors
///      ↓
/// Rust stride_engine (via dart:ffi)   <-- distance/pace/speed/splits/
///      ↓                                  calories/elevation/auto-pause/
///      ↓                                  walk-vs-run classification all
///      ↓                                  computed natively, not in Dart
/// LocalDatabase (SQLite)               <-- crash-recovery checkpoints +
///      ↓                                  route points persisted from the
///      ↓                                  engine's live snapshot
/// Completed WorkoutSummary
///      ↓
/// Queue for sync → Firestore + Cloud Storage
/// ```
///
/// All of the actual workout math (distance, pace, speed, calories,
/// elevation, splits, auto-pause, walk/run classification, GPS filtering)
/// happens inside the native `stride_engine` Rust crate — this class is
/// now a thin coordinator: it forwards GPS/sensor samples into the engine,
/// persists what the engine reports, and exposes the result as the
/// existing [WalkingSession] model so the rest of the app (screens,
/// providers) doesn't need to change.
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

  StrideEngineClient? _engine;

  WalkingSession? _activeSession;
  WalkingSession? get activeSession => _activeSession;
  bool get isRecording => _activeSession != null;

  final List<RoutePoint> _currentPoints = [];
  List<RoutePoint> get currentPoints => List.unmodifiable(_currentPoints);

  StreamSubscription<RoutePoint>? _gpsSubscription;
  Timer? _tickTimer;

  final _sessionController = StreamController<WalkingSession>.broadcast();

  /// Emits the active session with updated stats every second.
  Stream<WalkingSession> get sessionStream => _sessionController.stream;

  /// Emits structured coaching events (auto-pause, splits, GPS quality,
  /// goal progress, etc.) as the native engine produces them — the UI can
  /// use this to drive voice coaching / toast notifications.
  final _coachingController = StreamController<StrideCoachingEvent>.broadcast();
  Stream<StrideCoachingEvent> get coachingEvents => _coachingController.stream;

  // ─── Start ──────────────────────────────────────────────────────────────

  /// Starts a new workout recording.
  Future<bool> startWorkout({
    required String userId,
    SessionType type = SessionType.walk,
    double weightKg = 70,
  }) async {
    if (kIsWeb || isRecording) return false;

    final hasPermission = await _gpsService.checkPermission();
    if (!hasPermission) return false;

    final now = DateTime.now();

    // Create the native workout session — this is the single source of
    // truth for every calculation from this point forward.
    final engine = StrideEngineClient.create(
      userId: userId,
      activityType: type,
      weightKg: weightKg,
      startedAt: now,
    );
    _engine = engine;
    engine.start();

    final session = WalkingSession(
      id: engine.workoutId,
      userId: userId,
      startTime: now,
      type: type,
      status: SessionStatus.active,
    );

    _activeSession = session;
    _currentPoints.clear();

    // Write to SQLite immediately
    await _localDb.startWorkout(session);

    // Start GPS stream → Rust engine → SQLite
    await _gpsService.startTracking();
    _gpsSubscription = _gpsService.pointStream.listen((point) {
      _currentPoints.add(point);
      _localDb.insertRoutePoint(session.id, point);
      _feedLocationSampleToEngine(point);
    });

    // Advance the engine's clock roughly once per second even if no new
    // GPS fix has arrived, and refresh the UI-facing session snapshot.
    _tickTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      _tickEngine();
    });

    return true;
  }

  void _feedLocationSampleToEngine(RoutePoint point) {
    final engine = _engine;
    if (engine == null) return;
    final now = DateTime.now();
    final update = engine.addLocationSample(
      latitude: point.latitude,
      longitude: point.longitude,
      altitudeMeters: point.altitude,
      accuracyMeters: null,
      speedMetersPerSecond: point.speed,
      recordedAt: point.timestamp,
      now: now,
    );
    _applyLiveUpdate(update);
    _persistCheckpoint(now);
  }

  void _tickEngine() {
    final engine = _engine;
    if (engine == null) return;
    final update = engine.tick(DateTime.now());
    _applyLiveUpdate(update);
  }

  void _applyLiveUpdate(StrideLiveUpdate update) {
    if (_activeSession == null) return;
    final s = update.session;

    _activeSession = _activeSession!.copyWith(
      distanceMeters: s.totalDistanceMeters,
      duration: Duration(milliseconds: s.activeMs),
      averagePaceMinPerKm: s.averagePaceSecPerKm / 60.0,
      caloriesBurned: s.caloriesEstimated,
      averageSpeedKmh: s.averageSpeedMps * 3.6,
      maxSpeedKmh: s.maxSpeedMps * 3.6,
      elevationGainMeters: s.elevationGainMeters,
      stepCount: s.stepCount,
      averageHeartRate: s.averageHeartRate,
      status: s.state == 'paused' ? SessionStatus.paused : SessionStatus.active,
    );

    _sessionController.add(_activeSession!);

    // Persist running totals to SQLite (crash recovery).
    _localDb.updateWorkoutStats(
      workoutId: _activeSession!.id,
      distanceMeters: s.totalDistanceMeters,
      durationSeconds: s.activeMs ~/ 1000,
      stepCount: s.stepCount,
      calories: s.caloriesEstimated,
    );

    for (final event in update.coachingEvents) {
      _coachingController.add(event);
    }
  }

  void _persistCheckpoint(DateTime now) {
    // The engine's checkpoint JSON is the durable crash-recovery state
    // consumed by `RecoveryService` at next app startup (via
    // `stride_evaluate_recovery` / `stride_restore_session`). It's an
    // upsert keyed by workout_id, so this is safe to call on every
    // accepted GPS sample without unbounded table growth.
    final engine = _engine;
    final session = _activeSession;
    if (engine == null || session == null) return;
    final checkpointJson = engine.buildCheckpoint(now);
    // Fire-and-forget: crash recovery is best-effort by nature, and we
    // don't want a slow disk write to block the GPS callback.
    unawaited(_localDb.saveCheckpoint(session.id, checkpointJson, now));
  }

  // ─── Crash Recovery ──────────────────────────────────────────────────

  /// Rebuilds a live native controller from a persisted checkpoint
  /// (produced by a prior process before it was killed/crashed) and
  /// attaches it to this recorder in the `Paused` state, without
  /// starting GPS tracking. This is the bridge between `RecoveryService`
  /// (which decides *whether* to recover) and the normal
  /// [resumeWorkout]/[stopWorkout]/[cancelWorkout] flows (which already
  /// know how to continue/finish/discard a paused session) — after
  /// calling this, the caller should invoke exactly one of those three
  /// depending on the user's/decision's chosen recovery option.
  ///
  /// Returns `false` (and does nothing) if a workout is already active
  /// on this recorder — recovery only makes sense once, at startup,
  /// before any new workout has been started.
  bool restoreFromCheckpoint({
    required Map<String, dynamic> checkpointJson,
    required WalkingSession sessionSeed,
    required String userId,
    required SessionType activityType,
    double weightKg = 70,
  }) {
    if (isRecording) return false;

    final engine = StrideEngineClient.restore(
      checkpointJson: checkpointJson,
      userId: userId,
      activityType: activityType,
      weightKg: weightKg,
    );
    _engine = engine;
    _activeSession = sessionSeed.copyWith(status: SessionStatus.paused);
    _currentPoints.clear();
    _sessionController.add(_activeSession!);
    return true;
  }

  // ─── Pause / Resume ─────────────────────────────────────────────────────

  Future<void> pauseWorkout() async {
    if (_activeSession == null ||
        _activeSession!.status != SessionStatus.active) return;

    final now = DateTime.now();
    _engine?.pause(now);

    _gpsService.stopTracking();
    _gpsSubscription?.cancel();
    _tickTimer?.cancel();

    _activeSession = _activeSession!.copyWith(status: SessionStatus.paused);
    await _localDb.insertPauseEvent(_activeSession!.id, 'pause');
    _sessionController.add(_activeSession!);
  }

  Future<void> resumeWorkout({double weightKg = 70}) async {
    if (_activeSession == null ||
        _activeSession!.status != SessionStatus.paused) return;

    _engine?.resume();

    await _localDb.insertPauseEvent(_activeSession!.id, 'resume');

    _activeSession = _activeSession!.copyWith(status: SessionStatus.active);

    await _gpsService.startTracking();
    _gpsSubscription = _gpsService.pointStream.listen((point) {
      _currentPoints.add(point);
      _localDb.insertRoutePoint(_activeSession!.id, point);
      _feedLocationSampleToEngine(point);
    });

    _tickTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      _tickEngine();
    });
  }

  // ─── Stop & Finalize ────────────────────────────────────────────────────

  /// Stops the workout, computes final stats via the native engine, and
  /// queues the result for cloud sync. Returns the finalized session, or
  /// null if nothing was active.
  Future<WalkingSession?> stopWorkout({double weightKg = 70}) async {
    if (_activeSession == null || _engine == null) return null;

    _gpsService.stopTracking();
    _gpsSubscription?.cancel();
    _tickTimer?.cancel();

    final now = DateTime.now();
    // The engine's state machine allows finishing directly from either
    // Active or Paused, so no forced pause is needed here.
    final summary = _engine!.finish(now);

    GeoLocation? startLoc;
    GeoLocation? endLoc;
    if (summary.startLatitude != null && summary.startLongitude != null) {
      startLoc = GeoLocation(
        latitude: summary.startLatitude!,
        longitude: summary.startLongitude!,
      );
    }
    if (summary.endLatitude != null && summary.endLongitude != null) {
      endLoc = GeoLocation(
        latitude: summary.endLatitude!,
        longitude: summary.endLongitude!,
      );
    }

    final finalized = _activeSession!.copyWith(
      endTime: now,
      distanceMeters: summary.distanceMeters,
      duration: Duration(milliseconds: summary.totalDurationMs),
      averagePaceMinPerKm: summary.averagePaceSecPerKm / 60.0,
      caloriesBurned: summary.caloriesEstimated,
      averageSpeedKmh: summary.averageSpeedMps * 3.6,
      maxSpeedKmh: summary.maxSpeedMps * 3.6,
      elevationGainMeters: summary.elevationGainMeters,
      stepCount: summary.stepCount,
      averageHeartRate: summary.averageHeartRateBpm?.round(),
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
    final routePoints = await _localDb.getRoutePoints(finalized.id);
    await _localDb.queueForUpload(finalized, routePoints);

    // Cache for offline history
    await _localDb.cacheSession(finalized);

    // Clean up active workout data from SQLite
    await _localDb.deleteRoutePoints(finalized.id);
    await _localDb.deleteStepSamples(finalized.id);
    await _localDb.deleteCheckpoint(finalized.id);
    await _localDb.deleteActiveWorkout(finalized.id);

    _sessionController.add(finalized);
    _engine!.dispose();
    _engine = null;
    _activeSession = null;
    _currentPoints.clear();

    return finalized;
  }

  /// Cancels the active workout without saving.
  Future<void> cancelWorkout() async {
    if (_activeSession == null) return;

    _gpsService.stopTracking();
    _gpsSubscription?.cancel();
    _tickTimer?.cancel();

    final id = _activeSession!.id;

    // discard() is only valid from the Paused state in the engine's state
    // machine, so pause first if the workout is still active.
    if (_engine != null) {
      try {
        if (_activeSession!.status == SessionStatus.active) {
          _engine!.pause(DateTime.now());
        }
        _engine!.discard();
      } catch (_) {
        // Best-effort: still clean up local state even if the native
        // state machine rejected the transition for some reason.
      }
      _engine!.dispose();
      _engine = null;
    }

    await _localDb.deleteRoutePoints(id);
    await _localDb.deleteStepSamples(id);
    await _localDb.deleteCheckpoint(id);
    await _localDb.deleteActiveWorkout(id);

    _activeSession = null;
    _currentPoints.clear();
  }

  /// Records a manual lap/split button press.
  StrideWorkoutSplit? recordManualLap() {
    if (_engine == null) return null;
    return _engine!.manualLap(DateTime.now());
  }

  /// Feeds a heart-rate sample (from a wearable / Health Connect / manual
  /// entry) into the native engine. `source` is accepted for API symmetry
  /// with [recordStepDelta] and future source-priority handling, even
  /// though the current engine HR path doesn't yet branch on it.
  void recordHeartRate(int bpm, {String source = 'wear_os'}) {
    _engine?.addHeartRateSample(at: DateTime.now(), bpm: bpm);
  }

  /// Feeds a step-count delta (from the phone's step sensor or a
  /// wearable) into the native engine, and persists the raw delta to
  /// SQLite so step history survives an app kill even before the
  /// workout finishes.
  void recordStepDelta(int delta, {String source = 'phone_step_sensor'}) {
    final session = _activeSession;
    if (session == null) return;
    final now = DateTime.now();
    _engine?.addStepDelta(at: now, delta: delta, source: source);
    unawaited(_localDb.insertStepSample(session.id, delta, source, now));
  }

  /// Disposes streams and timers.
  void dispose() {
    _gpsSubscription?.cancel();
    _tickTimer?.cancel();
    _sessionController.close();
    _coachingController.close();
    _engine?.dispose();
    _engine = null;
  }
}
