import 'sync_status.dart';

enum SessionStatus { active, paused, completed, cancelled }

enum SessionType { walk, run, hike }

/// Production-ready walking session matching the Firestore document schema:
/// `users/{userId}/workouts/{workoutId}`
///
/// GPS route points are NOT stored in this document — they are compressed
/// and uploaded to Cloud Storage. This document holds the summary and a
/// reference to the route file via [routeFilePath] / [routePreviewUrl].
class WalkingSession {
  final String id;
  final String userId;
  final DateTime startTime;
  final DateTime? endTime;
  final double distanceMeters;
  final Duration duration;
  final double averagePaceMinPerKm;
  final double caloriesBurned;
  final double averageSpeedKmh;
  final double? maxSpeedKmh;
  final double? elevationGainMeters;
  final int stepCount;
  final int? averageHeartRate;
  final SessionStatus status;
  final SessionType type;

  /// Cloud Storage path to the compressed GPS route file.
  final String? routeFilePath;

  /// URL to a rendered preview image of the route.
  final String? routePreviewUrl;

  /// Start and end coordinates for quick display without loading the route.
  final GeoLocation? startLocation;
  final GeoLocation? endLocation;

  /// Sync state for the local → cloud pipeline.
  final SyncMetadata? syncMetadata;

  /// In-memory only — not persisted to Firestore.
  final List<RoutePoint> routePoints;

  const WalkingSession({
    required this.id,
    required this.userId,
    required this.startTime,
    this.endTime,
    this.distanceMeters = 0,
    this.duration = Duration.zero,
    this.averagePaceMinPerKm = 0,
    this.caloriesBurned = 0,
    this.averageSpeedKmh = 0,
    this.maxSpeedKmh,
    this.elevationGainMeters,
    this.stepCount = 0,
    this.averageHeartRate,
    this.status = SessionStatus.active,
    this.type = SessionType.walk,
    this.routeFilePath,
    this.routePreviewUrl,
    this.startLocation,
    this.endLocation,
    this.syncMetadata,
    this.routePoints = const [],
  });

  double get distanceKm => distanceMeters / 1000;

  /// Serializes to the Firestore document format (no route points).
  Map<String, dynamic> toMap() => {
        'id': id,
        'userId': userId,
        'activityType': type.name,
        'startedAt': startTime.toIso8601String(),
        'endedAt': endTime?.toIso8601String(),
        'durationSeconds': duration.inSeconds,
        'distanceMeters': distanceMeters,
        'averagePace': averagePaceMinPerKm,
        'maximumSpeed': maxSpeedKmh,
        'averageSpeedKmh': averageSpeedKmh,
        'stepCount': stepCount,
        'averageHeartRate': averageHeartRate,
        'caloriesEstimated': caloriesBurned,
        'elevationGain': elevationGainMeters,
        'routeFilePath': routeFilePath,
        'routePreviewUrl': routePreviewUrl,
        'startLocation': startLocation?.toMap(),
        'endLocation': endLocation?.toMap(),
        'syncStatus': syncMetadata?.state.name ?? SyncState.pending.name,
        'status': status.name,
      };

  factory WalkingSession.fromMap(Map<String, dynamic> map) => WalkingSession(
        id: map['id'] as String,
        userId: map['userId'] as String,
        startTime: DateTime.parse(
            (map['startedAt'] ?? map['startTime']) as String),
        endTime: map['endedAt'] != null
            ? DateTime.parse(map['endedAt'] as String)
            : map['endTime'] != null
                ? DateTime.parse(map['endTime'] as String)
                : null,
        distanceMeters: (map['distanceMeters'] as num).toDouble(),
        duration: Duration(seconds: map['durationSeconds'] as int),
        averagePaceMinPerKm:
            (map['averagePace'] ?? map['averagePaceMinPerKm'] as num)
                .toDouble(),
        caloriesBurned:
            (map['caloriesEstimated'] ?? map['caloriesBurned'] as num)
                .toDouble(),
        averageSpeedKmh: (map['averageSpeedKmh'] as num).toDouble(),
        maxSpeedKmh: (map['maximumSpeed'] as num?)?.toDouble(),
        elevationGainMeters:
            (map['elevationGain'] ?? map['elevationGainMeters'] as num?)
                ?.toDouble(),
        stepCount: map['stepCount'] as int? ?? 0,
        averageHeartRate: map['averageHeartRate'] as int?,
        routeFilePath: map['routeFilePath'] as String?,
        routePreviewUrl: map['routePreviewUrl'] as String?,
        startLocation: map['startLocation'] != null
            ? GeoLocation.fromMap(
                map['startLocation'] as Map<String, dynamic>)
            : null,
        endLocation: map['endLocation'] != null
            ? GeoLocation.fromMap(
                map['endLocation'] as Map<String, dynamic>)
            : null,
        status: SessionStatus.values.firstWhere(
          (e) => e.name == map['status'],
          orElse: () => SessionStatus.completed,
        ),
        type: SessionType.values.firstWhere(
          (e) => e.name == (map['activityType'] ?? map['type']),
          orElse: () => SessionType.walk,
        ),
      );

  WalkingSession copyWith({
    DateTime? endTime,
    double? distanceMeters,
    Duration? duration,
    double? averagePaceMinPerKm,
    double? caloriesBurned,
    double? averageSpeedKmh,
    double? maxSpeedKmh,
    double? elevationGainMeters,
    int? stepCount,
    int? averageHeartRate,
    List<RoutePoint>? routePoints,
    SessionStatus? status,
    String? routeFilePath,
    String? routePreviewUrl,
    GeoLocation? startLocation,
    GeoLocation? endLocation,
    SyncMetadata? syncMetadata,
  }) =>
      WalkingSession(
        id: id,
        userId: userId,
        startTime: startTime,
        endTime: endTime ?? this.endTime,
        distanceMeters: distanceMeters ?? this.distanceMeters,
        duration: duration ?? this.duration,
        averagePaceMinPerKm:
            averagePaceMinPerKm ?? this.averagePaceMinPerKm,
        caloriesBurned: caloriesBurned ?? this.caloriesBurned,
        averageSpeedKmh: averageSpeedKmh ?? this.averageSpeedKmh,
        maxSpeedKmh: maxSpeedKmh ?? this.maxSpeedKmh,
        elevationGainMeters:
            elevationGainMeters ?? this.elevationGainMeters,
        stepCount: stepCount ?? this.stepCount,
        averageHeartRate: averageHeartRate ?? this.averageHeartRate,
        routePoints: routePoints ?? this.routePoints,
        status: status ?? this.status,
        type: type,
        routeFilePath: routeFilePath ?? this.routeFilePath,
        routePreviewUrl: routePreviewUrl ?? this.routePreviewUrl,
        startLocation: startLocation ?? this.startLocation,
        endLocation: endLocation ?? this.endLocation,
        syncMetadata: syncMetadata ?? this.syncMetadata,
      );
}

/// Lightweight lat/lng pair for start/end location in Firestore documents.
class GeoLocation {
  final double latitude;
  final double longitude;

  const GeoLocation({required this.latitude, required this.longitude});

  Map<String, dynamic> toMap() => {
        'latitude': latitude,
        'longitude': longitude,
      };

  factory GeoLocation.fromMap(Map<String, dynamic> map) => GeoLocation(
        latitude: (map['latitude'] as num).toDouble(),
        longitude: (map['longitude'] as num).toDouble(),
      );
}

class RoutePoint {
  final double latitude;
  final double longitude;
  final double? altitude;
  final double? speed;
  final DateTime timestamp;

  const RoutePoint({
    required this.latitude,
    required this.longitude,
    this.altitude,
    this.speed,
    required this.timestamp,
  });

  Map<String, dynamic> toMap() => {
        'latitude': latitude,
        'longitude': longitude,
        'altitude': altitude,
        'speed': speed,
        'timestamp': timestamp.toIso8601String(),
      };

  factory RoutePoint.fromMap(Map<String, dynamic> map) => RoutePoint(
        latitude: (map['latitude'] as num).toDouble(),
        longitude: (map['longitude'] as num).toDouble(),
        altitude: (map['altitude'] as num?)?.toDouble(),
        speed: (map['speed'] as num?)?.toDouble(),
        timestamp: DateTime.parse(map['timestamp'] as String),
      );
}
