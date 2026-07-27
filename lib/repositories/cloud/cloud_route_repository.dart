import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

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

  SavedRoute copyWith({
    required bool isFavorite,
  }) => SavedRoute(
        id: id,
        userId: userId,
        name: name,
        distanceMeters: distanceMeters,
        elevationGainMeters: elevationGainMeters,
        routeFilePath: routeFilePath,
        previewUrl: previewUrl,
        startLocation: startLocation,
        endLocation: endLocation,
        createdAt: createdAt,
        isFavorite: isFavorite,
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

/// Real route repository backed by SharedPreferences for local
/// persistence. Saved routes are stored as JSON keyed by route ID.
/// When Firebase is connected, swap for `FirestoreRouteRepository`.
class LocalCloudRouteRepository implements CloudRouteRepository {
  static const _keyPrefix = 'cloud_route_';
  static const _keyRouteFiles = 'cloud_route_files_';

  Future<SharedPreferences> get _prefs async =>
      await SharedPreferences.getInstance();

  @override
  Future<void> saveRoute(SavedRoute route) async {
    final prefs = await _prefs;
    await prefs.setString(
        '$_keyPrefix${route.id}', jsonEncode(route.toMap()));
  }

  @override
  Future<List<SavedRoute>> getRoutes(String userId) async {
    final prefs = await _prefs;
    final keys = prefs.getKeys().where((k) => k.startsWith(_keyPrefix));
    final routes = <SavedRoute>[];
    for (final key in keys) {
      final json = prefs.getString(key);
      if (json != null) {
        final route = SavedRoute.fromMap(jsonDecode(json) as Map<String, dynamic>);
        if (route.userId == userId) routes.add(route);
      }
    }
    routes.sort((a, b) => b.createdAt.compareTo(a.createdAt));
    return routes;
  }

  @override
  Future<void> toggleFavorite(String userId, String routeId) async {
    final prefs = await _prefs;
    final json = prefs.getString('$_keyPrefix$routeId');
    if (json == null) return;
    final route = SavedRoute.fromMap(jsonDecode(json) as Map<String, dynamic>);
    if (route.userId != userId) return;
    await prefs.setString(
        '$_keyPrefix$routeId',
        jsonEncode(route.copyWith(isFavorite: !route.isFavorite).toMap()));
  }

  @override
  Future<void> deleteRoute(String userId, String routeId) async {
    final prefs = await _prefs;
    await prefs.remove('$_keyPrefix$routeId');
    await prefs.remove('$_keyRouteFiles$routeId');
  }

  @override
  Future<String> uploadRouteFile(
      String userId, String workoutId, List<RoutePoint> points) async {
    final prefs = await _prefs;
    final compressed = jsonEncode(points.map((p) => p.toMap()).toList());
    final storagePath = 'users/$userId/routes/$workoutId.json';
    await prefs.setString('$_keyRouteFiles$workoutId', compressed);
    return storagePath;
  }
}
