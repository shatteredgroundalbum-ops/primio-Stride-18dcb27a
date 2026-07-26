import 'dart:convert';

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:path/path.dart' as p;
import 'package:sqflite/sqflite.dart';

import '../models/sync_status.dart';
import '../models/walking_session.dart';

/// Local SQLite database for offline-first workout recording.
///
/// Production data flow:
/// ```
/// Sensors / GPS  →  LocalDatabase (SQLite)
///                         ↓
///                 Workout calculations
///                         ↓
///                 Completed summary
///                         ↓
///              Firestore + Cloud Storage
/// ```
///
/// The live workout never depends on a network connection.
class LocalDatabase {
  static const _dbName = 'stride_local.db';
  // v1: initial schema (active_workout, route_points, heart_rate_samples,
  //     pause_events, pending_uploads, cached_plans, cached_sessions)
  // v2: adds step_samples, workout_checkpoints, pending_deletions —
  //     required for full offline-first crash recovery + step persistence
  //     + tombstone-based deletion sync (spec items 2 & 23).
  static const _dbVersion = 2;

  Database? _db;

  /// Returns the database, creating it if needed.
  /// Returns null on web (SQLite is mobile-only).
  Future<Database?> get database async {
    if (kIsWeb) return null;
    _db ??= await _initDatabase();
    return _db;
  }

  Future<Database> _initDatabase() async {
    final dbPath = await getDatabasesPath();
    final path = p.join(dbPath, _dbName);
    return openDatabase(
      path,
      version: _dbVersion,
      onCreate: _onCreate,
      onUpgrade: _onUpgrade,
    );
  }

  /// Versioned schema migration path. Each `if (oldVersion < N)` block is
  /// additive-only and safe to run on a database that's already been
  /// migrated up to some intermediate version — never drop/rewrite tables
  /// containing user data here.
  Future<void> _onUpgrade(Database db, int oldVersion, int newVersion) async {
    if (oldVersion < 2) {
      await _createV2Tables(db);
    }
  }

  Future<void> _onCreate(Database db, int version) async {
    // Active workout state — at most one row at a time
    await db.execute('''
      CREATE TABLE active_workout (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        activity_type TEXT NOT NULL,
        started_at TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'active',
        distance_meters REAL NOT NULL DEFAULT 0,
        duration_seconds INTEGER NOT NULL DEFAULT 0,
        step_count INTEGER NOT NULL DEFAULT 0,
        calories_estimated REAL NOT NULL DEFAULT 0
      )
    ''');

    // Incoming GPS samples — written every second during a workout
    await db.execute('''
      CREATE TABLE route_points (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        workout_id TEXT NOT NULL,
        latitude REAL NOT NULL,
        longitude REAL NOT NULL,
        altitude REAL,
        speed REAL,
        timestamp TEXT NOT NULL,
        FOREIGN KEY (workout_id) REFERENCES active_workout(id)
      )
    ''');

    // Heart-rate samples from wearables
    await db.execute('''
      CREATE TABLE heart_rate_samples (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        workout_id TEXT NOT NULL,
        bpm INTEGER NOT NULL,
        timestamp TEXT NOT NULL,
        FOREIGN KEY (workout_id) REFERENCES active_workout(id)
      )
    ''');

    // Pause/resume events for accurate duration calculation
    await db.execute('''
      CREATE TABLE pause_events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        workout_id TEXT NOT NULL,
        event_type TEXT NOT NULL,
        timestamp TEXT NOT NULL,
        FOREIGN KEY (workout_id) REFERENCES active_workout(id)
      )
    ''');

    // Completed workouts waiting to be synced to Firestore + Cloud Storage
    await db.execute('''
      CREATE TABLE pending_uploads (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        workout_json TEXT NOT NULL,
        route_points_json TEXT NOT NULL,
        sync_state TEXT NOT NULL DEFAULT 'pending',
        retry_count INTEGER NOT NULL DEFAULT 0,
        last_attempt TEXT,
        error_message TEXT,
        created_at TEXT NOT NULL
      )
    ''');

    // Cached training plans for offline access
    await db.execute('''
      CREATE TABLE cached_plans (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        plan_json TEXT NOT NULL,
        cached_at TEXT NOT NULL
      )
    ''');

    // Cached completed sessions for offline history
    await db.execute('''
      CREATE TABLE cached_sessions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        session_json TEXT NOT NULL,
        cached_at TEXT NOT NULL
      )
    ''');

    // Indexes for common queries
    await db.execute(
        'CREATE INDEX idx_route_points_workout ON route_points(workout_id)');
    await db.execute(
        'CREATE INDEX idx_hr_workout ON heart_rate_samples(workout_id)');
    await db.execute(
        'CREATE INDEX idx_pending_state ON pending_uploads(sync_state)');
    await db.execute(
        'CREATE INDEX idx_cached_sessions_user ON cached_sessions(user_id)');

    // Fresh installs get the full current schema directly (no need to
    // route through onUpgrade).
    await _createV2Tables(db);
  }

  /// Tables introduced in schema version 2. Split into its own method so
  /// both a fresh `_onCreate` and an `_onUpgrade` from v1 can call it.
  Future<void> _createV2Tables(Database db) async {
    // Pedometer/step-sensor deltas — written every time a step-count
    // delta is fed into the native engine, so step history survives an
    // app kill even before the workout finishes.
    await db.execute('''
      CREATE TABLE IF NOT EXISTS step_samples (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        workout_id TEXT NOT NULL,
        step_delta INTEGER NOT NULL,
        source TEXT NOT NULL,
        timestamp TEXT NOT NULL,
        FOREIGN KEY (workout_id) REFERENCES active_workout(id)
      )
    ''');

    // Durable crash-recovery snapshots produced by
    // `WorkoutSessionController::build_checkpoint` / the
    // `stride_build_checkpoint` FFI call. Only the latest row per
    // workout matters; older ones are pruned as new ones arrive.
    await db.execute('''
      CREATE TABLE IF NOT EXISTS workout_checkpoints (
        workout_id TEXT PRIMARY KEY,
        checkpoint_json TEXT NOT NULL,
        checkpoint_at TEXT NOT NULL,
        FOREIGN KEY (workout_id) REFERENCES active_workout(id)
      )
    ''');

    // Tombstone queue for deletions that happened while offline (or
    // whose delete-on-server call failed), so a workout deleted locally
    // doesn't silently reappear from a stale cloud copy on next sync.
    await db.execute('''
      CREATE TABLE IF NOT EXISTS pending_deletions (
        workout_id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        deleted_at TEXT NOT NULL,
        synced INTEGER NOT NULL DEFAULT 0
      )
    ''');

    await db.execute(
        'CREATE INDEX IF NOT EXISTS idx_step_samples_workout ON step_samples(workout_id)');
    await db.execute(
        'CREATE INDEX IF NOT EXISTS idx_pending_deletions_synced ON pending_deletions(synced)');
  }

  // ─── Active Workout ─────────────────────────────────────────────

  /// Starts a new workout by inserting into active_workout.
  Future<void> startWorkout(WalkingSession session) async {
    final db = await database;
    if (db == null) return;
    await db.insert('active_workout', {
      'id': session.id,
      'user_id': session.userId,
      'activity_type': session.type.name,
      'started_at': session.startTime.toIso8601String(),
      'status': session.status.name,
    });
  }

  /// Updates the running totals of the active workout.
  Future<void> updateWorkoutStats({
    required String workoutId,
    required double distanceMeters,
    required int durationSeconds,
    required int stepCount,
    required double calories,
  }) async {
    final db = await database;
    if (db == null) return;
    await db.update(
      'active_workout',
      {
        'distance_meters': distanceMeters,
        'duration_seconds': durationSeconds,
        'step_count': stepCount,
        'calories_estimated': calories,
      },
      where: 'id = ?',
      whereArgs: [workoutId],
    );
  }

  /// Returns the active workout row, or null if none is active.
  Future<Map<String, dynamic>?> getActiveWorkout() async {
    final db = await database;
    if (db == null) return null;
    final rows = await db.query('active_workout',
        where: 'status = ?', whereArgs: ['active'], limit: 1);
    return rows.isEmpty ? null : rows.first;
  }

  /// Ends the active workout — marks it completed.
  Future<void> completeWorkout(String workoutId) async {
    final db = await database;
    if (db == null) return;
    await db.update(
      'active_workout',
      {'status': 'completed'},
      where: 'id = ?',
      whereArgs: [workoutId],
    );
  }

  /// Removes the active workout after it has been finalized.
  Future<void> deleteActiveWorkout(String workoutId) async {
    final db = await database;
    if (db == null) return;
    await db
        .delete('active_workout', where: 'id = ?', whereArgs: [workoutId]);
  }

  // ─── Route Points ───────────────────────────────────────────────

  /// Inserts a GPS sample from the position stream.
  Future<void> insertRoutePoint(String workoutId, RoutePoint point) async {
    final db = await database;
    if (db == null) return;
    await db.insert('route_points', {
      'workout_id': workoutId,
      'latitude': point.latitude,
      'longitude': point.longitude,
      'altitude': point.altitude,
      'speed': point.speed,
      'timestamp': point.timestamp.toIso8601String(),
    });
  }

  /// Returns all GPS points for a workout, ordered by time.
  Future<List<RoutePoint>> getRoutePoints(String workoutId) async {
    final db = await database;
    if (db == null) return [];
    final rows = await db.query('route_points',
        where: 'workout_id = ?',
        whereArgs: [workoutId],
        orderBy: 'timestamp ASC');
    return rows.map((r) => RoutePoint.fromMap({
          'latitude': r['latitude'],
          'longitude': r['longitude'],
          'altitude': r['altitude'],
          'speed': r['speed'],
          'timestamp': r['timestamp'],
        })).toList();
  }

  /// Deletes all route points for a workout after compression/upload.
  Future<void> deleteRoutePoints(String workoutId) async {
    final db = await database;
    if (db == null) return;
    await db.delete('route_points',
        where: 'workout_id = ?', whereArgs: [workoutId]);
  }

  // ─── Pause Events ──────────────────────────────────────────────

  Future<void> insertPauseEvent(
      String workoutId, String eventType) async {
    final db = await database;
    if (db == null) return;
    await db.insert('pause_events', {
      'workout_id': workoutId,
      'event_type': eventType,
      'timestamp': DateTime.now().toIso8601String(),
    });
  }

  Future<List<Map<String, dynamic>>> getPauseEvents(
      String workoutId) async {
    final db = await database;
    if (db == null) return [];
    return db.query('pause_events',
        where: 'workout_id = ?',
        whereArgs: [workoutId],
        orderBy: 'timestamp ASC');
  }

  // ─── Step Samples ────────────────────────────────────────────────

  /// Persists one step-count delta (mirrors `stride_add_step_delta`'s
  /// input) so raw step history survives an app kill before the workout
  /// finishes and gets rolled up into a final summary.
  Future<void> insertStepSample(
    String workoutId,
    int stepDelta,
    String source,
    DateTime timestamp,
  ) async {
    final db = await database;
    if (db == null) return;
    await db.insert('step_samples', {
      'workout_id': workoutId,
      'step_delta': stepDelta,
      'source': source,
      'timestamp': timestamp.toIso8601String(),
    });
  }

  /// Returns all step samples for a workout, ordered by time.
  Future<List<Map<String, dynamic>>> getStepSamples(String workoutId) async {
    final db = await database;
    if (db == null) return [];
    return db.query('step_samples',
        where: 'workout_id = ?',
        whereArgs: [workoutId],
        orderBy: 'timestamp ASC');
  }

  /// Deletes all step samples for a workout after it has been
  /// finalized/uploaded.
  Future<void> deleteStepSamples(String workoutId) async {
    final db = await database;
    if (db == null) return;
    await db.delete('step_samples',
        where: 'workout_id = ?', whereArgs: [workoutId]);
  }

  // ─── Workout Checkpoints (crash recovery) ───────────────────────

  /// Upserts the latest crash-recovery checkpoint for a workout. Called
  /// on every periodic checkpoint tick from `WorkoutRecorder` — replaces
  /// any prior checkpoint for the same `workout_id` since only the most
  /// recent snapshot is ever useful for recovery.
  Future<void> saveCheckpoint(
    String workoutId,
    Map<String, dynamic> checkpointJson,
    DateTime checkpointAt,
  ) async {
    final db = await database;
    if (db == null) return;
    await db.insert(
      'workout_checkpoints',
      {
        'workout_id': workoutId,
        'checkpoint_json': jsonEncode(checkpointJson),
        'checkpoint_at': checkpointAt.toIso8601String(),
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Returns the decoded checkpoint JSON for a workout, or null if none
  /// has been saved yet.
  Future<Map<String, dynamic>?> getCheckpoint(String workoutId) async {
    final db = await database;
    if (db == null) return null;
    final rows = await db.query('workout_checkpoints',
        where: 'workout_id = ?', whereArgs: [workoutId], limit: 1);
    if (rows.isEmpty) return null;
    return jsonDecode(rows.first['checkpoint_json'] as String)
        as Map<String, dynamic>;
  }

  /// Returns the most recently written checkpoint across *any* workout
  /// (used at app startup — there should only ever be one active
  /// workout, but this is defensively ordered by recency regardless).
  Future<Map<String, dynamic>?> getLatestCheckpoint() async {
    final db = await database;
    if (db == null) return null;
    final rows = await db.query('workout_checkpoints',
        orderBy: 'checkpoint_at DESC', limit: 1);
    if (rows.isEmpty) return null;
    return jsonDecode(rows.first['checkpoint_json'] as String)
        as Map<String, dynamic>;
  }

  /// Removes the checkpoint for a workout once it has finished/been
  /// discarded and no longer needs crash-recovery.
  Future<void> deleteCheckpoint(String workoutId) async {
    final db = await database;
    if (db == null) return;
    await db.delete('workout_checkpoints',
        where: 'workout_id = ?', whereArgs: [workoutId]);
  }

  // ─── Pending Deletions (tombstone sync queue) ──────────────────

  /// Records that a workout was deleted locally, so the sync layer can
  /// propagate the deletion to the cloud (and so a stale cloud copy
  /// doesn't resurrect it before that propagation happens).
  Future<void> queueDeletion(String workoutId, String userId) async {
    final db = await database;
    if (db == null) return;
    await db.insert(
      'pending_deletions',
      {
        'workout_id': workoutId,
        'user_id': userId,
        'deleted_at': DateTime.now().toIso8601String(),
        'synced': 0,
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Returns all deletions not yet propagated to the cloud.
  Future<List<Map<String, dynamic>>> getPendingDeletions() async {
    final db = await database;
    if (db == null) return [];
    return db.query('pending_deletions', where: 'synced = 0');
  }

  /// Marks a deletion as successfully propagated. Kept as a synced
  /// tombstone rather than deleted outright, so a slow/duplicate cloud
  /// sync pass can't resurrect the workout; call [purgeSyncedDeletions]
  /// periodically to garbage-collect old tombstones.
  Future<void> markDeletionSynced(String workoutId) async {
    final db = await database;
    if (db == null) return;
    await db.update('pending_deletions', {'synced': 1},
        where: 'workout_id = ?', whereArgs: [workoutId]);
  }

  /// Garbage-collects tombstones that have been synced for longer than
  /// [olderThan] (default 30 days) — by then every client has almost
  /// certainly already observed the deletion.
  Future<void> purgeSyncedDeletions(
      {Duration olderThan = const Duration(days: 30)}) async {
    final db = await database;
    if (db == null) return;
    final cutoff = DateTime.now().subtract(olderThan).toIso8601String();
    await db.delete('pending_deletions',
        where: 'synced = 1 AND deleted_at < ?', whereArgs: [cutoff]);
  }

  // ─── Pending Uploads ───────────────────────────────────────────

  /// Queues a completed workout for cloud upload.
  Future<void> queueForUpload(
      WalkingSession session, List<RoutePoint> points) async {
    final db = await database;
    if (db == null) return;
    await db.insert('pending_uploads', {
      'id': session.id,
      'user_id': session.userId,
      'workout_json': jsonEncode(session.toMap()),
      'route_points_json':
          jsonEncode(points.map((p) => p.toMap()).toList()),
      'sync_state': SyncState.pending.name,
      'retry_count': 0,
      'created_at': DateTime.now().toIso8601String(),
    });
  }

  /// Returns all pending uploads.
  Future<List<Map<String, dynamic>>> getPendingUploads() async {
    final db = await database;
    if (db == null) return [];
    return db.query('pending_uploads',
        where: 'sync_state IN (?, ?)',
        whereArgs: [SyncState.pending.name, SyncState.failed.name],
        orderBy: 'created_at ASC');
  }

  /// Updates the sync state of a pending upload.
  Future<void> updateUploadState(
      String workoutId, SyncState state,
      {String? error}) async {
    final db = await database;
    if (db == null) return;
    await db.update(
      'pending_uploads',
      {
        'sync_state': state.name,
        'last_attempt': DateTime.now().toIso8601String(),
        if (state == SyncState.failed) 'error_message': error,
        if (state == SyncState.failed)
          'retry_count': (await _getRetryCount(db, workoutId)) + 1,
      },
      where: 'id = ?',
      whereArgs: [workoutId],
    );
  }

  Future<int> _getRetryCount(Database db, String workoutId) async {
    final rows = await db.query('pending_uploads',
        columns: ['retry_count'],
        where: 'id = ?',
        whereArgs: [workoutId]);
    if (rows.isEmpty) return 0;
    return rows.first['retry_count'] as int? ?? 0;
  }

  /// Removes a successfully synced workout from the upload queue.
  Future<void> removeFromUploadQueue(String workoutId) async {
    final db = await database;
    if (db == null) return;
    await db.delete('pending_uploads',
        where: 'id = ?', whereArgs: [workoutId]);
  }

  // ─── Cache ─────────────────────────────────────────────────────

  /// Caches a training plan locally for offline access.
  Future<void> cachePlan(
      String planId, String userId, Map<String, dynamic> planJson) async {
    final db = await database;
    if (db == null) return;
    await db.insert(
      'cached_plans',
      {
        'id': planId,
        'user_id': userId,
        'plan_json': jsonEncode(planJson),
        'cached_at': DateTime.now().toIso8601String(),
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Returns cached training plans for offline display.
  Future<List<Map<String, dynamic>>> getCachedPlans(String userId) async {
    final db = await database;
    if (db == null) return [];
    final rows = await db.query('cached_plans',
        where: 'user_id = ?', whereArgs: [userId]);
    return rows
        .map((r) =>
            jsonDecode(r['plan_json'] as String) as Map<String, dynamic>)
        .toList();
  }

  /// Caches a completed session for offline history.
  Future<void> cacheSession(WalkingSession session) async {
    final db = await database;
    if (db == null) return;
    await db.insert(
      'cached_sessions',
      {
        'id': session.id,
        'user_id': session.userId,
        'session_json': jsonEncode(session.toMap()),
        'cached_at': DateTime.now().toIso8601String(),
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Returns cached sessions for offline history display.
  Future<List<WalkingSession>> getCachedSessions(String userId) async {
    final db = await database;
    if (db == null) return [];
    final rows = await db.query('cached_sessions',
        where: 'user_id = ?',
        whereArgs: [userId],
        orderBy: 'cached_at DESC');
    return rows
        .map((r) => WalkingSession.fromMap(
            jsonDecode(r['session_json'] as String)
                as Map<String, dynamic>))
        .toList();
  }

  // ─── Cleanup ───────────────────────────────────────────────────

  /// Clears all data for a user (e.g. on logout).
  Future<void> clearUserData(String userId) async {
    final db = await database;
    if (db == null) return;
    await db.delete('cached_sessions',
        where: 'user_id = ?', whereArgs: [userId]);
    await db.delete('cached_plans',
        where: 'user_id = ?', whereArgs: [userId]);
    await db.delete('pending_uploads',
        where: 'user_id = ?', whereArgs: [userId]);
    await db.delete('pending_deletions',
        where: 'user_id = ?', whereArgs: [userId]);
  }

  /// Closes the database connection.
  Future<void> close() async {
    final db = _db;
    if (db != null && db.isOpen) {
      await db.close();
      _db = null;
    }
  }
}
