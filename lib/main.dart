import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'database/local_database.dart';
import 'providers/auth_provider.dart';
import 'repositories/auth_repository.dart';
import 'repositories/cloud/cloud_route_repository.dart';
import 'repositories/cloud/cloud_summary_repository.dart';
import 'repositories/cloud/cloud_user_repository.dart';
import 'repositories/cloud/cloud_workout_repository.dart';
import 'router/app_router.dart';
import 'services/auth_service.dart';
import 'services/gps_service.dart';
import 'services/health_service.dart';
import 'services/preferences_service.dart';
import 'services/recovery_service.dart';
import 'services/stride_notification_service.dart';
import 'services/sync_service.dart';
import 'services/workout_recorder.dart';
import 'theme/theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await StrideNotificationService.initialize();

  // ── Storage layer ──────────────────────────────────────────────
  final localDb = LocalDatabase();
  final prefs = PreferencesService();

  // ── Cloud repositories (swap mocks for Firestore when connected) ─
  final cloudUserRepo = MockCloudUserRepository();
  final cloudWorkoutRepo = MockCloudWorkoutRepository();
  final cloudRouteRepo = MockCloudRouteRepository();
  final cloudSummaryRepo = MockCloudSummaryRepository();

  // ── Auth ───────────────────────────────────────────────────────
  final authRepository = MockAuthRepository();
  final authService = AuthService(
    authRepository: authRepository,
    cloudUserRepository: cloudUserRepo,
  );
  final authProvider = AuthProvider(authService: authService);

  // ── GPS + Workout Recorder ─────────────────────────────────────
  final gpsService = GpsService();
  final workoutRecorder = WorkoutRecorder(
    localDb: localDb,
    gpsService: gpsService,
  );

  // ── Sync engine (local SQLite → Firestore + Cloud Storage) ────
  final syncService = SyncService(
    localDb: localDb,
    workoutRepo: cloudWorkoutRepo,
    routeRepo: cloudRouteRepo,
    summaryRepo: cloudSummaryRepo,
    prefs: prefs,
  );
  syncService.startPeriodicSync();

  // ── Crash recovery (spec section 23) ─────────────────────────────
  // Must run after workoutRecorder/syncService exist but before runApp,
  // so any recovered "paused" workout is already attached to
  // workoutRecorder by the time the first frame builds and screens
  // start reading WorkoutRecorder.activeSession.
  final recoveryService = RecoveryService(
    localDb: localDb,
    workoutRecorder: workoutRecorder,
    syncService: syncService,
  );
  final currentUserId = authProvider.currentUser?.id ?? 'local-user';
  await recoveryService.run(currentUserId: currentUserId);

  runApp(StrideApp(
    authProvider: authProvider,
    localDb: localDb,
    prefs: prefs,
    workoutRecorder: workoutRecorder,
    syncService: syncService,
    cloudWorkoutRepo: cloudWorkoutRepo,
    cloudRouteRepo: cloudRouteRepo,
    cloudSummaryRepo: cloudSummaryRepo,
  ));
}

class StrideApp extends StatelessWidget {
  final AuthProvider authProvider;
  final LocalDatabase localDb;
  final PreferencesService prefs;
  final WorkoutRecorder workoutRecorder;
  final SyncService syncService;
  final CloudWorkoutRepository cloudWorkoutRepo;
  final CloudRouteRepository cloudRouteRepo;
  final CloudSummaryRepository cloudSummaryRepo;

  const StrideApp({
    super.key,
    required this.authProvider,
    required this.localDb,
    required this.prefs,
    required this.workoutRecorder,
    required this.syncService,
    required this.cloudWorkoutRepo,
    required this.cloudRouteRepo,
    required this.cloudSummaryRepo,
  });

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider.value(value: authProvider),
        Provider(create: (_) => HealthService()),
        Provider<LocalDatabase>.value(value: localDb),
        Provider<PreferencesService>.value(value: prefs),
        Provider<WorkoutRecorder>.value(value: workoutRecorder),
        Provider<SyncService>.value(value: syncService),
        Provider<CloudWorkoutRepository>.value(value: cloudWorkoutRepo),
        Provider<CloudRouteRepository>.value(value: cloudRouteRepo),
        Provider<CloudSummaryRepository>.value(value: cloudSummaryRepo),
      ],
      child: MaterialApp.router(
        title: 'S.T.R.I.D.E.',
        theme: AppTheme.darkTheme,
        routerConfig: AppRouter.createRouter(authProvider),
      ),
    );
  }
}