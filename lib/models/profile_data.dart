class UserProfile {
  final String name;
  final int age;
  final double weightKg;
  final double heightCm;
  final String goal;
  final String activityLevel;
  final DateTime memberSince;

  const UserProfile({
    required this.name,
    required this.age,
    required this.weightKg,
    required this.heightCm,
    required this.goal,
    required this.activityLevel,
    required this.memberSince,
  });

  double get bmi => weightKg / ((heightCm / 100) * (heightCm / 100));

  String get bmiCategory {
    if (bmi < 18.5) return 'Underweight';
    if (bmi < 25) return 'Normal';
    if (bmi < 30) return 'Overweight';
    return 'Obese';
  }
}

class BodyStats {
  final double weightKg;
  final double bodyFatPercent;
  final double muscleMassKg;
  final List<WeightEntry> weightHistory;

  const BodyStats({
    required this.weightKg,
    required this.bodyFatPercent,
    required this.muscleMassKg,
    required this.weightHistory,
  });
}

class WeightEntry {
  final DateTime date;
  final double weightKg;

  const WeightEntry({required this.date, required this.weightKg});
}

class Achievement {
  final String title;
  final String description;
  final String iconName;
  final bool earned;
  final DateTime? earnedDate;

  const Achievement({
    required this.title,
    required this.description,
    required this.iconName,
    required this.earned,
    this.earnedDate,
  });
}
