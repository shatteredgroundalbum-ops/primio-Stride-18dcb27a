import 'package:flutter/material.dart';
import '../theme/theme.dart';
import '../widgets/common/placeholder_screen.dart';

class ExerciseScreen extends StatelessWidget {
  const ExerciseScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    return PlaceholderScreen(
      title: 'Exercise',
      icon: Icons.fitness_center,
      accentColor: appColors.exerciseAccent,
      description: 'Weightlifting programs, workout tracking, and exercise recommendations powered by AI.',
    );
  }
}
