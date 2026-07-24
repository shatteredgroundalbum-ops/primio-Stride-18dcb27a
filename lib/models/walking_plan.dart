enum PlanActivityType { walk, run, interval, recovery, rest }

class WalkingPlan {
  final String id;
  final String userId;
  final String name;
  final String goal;
  final int durationWeeks;
  final List<PlanDay> days;
  final DateTime createdAt;
  final bool isActive;

  const WalkingPlan({
    required this.id,
    required this.userId,
    required this.name,
    required this.goal,
    required this.durationWeeks,
    required this.days,
    required this.createdAt,
    this.isActive = true,
  });

  int get completedDays => days.where((d) => d.isCompleted).length;
  double get progress => days.isEmpty ? 0 : completedDays / days.length;

  Map<String, dynamic> toMap() => {
        'id': id,
        'userId': userId,
        'name': name,
        'goal': goal,
        'durationWeeks': durationWeeks,
        'days': days.map((d) => d.toMap()).toList(),
        'createdAt': createdAt.toIso8601String(),
        'isActive': isActive,
      };

  factory WalkingPlan.fromMap(Map<String, dynamic> map) => WalkingPlan(
        id: map['id'] as String,
        userId: map['userId'] as String,
        name: map['name'] as String,
        goal: map['goal'] as String,
        durationWeeks: map['durationWeeks'] as int,
        days: (map['days'] as List)
            .map((d) => PlanDay.fromMap(d as Map<String, dynamic>))
            .toList(),
        createdAt: DateTime.parse(map['createdAt'] as String),
        isActive: map['isActive'] as bool? ?? true,
      );
}

class PlanDay {
  final int dayNumber;
  final int weekNumber;
  final PlanActivityType activityType;
  final int targetMinutes;
  final double? targetDistanceKm;
  final String notes;
  final bool isCompleted;

  const PlanDay({
    required this.dayNumber,
    required this.weekNumber,
    required this.activityType,
    required this.targetMinutes,
    this.targetDistanceKm,
    this.notes = '',
    this.isCompleted = false,
  });

  Map<String, dynamic> toMap() => {
        'dayNumber': dayNumber,
        'weekNumber': weekNumber,
        'activityType': activityType.name,
        'targetMinutes': targetMinutes,
        'targetDistanceKm': targetDistanceKm,
        'notes': notes,
        'isCompleted': isCompleted,
      };

  factory PlanDay.fromMap(Map<String, dynamic> map) => PlanDay(
        dayNumber: map['dayNumber'] as int,
        weekNumber: map['weekNumber'] as int,
        activityType: PlanActivityType.values
            .firstWhere((e) => e.name == map['activityType']),
        targetMinutes: map['targetMinutes'] as int,
        targetDistanceKm: (map['targetDistanceKm'] as num?)?.toDouble(),
        notes: map['notes'] as String? ?? '',
        isCompleted: map['isCompleted'] as bool? ?? false,
      );

  PlanDay copyWith({bool? isCompleted}) => PlanDay(
        dayNumber: dayNumber,
        weekNumber: weekNumber,
        activityType: activityType,
        targetMinutes: targetMinutes,
        targetDistanceKm: targetDistanceKm,
        notes: notes,
        isCompleted: isCompleted ?? this.isCompleted,
      );
}
