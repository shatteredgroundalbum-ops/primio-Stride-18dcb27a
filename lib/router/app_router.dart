import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import '../providers/dashboard_provider.dart';
import '../screens/dashboard_screen.dart';
import '../screens/exercise_screen.dart';
import '../screens/fasting_screen.dart';
import '../screens/nutrition_screen.dart';
import '../screens/profile_screen.dart';
import '../services/health_service.dart';
import '../widgets/common/app_shell.dart';

class AppRouter {
  static final _rootNavigatorKey = GlobalKey<NavigatorState>();

  static final router = GoRouter(
    navigatorKey: _rootNavigatorKey,
    initialLocation: '/dashboard',
    routes: [
      StatefulShellRoute.indexedStack(
        builder: (context, state, navigationShell) {
          return AppShell(navigationShell: navigationShell);
        },
        branches: [
          StatefulShellBranch(routes: [
            GoRoute(
              path: '/dashboard',
              builder: (context, state) {
                final service = context.read<HealthService>();
                return ChangeNotifierProvider(
                  create: (_) => DashboardProvider(service: service)..loadDashboard(),
                  child: const DashboardScreen(),
                );
              },
            ),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(
              path: '/exercise',
              builder: (context, state) => const ExerciseScreen(),
            ),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(
              path: '/nutrition',
              builder: (context, state) => const NutritionScreen(),
            ),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(
              path: '/fasting',
              builder: (context, state) => const FastingScreen(),
            ),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(
              path: '/profile',
              builder: (context, state) => const ProfileScreen(),
            ),
          ]),
        ],
      ),
    ],
  );
}
