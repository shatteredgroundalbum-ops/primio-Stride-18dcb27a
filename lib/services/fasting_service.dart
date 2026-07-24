import '../models/fasting_data.dart';

class FastingService {
  static const protocols = [
    FastingProtocol(name: '16:8', fastHours: 16, eatHours: 8, description: 'Most popular. Fast 16h, eat within an 8h window.', difficulty: 'Beginner'),
    FastingProtocol(name: '18:6', fastHours: 18, eatHours: 6, description: 'Extended fast with a tighter eating window.', difficulty: 'Intermediate'),
    FastingProtocol(name: '20:4', fastHours: 20, eatHours: 4, description: 'Warrior Diet. One large meal with snacks.', difficulty: 'Advanced'),
    FastingProtocol(name: 'OMAD', fastHours: 23, eatHours: 1, description: 'One Meal A Day. Maximum fasting benefits.', difficulty: 'Expert'),
    FastingProtocol(name: '14:10', fastHours: 14, eatHours: 10, description: 'Gentle introduction to fasting.', difficulty: 'Beginner'),
  ];

  Future<FastingLog?> getActiveFast() async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    return FastingLog(
      date: DateTime.now(),
      protocol: protocols[0],
      startTime: DateTime.now().subtract(const Duration(hours: 12, minutes: 34)),
      completed: false,
      targetDuration: const Duration(hours: 16),
    );
  }

  Future<List<FastingLog>> getRecentLogs() async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    final now = DateTime.now();
    return [
      FastingLog(
        date: now.subtract(const Duration(days: 1)),
        protocol: protocols[0],
        startTime: now.subtract(const Duration(days: 1, hours: 20)),
        endTime: now.subtract(const Duration(days: 1, hours: 4)),
        completed: true,
        targetDuration: const Duration(hours: 16),
      ),
      FastingLog(
        date: now.subtract(const Duration(days: 2)),
        protocol: protocols[0],
        startTime: now.subtract(const Duration(days: 2, hours: 21)),
        endTime: now.subtract(const Duration(days: 2, hours: 5)),
        completed: true,
        targetDuration: const Duration(hours: 16),
      ),
      FastingLog(
        date: now.subtract(const Duration(days: 3)),
        protocol: protocols[1],
        startTime: now.subtract(const Duration(days: 3, hours: 20)),
        endTime: now.subtract(const Duration(days: 3, hours: 3)),
        completed: false,
        targetDuration: const Duration(hours: 18),
      ),
      FastingLog(
        date: now.subtract(const Duration(days: 4)),
        protocol: protocols[0],
        startTime: now.subtract(const Duration(days: 4, hours: 22)),
        endTime: now.subtract(const Duration(days: 4, hours: 6)),
        completed: true,
        targetDuration: const Duration(hours: 16),
      ),
      FastingLog(
        date: now.subtract(const Duration(days: 5)),
        protocol: protocols[0],
        startTime: now.subtract(const Duration(days: 5, hours: 20)),
        endTime: now.subtract(const Duration(days: 5, hours: 4)),
        completed: true,
        targetDuration: const Duration(hours: 16),
      ),
    ];
  }

  Future<FastingStats> getStats() async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    return const FastingStats(
      totalFasts: 42,
      completedFasts: 37,
      currentStreak: 5,
      longestStreak: 14,
      averageDuration: Duration(hours: 15, minutes: 48),
      longestFast: Duration(hours: 22, minutes: 10),
    );
  }
}
