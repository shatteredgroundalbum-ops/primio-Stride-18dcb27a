# S.T.R.I.D.E.

## Overview
A walking/running intelligence platform with GPS tracking, personalized training plans, nutrition, fasting, and recovery. Designed as a production-ready offline-first fitness app with a multi-database backend architecture.

## Tech Stack & Key Decisions
- **Multi-database architecture**: SQLite (offline recording), Firestore (cloud sync), Cloud Storage (route files), SharedPreferences (device settings) — each storage system handles the data it's best at
- Dark theme with gradient accents; fl_chart for charts, flutter_animate for entrance animations
- geolocator for GPS; sqflite for local offline database
- All cloud repositories are abstract interfaces with in-memory mocks — designed as 1:1 Firestore swap points
- go_router with auth gate; ChangeNotifier + Provider for state

## Architecture
- **Data flow**: GPS sensors → SQLite → workout calculations → completed summary → Firestore + Cloud Storage
- **Storage split**: live workout data in SQLite, completed summaries in Firestore, compressed GPS routes in Cloud Storage, small prefs in SharedPreferences
- **Firestore schema**: user-centric — `users/{userId}/workouts/`, `/trainingPlans/`, `/achievements/`, `/savedRoutes/`, `/dailySummaries/`
- **Sync pipeline**: `SyncService` runs every 5 min, processes `pending_uploads` table in SQLite, uploads route files to Cloud Storage, writes summaries to Firestore
- `WorkoutRecorder` orchestrates live recording: start/pause/resume/stop, writes every GPS point to SQLite, recalculates stats every second, queues for sync on completion
- AuthProvider is app-global; tab providers remain route-scoped in GoRoute builders
- Layered: models → repositories → services → providers → screens + widgets

## Conventions
- Cloud repositories live in `repositories/cloud/` — abstract interface + mock implementation per file
- `LocalDatabase` owns all SQLite tables; single instance created in main.dart and provided globally
- `PreferencesService` wraps SharedPreferences with typed getters/setters — no raw string keys in UI code
- GPS route points are NEVER stored in Firestore documents — always compressed and uploaded to Cloud Storage, with a path reference in the workout document
- Models use `toMap`/`fromMap` with Firestore field names (`startedAt`, `activityType`, `caloriesEstimated`)

## Key Patterns & Gotchas
- `WorkoutRecorder` persists running stats to SQLite every second for crash recovery — if app is killed, the active workout can be resumed
- SyncService skips uploads after 5 failed retries to avoid infinite loops
- `GpsService` methods (distance, pace, calories) are all static — used by both the recorder and analytics
- MockCloudUserRepository has a `createDefaultUser` convenience method not on the abstract interface — only used during mock auth flow
- LocalDatabase returns empty/null on web (kIsWeb guard) — all workout recording is mobile-only

## Design System
- Bold dark fitness aesthetic: near-black surface with vibrant category accents
- Inter font via google_fonts; 16px spacing grid
- GlassCard, ProgressRing, SectionHeader are shared primitives in `widgets/common/`
- All colors from colorScheme/AppColorsExtension, text from textTheme, dims from AppTheme tokens
