import '../models/fasting_data.dart';

/// Fasting service — returns empty state by default. No mock or
/// hardcoded data. When the user starts/ends fasts through the UI,
/// the data will be persisted and read back from storage.
class FastingService {
  static const protocols = [
    FastingProtocol(name: '16:8', fastHours: 16, eatHours: 8, description: 'Most popular. Fast 16h, eat within an 8h window.', difficulty: 'Beginner'),
    FastingProtocol(name: '18:6', fastHours: 18, eatHours: 6, description: 'Extended fast with a tighter eating window.', difficulty: 'Intermediate'),
    FastingProtocol(name: '20:4', fastHours: 20, eatHours: 4, description: 'Warrior Diet. One large meal with snacks.', difficulty: 'Advanced'),
    FastingProtocol(name: 'OMAD', fastHours: 23, eatHours: 1, description: 'One Meal A Day. Maximum fasting benefits.', difficulty: 'Expert'),
    FastingProtocol(name: '14:10', fastHours: 14, eatHours: 10, description: 'Gentle introduction to fasting.', difficulty: 'Beginner'),
  ];

  /// Returns null — no active fast until the user starts one.
  Future<FastingLog?> getActiveFast() async {
    return null;
  }

  /// Returns empty list — no fasting history until the user has data.
  Future<List<FastingLog>> getRecentLogs() async {
    return const [];
  }

  /// Returns zero-state stats — no fasting data yet.
  Future<FastingStats> getStats() async {
    return const FastingStats(
      totalFasts: 0,
      completedFasts: 0,
      currentStreak: 0,
      longestStreak: 0,
      averageDuration: Duration.zero,
      longestFast: Duration.zero,
    );
  }
}
