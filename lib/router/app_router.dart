import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../providers/auth_provider.dart';
import '../providers/dashboard_provider.dart';
import '../providers/exercise_provider.dart';
import '../providers/fasting_provider.dart';
import '../providers/nutrition_provider.dart';
import '../providers/profile_provider.dart';
import '../screens/auth/forgot_password_screen.dart';
import '../screens/auth/login_screen.dart';
import '../screens/auth/register_screen.dart';
import '../screens/dashboard_screen.dart';
import '../screens/exercise_screen.dart';
import '../screens/fasting_screen.dart';
import '../screens/nutrition_screen.dart';
import '../screens/profile_screen.dart';
import '../services/exercise_service.dart';
import '../services/fasting_service.dart';
import '../services/health_service.dart';
import '../services/nutrition_service.dart';
import '../services/profile_service.dart';
import '../widgets/common/app_shell.dart';

class AppRouter {
  static final _rootNavigatorKey = GlobalKey<NavigatorState>();

  static GoRouter createRouter(AuthProvider authProvider) {
    return GoRouter(
      navigatorKey: _rootNavigatorKey,
      initialLocation: '/dashboard',
      refreshListenable: authProvider,
      redirect: (context, state) {
        final isAuthenticated = authProvider.isAuthenticated;
        final isAuthRoute = state.matchedLocation.startsWith('/auth');

        if (!isAuthenticated && !isAuthRoute) return '/auth/login';
        if (isAuthenticated && isAuthRoute) return '/dashboard';
        return null;
      },
      routes: [
        GoRoute(
          path: '/auth/login',
          builder: (context, state) => const LoginScreen(),
        ),
        GoRoute(
          path: '/auth/register',
          builder: (context, state) => const RegisterScreen(),
        ),
        GoRoute(
          path: '/auth/forgot-password',
          builder: (context, state) => const ForgotPasswordScreen(),
        ),
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
                    create: (_) =>
                        DashboardProvider(service: service)..loadDashboard(),
                    child: const DashboardScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/exercise',
                builder: (context, state) {
                  return ChangeNotifierProvider(
                    create: (_) => ExerciseProvider(
                        service: ExerciseService())
                      ..loadExerciseData(),
                    child: const ExerciseScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/nutrition',
                builder: (context, state) {
                  return ChangeNotifierProvider(
                    create: (_) => NutritionProvider(
                        service: NutritionService())
                      ..loadNutritionData(),
                    child: const NutritionScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/fasting',
                builder: (context, state) {
                  return ChangeNotifierProvider(
                    create: (_) =>
                        FastingProvider(service: FastingService())
                          ..loadFastingData(),
                    child: const FastingScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/profile',
                builder: (context, state) {
                  return ChangeNotifierProvider(
                    create: (_) =>
                        ProfileProvider(service: ProfileService())
                          ..loadProfile(),
                    child: const ProfileScreen(),
                  );
                },
              ),
            ]),
          ],
        ),
      ],
    );
  }
}
