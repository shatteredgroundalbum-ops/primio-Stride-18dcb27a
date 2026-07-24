import 'package:flutter/material.dart';
import '../widgets/common/placeholder_screen.dart';

class ProfileScreen extends StatelessWidget {
  const ProfileScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return PlaceholderScreen(
      title: 'Profile',
      icon: Icons.person,
      accentColor: Theme.of(context).colorScheme.primary,
      description: 'Your goals, progress reports, body measurements, and account settings.',
    );
  }
}
