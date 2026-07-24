# S.T.R.I.D.E.

## Overview
A walking/running intelligence platform combining GPS tracking, personalized training plans, step tracking, intermittent fasting, nutrition, and recovery into one adaptive experience. Features auth flow, repository layer, GPS service, AI training engine, and analytics — all client-side with Firestore-ready models.

## Tech Stack & Key Decisions
- Dark theme with gradient accents; reduces eye strain during workouts
- fl_chart for charts, flutter_animate for entrance animations
- geolocator for GPS position tracking with haversine distance calculations
- awesome_notifications for workout reminders and achievement alerts
- ChangeNotifier + Provider for state — route-scoped per tab, global for auth
- go_router with refreshListenable auth gate — unauthenticated users redirect to login
- All models have toMap/fromMap for Firestore readiness; currently backed by in-memory repositories

## Architecture
- Layered: models → repositories → services → providers → screens + widgets
- AuthProvider is app-global (created in main.dart, passed to GoRouter as refreshListenable)
- Tab providers remain route-scoped in GoRoute builders
- Repositories use in-memory storage — designed as 1:1 swap points for Cloud Firestore
- GpsService uses geolocator streams; static methods for distance/pace/calorie math
- TrainingEngine generates progressive walking plans using MET-based calorie estimation and Mifflin-St Jeor BMR

## Conventions
- Auth screens live in `screens/auth/`; auth widgets in `widgets/auth/`
- Repository classes: one per Firestore collection, CRUD methods returning Futures
- Models: constructors, toMap, fromMap factory, copyWith where mutation is needed
- Services never import Flutter (except GpsService which needs `kIsWeb`); providers never import repositories directly
- GlassCard, SectionHeader, ProgressRing remain shared primitives

## Key Patterns & Gotchas
- MockAuthRepository accepts any email with password ≥ 6 chars — swap to FirebaseAuthRepository for production
- GoRouter redirect checks auth on every navigation; refreshListenable triggers on logout/login
- GpsService is disabled on web (kIsWeb guard) — GPS tracking is mobile-only
- TrainingEngine uses progressive overload (~10% weekly increase) with hardcoded rest days (Sun) and recovery days (Wed)
- StrideNotificationService.initialize() must be called before runApp in main()

## Design System
- Bold dark fitness aesthetic: near-black surface with vibrant category accents
- Inter font via google_fonts; 16px spacing grid
- Auth screens use GlassCard containers with gradient logo circle
- All colors from colorScheme/AppColorsExtension, all text from textTheme, all dims from AppTheme tokens
