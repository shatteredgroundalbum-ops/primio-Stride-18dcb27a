class FastingProtocol {
  final String name;
  final int fastHours;
  final int eatHours;
  final String description;
  final String difficulty;

  const FastingProtocol({
    required this.name,
    required this.fastHours,
    required this.eatHours,
    required this.description,
    required this.difficulty,
  });

  String get label => '$fastHours:$eatHours';
}

class FastingLog {
  final DateTime date;
  final FastingProtocol protocol;
  final DateTime startTime;
  final DateTime? endTime;
  final bool completed;
  final Duration targetDuration;

  const FastingLog({
    required this.date,
    required this.protocol,
    required this.startTime,
    this.endTime,
    required this.completed,
    required this.targetDuration,
  });

  Duration get actualDuration {
    if (endTime != null) return endTime!.difference(startTime);
    return DateTime.now().difference(startTime);
  }

  double get progress =>
      targetDuration.inMinutes > 0
          ? (actualDuration.inMinutes / targetDuration.inMinutes).clamp(0.0, 1.0)
          : 0.0;
}

class FastingStats {
  final int totalFasts;
  final int completedFasts;
  final int currentStreak;
  final int longestStreak;
  final Duration averageDuration;
  final Duration longestFast;

  const FastingStats({
    required this.totalFasts,
    required this.completedFasts,
    required this.currentStreak,
    required this.longestStreak,
    required this.averageDuration,
    required this.longestFast,
  });

  double get completionRate =>
      totalFasts > 0 ? completedFasts / totalFasts : 0.0;
}
