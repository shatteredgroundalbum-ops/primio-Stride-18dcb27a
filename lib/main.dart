import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'database/local_database.dart';
import 'providers/auth_provider.dart';
import 'repositories/auth_repository.dart';
import 'repositories/cloud/cloud_plan_repository.dart';
import 'repositories/cloud/cloud_route_repository.dart';
import 'repositories/cloud/cloud_summary_repository.dart';
import 'repositories/cloud/cloud_user_repository.dart';
import 'repositories/cloud/cloud_workout_repository.dart';
import 'router/app_router.dart';
import 'services/auth_service.dart';
import 'services/exercise_service.dart';
import 'services/fasting_service.dart';
import 'services/gps_service.dart';
import 'services/health_service.dart';
import 'services/nutrition_service.dart';
import 'services/preferences_service.dart';
import 'services/profile_service.dart';
import 'services/recovery_service.dart';
import 'services/stride_notification_service.dart';
import 'services/sync_service.dart';
import 'services/workout_recorder.dart';
import 'theme/theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await StrideNotificationService.initialize();

  // ── Storage layer ────────────────────────────────────────────────
  final localDb = LocalDatabase();
  final prefs = PreferencesService();

  // ── Cloud repositories (local-backed; swap for Firestore when connected) ──
  final cloudUserRepo = LocalCloudUserRepository();
  final cloudWorkoutRepo = LocalCloudWorkoutRepository(localDb: localDb);
  final cloudRouteRepo = LocalCloudRouteRepository();
  final cloudSummaryRepo = LocalCloudSummaryRepository(localDb: localDb);
  final cloudPlanRepo = LocalCloudPlanRepository();

  // ── Auth ─────────────────────────────────────────────────────────
  final authRepository = LocalAuthRepository();
  final authService = AuthService(
    authRepository: authRepository,
    cloudUserRepository: cloudUserRepo,
  );
  final authProvider = AuthProvider(authService: authService);

  // ── GPS + Workout Recorder ───────────────────────────────────────
  final gpsService = GpsService();
  final workoutRecorder = WorkoutRecorder(
    localDb: localDb,
    gpsService: gpsService,
  );

  // ── Sync engine (local SQLite → Firestore + Cloud Storage) ───────
  final syncService = SyncService(
    localDb: localDb,
    workoutRepo: cloudWorkoutRepo,
    routeRepo: cloudRouteRepo,
    summaryRepo: cloudSummaryRepo,
    prefs: prefs,
  );
  syncService.startPeriodicSync();

  // ── Crash recovery (spec section 23) ──────────────────────────────
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

  // ── App services (empty-state; no mock data) ─────────────────────
  final healthService = HealthService(localDb: localDb);
  final exerciseService = ExerciseService();
  final nutritionService = NutritionService();
  final fastingService = FastingService();
  final profileService = ProfileService();

  runApp(StrideApp(
    authProvider: authProvider,
    localDb: localDb,
    prefs: prefs,
    workoutRecorder: workoutRecorder,
    syncService: syncService,
    cloudWorkoutRepo: cloudWorkoutRepo,
    cloudRouteRepo: cloudRouteRepo,
    cloudSummaryRepo: cloudSummaryRepo,
    cloudPlanRepo: cloudPlanRepo,
    healthService: healthService,
    exerciseService: exerciseService,
    nutritionService: nutritionService,
    fastingService: fastingService,
    profileService: profileService,
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
  final CloudPlanRepository cloudPlanRepo;
  final HealthService healthService;
  final ExerciseService exerciseService;
  final NutritionService nutritionService;
  final FastingService fastingService;
  final ProfileService profileService;

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
    required this.cloudPlanRepo,
    required this.healthService,
    required this.exerciseService,
    required this.nutritionService,
    required this.fastingService,
    required this.profileService,
  });

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider.value(value: authProvider),
        Provider<LocalDatabase>.value(value: localDb),
        Provider<PreferencesService>.value(value: prefs),
        Provider<WorkoutRecorder>.value(value: workoutRecorder),
        Provider<SyncService>.value(value: syncService),
        Provider<CloudWorkoutRepository>.value(value: cloudWorkoutRepo),
        Provider<CloudRouteRepository>.value(value: cloudRouteRepo),
        Provider<CloudSummaryRepository>.value(value: cloudSummaryRepo),
        Provider<CloudPlanRepository>.value(value: cloudPlanRepo),
        Provider<HealthService>.value(value: healthService),
        Provider<ExerciseService>.value(value: exerciseService),
        Provider<NutritionService>.value(value: nutritionService),
        Provider<FastingService>.value(value: fastingService),
        Provider<ProfileService>.value(value: profileService),
      ],
      child: MaterialApp.router(
        title: 'S.T.R.I.D.E.',
        theme: AppTheme.darkTheme,
        routerConfig: AppRouter.createRouter(authProvider),
      ),
    );
  }
}
