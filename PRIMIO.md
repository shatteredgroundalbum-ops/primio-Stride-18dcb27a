# S.T.R.I.D.E.

## Overview
A comprehensive personal health intelligence platform that integrates step tracking, exercise programs, intermittent fasting, nutrition/macro tracking, recovery monitoring, and AI-powered recommendations into a unified dashboard. Designed for health-conscious individuals who want one app instead of six.

## Tech Stack & Key Decisions
- Dark theme with gradient accents chosen to match fitness/health app conventions and reduce eye strain during workouts
- fl_chart for weekly steps visualization — lightweight and sufficient for bar charts
- flutter_animate for staggered entrance animations on dashboard cards
- ChangeNotifier + Provider for state — route-scoped; app complexity doesn't warrant BLoC yet
- go_router StatefulShellRoute for bottom navigation with 5 tabs preserving state between switches
- Mock HealthService returns realistic data — designed to be swapped with real APIs (Firebase, HealthKit, etc.)

## Architecture
- Single service layer (HealthService) handles both data retrieval and AI insight generation
- Dashboard is the only fully implemented screen; other tabs are placeholder screens
- Repositories layer omitted intentionally — no persistence yet; HealthService returns mock data directly
- Provider wired at route level inside StatefulShellBranch GoRoute builder
- HealthService provided at app-global level since it's stateless and shared

## Conventions
- All accent colors are semantic: stepsAccent (cyan), exerciseAccent (pink), nutritionAccent (green), fastingActive (orange), recoveryAccent (purple)
- Glass card pattern (GlassCard widget) used as the base container for all dashboard sections
- Section headers use SectionHeader widget with optional icon and action
- Placeholder screens use PlaceholderScreen widget with accent color, icon, and description
- New screens: add GoRoute in StatefulShellBranch, create screen file, wire providers in route builder

## Key Patterns & Gotchas
- FastingState.progress calculates from elapsed/target duration — ensure elapsed never exceeds target for UI sanity
- AI insights are generated synchronously from health data in HealthService.getInsights() — will need async when real Gemini integration added
- Weekly steps array is fixed at 7 elements (Mon-Sun); index 3 is "today" in the mock data
- NavigationBar uses WidgetStateProperty (not MaterialStateProperty) for M3 icon theming

## Design System
- Bold dark fitness aesthetic: near-black surface (#0D0D14) with vibrant category-specific accent colors
- Inter font family throughout for maximum readability on dark backgrounds
- 16px base spacing grid with glass-morphic cards (subtle gradient + border) as primary containers
- Each health domain has a dedicated accent color for instant visual recognition across all UI elements
