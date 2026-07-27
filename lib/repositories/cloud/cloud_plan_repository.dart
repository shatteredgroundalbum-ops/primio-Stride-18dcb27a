import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import '../../models/walking_plan.dart';

/// Abstract interface matching Firestore path:
/// `users/{userId}/trainingPlans/{planId}`
abstract class CloudPlanRepository {
  Future<void> savePlan(String userId, WalkingPlan plan);
  Future<WalkingPlan?> getActivePlan(String userId);
  Future<List<WalkingPlan>> getPlans(String userId);
  Future<void> updatePlanDay(
      String userId, String planId, int dayNumber, bool completed);
  Future<void> deletePlan(String userId, String planId);
}

/// Real plan repository backed by SharedPreferences for local
/// persistence. Plans are stored as JSON keyed by plan ID. When
/// Firebase is connected, swap for `FirestorePlanRepository`.
class LocalCloudPlanRepository implements CloudPlanRepository {
  static const _keyPrefix = 'cloud_plan_';

  Future<SharedPreferences> get _prefs async =>
      await SharedPreferences.getInstance();

  @override
  Future<void> savePlan(String userId, WalkingPlan plan) async {
    final prefs = await _prefs;
    await prefs.setString(
        '$_keyPrefix${plan.id}', jsonEncode(plan.toMap()));
  }

  @override
  Future<WalkingPlan?> getActivePlan(String userId) async {
    final plans = await getPlans(userId);
    try {
      return plans.firstWhere((p) => p.isActive);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<List<WalkingPlan>> getPlans(String userId) async {
    final prefs = await _prefs;
    final keys = prefs.getKeys().where((k) => k.startsWith(_keyPrefix));
    final plans = <WalkingPlan>[];
    for (final key in keys) {
      final json = prefs.getString(key);
      if (json != null) {
        final plan = WalkingPlan.fromMap(jsonDecode(json) as Map<String, dynamic>);
        if (plan.userId == userId) plans.add(plan);
      }
    }
    return plans;
  }

  @override
  Future<void> updatePlanDay(
      String userId, String planId, int dayNumber, bool completed) async {
    final prefs = await _prefs;
    final json = prefs.getString('$_keyPrefix$planId');
    if (json == null) return;
    final plan = WalkingPlan.fromMap(jsonDecode(json) as Map<String, dynamic>);
    if (plan.userId != userId) return;

    final updatedDays = plan.days
        .map((d) => d.dayNumber == dayNumber
            ? d.copyWith(isCompleted: completed)
            : d)
        .toList();

    final updated = WalkingPlan(
      id: plan.id,
      userId: plan.userId,
      name: plan.name,
      goal: plan.goal,
      durationWeeks: plan.durationWeeks,
      days: updatedDays,
      createdAt: plan.createdAt,
      isActive: plan.isActive,
    );
    await prefs.setString(
        '$_keyPrefix$planId', jsonEncode(updated.toMap()));
  }

  @override
  Future<void> deletePlan(String userId, String planId) async {
    final prefs = await _prefs;
    await prefs.remove('$_keyPrefix$planId');
  }
}
