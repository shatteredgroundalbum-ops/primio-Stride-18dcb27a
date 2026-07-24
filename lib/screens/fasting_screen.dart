import 'package:flutter/material.dart';
import '../theme/theme.dart';
import '../widgets/common/placeholder_screen.dart';

class FastingScreen extends StatelessWidget {
  const FastingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    return PlaceholderScreen(
      title: 'Fasting',
      icon: Icons.timer,
      accentColor: appColors.fastingActive,
      description: 'Intermittent fasting management with smart timers and personalized protocol recommendations.',
    );
  }
}
