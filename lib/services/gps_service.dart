import 'dart:async';
import 'dart:math';

import 'package:flutter/foundation.dart';
import 'package:geolocator/geolocator.dart';

import '../models/walking_session.dart';

class GpsService {
  StreamSubscription<Position>? _positionSubscription;
  final _pointController = StreamController<RoutePoint>.broadcast();
  bool _isTracking = false;

  Stream<RoutePoint> get pointStream => _pointController.stream;
  bool get isTracking => _isTracking;

  Future<bool> checkPermission() async {
    if (kIsWeb) return false;

    final serviceEnabled = await Geolocator.isLocationServiceEnabled();
    if (!serviceEnabled) return false;

    var permission = await Geolocator.checkPermission();
    if (permission == LocationPermission.denied) {
      permission = await Geolocator.requestPermission();
    }
    return permission == LocationPermission.whileInUse ||
        permission == LocationPermission.always;
  }

  Future<void> startTracking() async {
    if (kIsWeb || _isTracking) return;

    final hasPermission = await checkPermission();
    if (!hasPermission) return;

    _isTracking = true;
    const locationSettings = LocationSettings(
      accuracy: LocationAccuracy.high,
      distanceFilter: 5,
    );

    _positionSubscription = Geolocator.getPositionStream(
      locationSettings: locationSettings,
    ).listen((position) {
      final point = RoutePoint(
        latitude: position.latitude,
        longitude: position.longitude,
        altitude: position.altitude,
        speed: position.speed,
        timestamp: DateTime.now(),
      );
      _pointController.add(point);
    });
  }

  void stopTracking() {
    _isTracking = false;
    _positionSubscription?.cancel();
    _positionSubscription = null;
  }

  void dispose() {
    stopTracking();
    _pointController.close();
  }

  /// Haversine distance between two points in meters.
  static double distanceBetween(RoutePoint a, RoutePoint b) {
    const earthRadius = 6371000.0;
    final dLat = _toRadians(b.latitude - a.latitude);
    final dLon = _toRadians(b.longitude - a.longitude);
    final sinHalf = sin(dLat / 2) * sin(dLat / 2) +
        cos(_toRadians(a.latitude)) *
            cos(_toRadians(b.latitude)) *
            sin(dLon / 2) *
            sin(dLon / 2);
    final c = 2 * atan2(sqrt(sinHalf), sqrt(1 - sinHalf));
    return earthRadius * c;
  }

  static double _toRadians(double degrees) => degrees * pi / 180;

  /// Total distance of a route in meters.
  static double totalDistance(List<RoutePoint> points) {
    if (points.length < 2) return 0;
    double total = 0;
    for (int i = 1; i < points.length; i++) {
      total += distanceBetween(points[i - 1], points[i]);
    }
    return total;
  }

  /// Average pace in minutes per km.
  static double averagePace(double distanceMeters, Duration duration) {
    if (distanceMeters <= 0) return 0;
    final km = distanceMeters / 1000;
    return duration.inSeconds / 60 / km;
  }

  /// Average speed in km/h.
  static double averageSpeed(double distanceMeters, Duration duration) {
    if (duration.inSeconds <= 0) return 0;
    final km = distanceMeters / 1000;
    final hours = duration.inSeconds / 3600;
    return km / hours;
  }

  /// MET-based calorie estimate.
  static double estimateCalories({
    required double weightKg,
    required Duration duration,
    required double speedKmh,
  }) {
    double met;
    if (speedKmh < 4) {
      met = 3.0;
    } else if (speedKmh < 5.5) {
      met = 4.3;
    } else if (speedKmh < 7) {
      met = 6.0;
    } else if (speedKmh < 9) {
      met = 8.3;
    } else {
      met = 10.0 + (speedKmh - 9) * 0.5;
    }
    final hours = duration.inSeconds / 3600;
    return met * weightKg * hours;
  }

  /// Cumulative elevation gain from route points (meters).
  static double? elevationGain(List<RoutePoint> points) {
    double gain = 0;
    bool hasAltitude = false;
    for (int i = 1; i < points.length; i++) {
      final prev = points[i - 1].altitude;
      final curr = points[i].altitude;
      if (prev != null && curr != null) {
        hasAltitude = true;
        if (curr > prev) gain += curr - prev;
      }
    }
    return hasAltitude ? gain : null;
  }
}
