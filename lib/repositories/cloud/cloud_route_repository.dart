import 'dart:convert';

import '../../models/walking_session.dart';

/// Saved route metadata matching Firestore path:
/// `users/{userId}/savedRoutes/{routeId}`
///
/// The actual GPS data is a compressed file in Cloud Storage.
/// This document holds the metadata and a reference to that file.
class SavedRoute {
  final String id;
  final String userId;
  final String name;
  final double distanceMeters;
  final double? elevationGainMeters;

  /// Cloud Storage path to the compressed route file.
  final String routeFilePath;

  /// Rendered preview image URL.
  final String? previewUrl;

  final GeoLocation startLocation;
  final GeoLocation endLocation;
  final DateTime createdAt;
  final bool isFavorite;

  const SavedRoute({
    required this.id,
    required this.userId,
    required this.name,
    required this.distanceMeters,
    this.elevationGainMeters,
    required this.routeFilePath,
    this.previewUrl,
    required this.startLocation,
    required this.endLocation,
    required this.createdAt,
    this.isFavorite = false,
  });

  Map<String, dynamic> toMap() => {
        'id': id,
        'userId': userId,
        'name': name,
        'distanceMeters': distanceMeters,
        'elevationGainMeters': elevationGainMeters,
        'routeFilePath': routeFilePath,
        'previewUrl': previewUrl,
        'startLocation': startLocation.toMap(),
        'endLocation': endLocation.toMap(),
        'createdAt': createdAt.toIso8601String(),
        'isFavorite': isFavorite,
      };

  factory SavedRoute.fromMap(Map<String, dynamic> map) => SavedRoute(
        id: map['id'] as String,
        userId: map['userId'] as String,
        name: map['name'] as String,
        distanceMeters: (map['distanceMeters'] as num).toDouble(),
        elevationGainMeters:
            (map['elevationGainMeters'] as num?)?.toDouble(),
        routeFilePath: map['routeFilePath'] as String,
        previewUrl: map['previewUrl'] as String?,
        startLocation: GeoLocation.fromMap(
            map['startLocation'] as Map<String, dynamic>),
        endLocation: GeoLocation.fromMap(
            map['endLocation'] as Map<String, dynamic>),
        createdAt: DateTime.parse(map['createdAt'] as String),
        isFavorite: map['isFavorite'] as bool? ?? false,
      );
}

/// Handles saved routes in Firestore and route files in Cloud Storage.
abstract class CloudRouteRepository {
  Future<void> saveRoute(SavedRoute route);
  Future<List<SavedRoute>> getRoutes(String userId);
  Future<void> toggleFavorite(String userId, String routeId);
  Future<void> deleteRoute(String userId, String routeId);

  /// Uploads compressed route points to Cloud Storage.
  /// Returns the storage path for the uploaded file.
  Future<String> uploadRouteFile(
      String userId, String workoutId, List<RoutePoint> points);
}

/// In-memory implementation. Swap with `FirestoreRouteRepository`.
class MockCloudRouteRepository implements CloudRouteRepository {
  final List<SavedRoute> _routes = [];
  final Map<String, String> _routeFiles = {};

  @override
  Future<void> saveRoute(SavedRoute route) async {
    await Future.delayed(const Duration(milliseconds: 80));
    _routes.removeWhere((r) => r.id == route.id);
    _routes.add(route);
  }

  @override
  Future<List<SavedRoute>> getRoutes(String userId) async {
    await Future.delayed(const Duration(milliseconds: 80));
    return _routes
        .where((r) => r.userId == userId)
        .toList()
      ..sort((a, b) => b.createdAt.compareTo(a.createdAt));
  }

  @override
  Future<void> toggleFavorite(String userId, String routeId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    final idx = _routes
        .indexWhere((r) => r.id == routeId && r.userId == userId);
    if (idx == -1) return;
    final route = _routes[idx];
    _routes[idx] = SavedRoute(
      id: route.id,
      userId: route.userId,
      name: route.name,
      distanceMeters: route.distanceMeters,
      elevationGainMeters: route.elevationGainMeters,
      routeFilePath: route.routeFilePath,
      previewUrl: route.previewUrl,
      startLocation: route.startLocation,
      endLocation: route.endLocation,
      createdAt: route.createdAt,
      isFavorite: !route.isFavorite,
    );
  }

  @override
  Future<void> deleteRoute(String userId, String routeId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    _routes.removeWhere(
        (r) => r.id == routeId && r.userId == userId);
    _routeFiles.remove(routeId);
  }

  @override
  Future<String> uploadRouteFile(
      String userId, String workoutId, List<RoutePoint> points) async {
    await Future.delayed(const Duration(milliseconds: 200));
    // In production: compress points → upload to Cloud Storage → return path.
    final compressed = jsonEncode(points.map((p) => p.toMap()).toList());
    final storagePath =
        'users/$userId/routes/$workoutId.json';
    _routeFiles[workoutId] = compressed;
    return storagePath;
  }
}
