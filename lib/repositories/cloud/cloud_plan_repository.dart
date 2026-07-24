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

/// In-memory implementation. Swap with `FirestorePlanRepository`.
class MockCloudPlanRepository implements CloudPlanRepository {
  final Map<String, List<WalkingPlan>> _store = {};

  @override
  Future<void> savePlan(String userId, WalkingPlan plan) async {
    await Future.delayed(const Duration(milliseconds: 80));
    final plans = _store[userId] ?? [];
    plans.removeWhere((p) => p.id == plan.id);
    plans.add(plan);
    _store[userId] = plans;
  }

  @override
  Future<WalkingPlan?> getActivePlan(String userId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    final plans = _store[userId] ?? [];
    try {
      return plans.firstWhere((p) => p.isActive);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<List<WalkingPlan>> getPlans(String userId) async {
    await Future.delayed(const Duration(milliseconds: 80));
    return _store[userId] ?? [];
  }

  @override
  Future<void> updatePlanDay(
      String userId, String planId, int dayNumber, bool completed) async {
    await Future.delayed(const Duration(milliseconds: 50));
    final plans = _store[userId] ?? [];
    final planIndex = plans.indexWhere((p) => p.id == planId);
    if (planIndex == -1) return;
    final plan = plans[planIndex];
    final updatedDays = plan.days
        .map((d) => d.dayNumber == dayNumber
            ? d.copyWith(isCompleted: completed)
            : d)
        .toList();
    plans[planIndex] = WalkingPlan(
      id: plan.id,
      userId: plan.userId,
      name: plan.name,
      goal: plan.goal,
      durationWeeks: plan.durationWeeks,
      days: updatedDays,
      createdAt: plan.createdAt,
      isActive: plan.isActive,
    );
    _store[userId] = plans;
  }

  @override
  Future<void> deletePlan(String userId, String planId) async {
    await Future.delayed(const Duration(milliseconds: 50));
    _store[userId]?.removeWhere((p) => p.id == planId);
  }
}
