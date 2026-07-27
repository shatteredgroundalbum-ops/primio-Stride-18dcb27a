import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../database/local_database.dart';
import '../providers/auth_provider.dart';
import '../providers/dashboard_provider.dart';
import '../providers/exercise_provider.dart';
import '../providers/fasting_provider.dart';
import '../providers/nutrition_provider.dart';
import '../providers/profile_provider.dart';
import '../providers/walking_provider.dart';
import '../screens/auth/forgot_password_screen.dart';
import '../screens/auth/login_screen.dart';
import '../screens/auth/register_screen.dart';
import '../screens/dashboard_screen.dart';
import '../screens/exercise_screen.dart';
import '../screens/fasting_screen.dart';
import '../screens/nutrition_screen.dart';
import '../screens/profile_screen.dart';
import '../screens/walking_screen.dart';
import '../services/exercise_service.dart';
import '../services/fasting_service.dart';
import '../services/health_service.dart';
import '../services/nutrition_service.dart';
import '../services/profile_service.dart';
import '../services/workout_recorder.dart';
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
                  final userId =
                      context.read<AuthProvider>().currentUser?.id ??
                      'local-user';
                  return ChangeNotifierProvider(
                    create: (_) => DashboardProvider(
                      service: service,
                      userId: userId,
                    )..loadDashboard(),
                    child: const DashboardScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/walking',
                builder: (context, state) {
                  final localDb = context.read<LocalDatabase>();
                  final recorder = context.read<WorkoutRecorder>();
                  final userId =
                      context.read<AuthProvider>().currentUser?.id ??
                      'local-user';
                  return ChangeNotifierProvider(
                    create: (_) => WalkingProvider(
                      localDb: localDb,
                      recorder: recorder,
                      userId: userId,
                    ),
                    child: const WalkingScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/exercise',
                builder: (context, state) {
                  final service = context.read<ExerciseService>();
                  return ChangeNotifierProvider(
                    create: (_) =>
                        ExerciseProvider(service: service)..loadExerciseData(),
                    child: const ExerciseScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/nutrition',
                builder: (context, state) {
                  final service = context.read<NutritionService>();
                  return ChangeNotifierProvider(
                    create: (_) => NutritionProvider(service: service)
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
                  final service = context.read<FastingService>();
                  return ChangeNotifierProvider(
                    create: (_) =>
                        FastingProvider(service: service)..loadFastingData(),
                    child: const FastingScreen(),
                  );
                },
              ),
            ]),
            StatefulShellBranch(routes: [
              GoRoute(
                path: '/profile',
                builder: (context, state) {
                  final service = context.read<ProfileService>();
                  final user = context.read<AuthProvider>().currentUser;
                  return ChangeNotifierProvider(
                    create: (_) => ProfileProvider(service: service)
                      ..loadProfile(user),
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
