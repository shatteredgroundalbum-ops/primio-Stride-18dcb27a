class UserModel {
  final String id;
  final String email;
  final String displayName;
  final int age;
  final double weightKg;
  final double heightCm;
  final String activityLevel;
  final DateTime createdAt;
  final UserGoals goals;

  const UserModel({
    required this.id,
    required this.email,
    required this.displayName,
    required this.age,
    required this.weightKg,
    required this.heightCm,
    required this.activityLevel,
    required this.createdAt,
    required this.goals,
  });

  double get bmi => weightKg / ((heightCm / 100) * (heightCm / 100));

  Map<String, dynamic> toMap() => {
        'id': id,
        'email': email,
        'displayName': displayName,
        'age': age,
        'weightKg': weightKg,
        'heightCm': heightCm,
        'activityLevel': activityLevel,
        'createdAt': createdAt.toIso8601String(),
        'goals': goals.toMap(),
      };

  factory UserModel.fromMap(Map<String, dynamic> map) => UserModel(
        id: map['id'] as String,
        email: map['email'] as String,
        displayName: map['displayName'] as String,
        age: map['age'] as int,
        weightKg: (map['weightKg'] as num).toDouble(),
        heightCm: (map['heightCm'] as num).toDouble(),
        activityLevel: map['activityLevel'] as String,
        createdAt: DateTime.parse(map['createdAt'] as String),
        goals: UserGoals.fromMap(map['goals'] as Map<String, dynamic>),
      );

  UserModel copyWith({
    String? displayName,
    int? age,
    double? weightKg,
    double? heightCm,
    String? activityLevel,
    UserGoals? goals,
  }) =>
      UserModel(
        id: id,
        email: email,
        displayName: displayName ?? this.displayName,
        age: age ?? this.age,
        weightKg: weightKg ?? this.weightKg,
        heightCm: heightCm ?? this.heightCm,
        activityLevel: activityLevel ?? this.activityLevel,
        createdAt: createdAt,
        goals: goals ?? this.goals,
      );
}

class UserGoals {
  final int dailySteps;
  final double dailyCalorieBurn;
  final int weeklyWorkouts;
  final double targetWeightKg;
  final String primaryGoal;

  const UserGoals({
    this.dailySteps = 10000,
    this.dailyCalorieBurn = 500,
    this.weeklyWorkouts = 5,
    this.targetWeightKg = 75,
    this.primaryGoal = 'maintain',
  });

  Map<String, dynamic> toMap() => {
        'dailySteps': dailySteps,
        'dailyCalorieBurn': dailyCalorieBurn,
        'weeklyWorkouts': weeklyWorkouts,
        'targetWeightKg': targetWeightKg,
        'primaryGoal': primaryGoal,
      };

  factory UserGoals.fromMap(Map<String, dynamic> map) => UserGoals(
        dailySteps: map['dailySteps'] as int? ?? 10000,
        dailyCalorieBurn: (map['dailyCalorieBurn'] as num?)?.toDouble() ?? 500,
        weeklyWorkouts: map['weeklyWorkouts'] as int? ?? 5,
        targetWeightKg: (map['targetWeightKg'] as num?)?.toDouble() ?? 75,
        primaryGoal: map['primaryGoal'] as String? ?? 'maintain',
      );
}

class AppSettings {
  final bool useMetric;
  final bool notificationsEnabled;
  final String dailyReminderTime;

  const AppSettings({
    this.useMetric = true,
    this.notificationsEnabled = true,
    this.dailyReminderTime = '08:00',
  });

  Map<String, dynamic> toMap() => {
        'useMetric': useMetric,
        'notificationsEnabled': notificationsEnabled,
        'dailyReminderTime': dailyReminderTime,
      };

  factory AppSettings.fromMap(Map<String, dynamic> map) => AppSettings(
        useMetric: map['useMetric'] as bool? ?? true,
        notificationsEnabled: map['notificationsEnabled'] as bool? ?? true,
        dailyReminderTime: map['dailyReminderTime'] as String? ?? '08:00',
      );
}
