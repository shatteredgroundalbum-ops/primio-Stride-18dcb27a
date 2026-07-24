enum SessionStatus { active, paused, completed, cancelled }

enum SessionType { walk, run, hike }

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
  final double? elevationGainMeters;
  final List<RoutePoint> routePoints;
  final SessionStatus status;
  final SessionType type;

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
    this.elevationGainMeters,
    this.routePoints = const [],
    this.status = SessionStatus.active,
    this.type = SessionType.walk,
  });

  double get distanceKm => distanceMeters / 1000;

  Map<String, dynamic> toMap() => {
        'id': id,
        'userId': userId,
        'startTime': startTime.toIso8601String(),
        'endTime': endTime?.toIso8601String(),
        'distanceMeters': distanceMeters,
        'durationSeconds': duration.inSeconds,
        'averagePaceMinPerKm': averagePaceMinPerKm,
        'caloriesBurned': caloriesBurned,
        'averageSpeedKmh': averageSpeedKmh,
        'elevationGainMeters': elevationGainMeters,
        'status': status.name,
        'type': type.name,
      };

  factory WalkingSession.fromMap(Map<String, dynamic> map) => WalkingSession(
        id: map['id'] as String,
        userId: map['userId'] as String,
        startTime: DateTime.parse(map['startTime'] as String),
        endTime: map['endTime'] != null
            ? DateTime.parse(map['endTime'] as String)
            : null,
        distanceMeters: (map['distanceMeters'] as num).toDouble(),
        duration: Duration(seconds: map['durationSeconds'] as int),
        averagePaceMinPerKm: (map['averagePaceMinPerKm'] as num).toDouble(),
        caloriesBurned: (map['caloriesBurned'] as num).toDouble(),
        averageSpeedKmh: (map['averageSpeedKmh'] as num).toDouble(),
        elevationGainMeters:
            (map['elevationGainMeters'] as num?)?.toDouble(),
        status: SessionStatus.values
            .firstWhere((e) => e.name == map['status']),
        type: SessionType.values.firstWhere(
          (e) => e.name == map['type'],
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
    double? elevationGainMeters,
    List<RoutePoint>? routePoints,
    SessionStatus? status,
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
        elevationGainMeters:
            elevationGainMeters ?? this.elevationGainMeters,
        routePoints: routePoints ?? this.routePoints,
        status: status ?? this.status,
        type: type,
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
