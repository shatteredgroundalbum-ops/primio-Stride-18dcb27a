import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../../theme/theme.dart';

class AppShell extends StatelessWidget {
  final StatefulNavigationShell navigationShell;

  const AppShell({super.key, required this.navigationShell});

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Scaffold(
      body: navigationShell,
      bottomNavigationBar: Container(
        decoration: BoxDecoration(
          border: Border(
            top: BorderSide(
              color: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
            ),
          ),
        ),
        child: NavigationBar(
          selectedIndex: navigationShell.currentIndex,
          onDestinationSelected: (index) => navigationShell.goBranch(index),
          destinations: [
            NavigationDestination(
              icon: const Icon(Icons.dashboard_outlined),
              selectedIcon: Icon(Icons.dashboard_rounded, color: colors.primary),
              label: 'Dashboard',
            ),
            NavigationDestination(
              icon: const Icon(Icons.fitness_center_outlined),
              selectedIcon: Icon(Icons.fitness_center, color: appColors.exerciseAccent),
              label: 'Exercise',
            ),
            NavigationDestination(
              icon: const Icon(Icons.restaurant_outlined),
              selectedIcon: Icon(Icons.restaurant, color: appColors.nutritionAccent),
              label: 'Nutrition',
            ),
            NavigationDestination(
              icon: const Icon(Icons.timer_outlined),
              selectedIcon: Icon(Icons.timer, color: appColors.fastingActive),
              label: 'Fasting',
            ),
            NavigationDestination(
              icon: const Icon(Icons.person_outline),
              selectedIcon: Icon(Icons.person, color: colors.primary),
              label: 'Profile',
            ),
          ],
        ),
      ),
    );
  }
}
