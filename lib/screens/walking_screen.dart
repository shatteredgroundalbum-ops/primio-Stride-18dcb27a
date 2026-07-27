import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:provider/provider.dart';

import '../models/walking_session.dart';
import '../providers/walking_provider.dart';
import '../theme/theme.dart';
import '../widgets/common/section_header.dart';
import '../widgets/walking/live_session_card.dart';
import '../widgets/walking/session_list_card.dart';
import '../widgets/walking/walking_stats_summary.dart';

/// Walking / Running / Jogging screen — the home of all
/// walking/running/hiking tracking in the app. Shows a live session
/// card (when a workout is in progress), a weekly stats summary, and
/// a session history list of completed sessions read from the local
/// SQLite cache. All data is real (from the Rust engine via
/// WorkoutRecorder and LocalDatabase); no mock or hardcoded values.
///
/// When the user has no sessions yet, the screen renders an inviting
/// empty state with a "Start your first session" call-to-action.
class WalkingScreen extends StatefulWidget {
  const WalkingScreen({super.key});

  @override
  State<WalkingScreen> createState() => _WalkingScreenState();
}

class _WalkingScreenState extends State<WalkingScreen> {
  SessionType _selectedType = SessionType.walk;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final provider = context.read<WalkingProvider>();
      provider.loadSessions();
      provider.subscribeToLiveSession();
    });
  }

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<WalkingProvider>();
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final text = Theme.of(context).textTheme;

    return Scaffold(
      body: SafeArea(
        child: RefreshIndicator(
          onRefresh: provider.loadSessions,
          color: appColors.stepsAccent,
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              AppTheme.spacingMd,
              AppTheme.spacingSm,
              AppTheme.spacingMd,
              AppTheme.spacingLg,
            ),
            children: [
              // ── Title ─────────────────────────────────────────────
              Text(
                'Walking & Running',
                style: text.headlineMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: AppTheme.spacingLg),

              // ── Live session card ──────────────────────────────────
              LiveSessionCard(
                activeSession: provider.activeSession,
                isRecording: provider.isRecording,
                isPaused: provider.activeSession?.status ==
                    SessionStatus.paused,
                distanceKm:
                    provider.activeSession?.distanceKm ?? 0,
                duration:
                    provider.activeSession?.duration ?? Duration.zero,
                paceMinPerKm:
                    provider.activeSession?.averagePaceMinPerKm ?? 0,
                calories:
                    provider.activeSession?.caloriesBurned ?? 0,
                steps: provider.activeSession?.stepCount ?? 0,
                selectedType: _selectedType,
                onTypeSelected: (type) {
                  setState(() => _selectedType = type);
                },
                onStart: () => provider.startWorkout(type: _selectedType),
                onPause: provider.pauseWorkout,
                onResume: provider.resumeWorkout,
                onStop: () => provider.stopWorkout,
                onCancel: provider.cancelWorkout,
              ).animate().fadeIn(duration: 400.ms).slideY(
                    begin: -0.1, end: 0),

              const SizedBox(height: AppTheme.spacingLg),

              // ── Weekly stats summary ──────────────────────────────
              WalkingStatsSummary(
                totalDistanceKm: provider.totalDistanceKm,
                totalDuration: provider.totalDuration,
                totalSteps: provider.totalSteps,
                totalCalories: provider.totalCalories,
                thisWeekDistanceKm: provider.thisWeekDistanceKm,
              ).animate().fadeIn(duration: 500.ms, delay: 100.ms).slideY(
                    begin: 0.05, end: 0),

              const SizedBox(height: AppTheme.spacingLg),

              // ── Session history ───────────────────────────────────
              SessionListCard(
                sessions: provider.sessions,
              ).animate().fadeIn(duration: 500.ms, delay: 200.ms).slideY(
                    begin: 0.05, end: 0),

              const SizedBox(height: AppTheme.spacingLg),

              // ── Empty-state call-to-action (only when no data) ────
              if (provider.sessions.isEmpty && !provider.isRecording)
                _buildEmptyState(context, appColors, text)
                    .animate()
                    .fadeIn(duration: 600.ms, delay: 300.ms),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildEmptyState(
    BuildContext context,
    AppColorsExtension appColors,
    TextTheme text,
  ) {
    return Container(
      padding: const EdgeInsets.all(AppTheme.spacingXl),
      child: Column(
        children: [
          Container(
            width: 96,
            height: 96,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              gradient: LinearGradient(
                colors: [
                  appColors.stepsAccent
                      .withOpacity(AppTheme.opacityLight),
                  appColors.stepsAccent
                      .withOpacity(AppTheme.opacitySubtle),
                ],
              ),
            ),
            child: Icon(
              Icons.directions_walk,
              size: AppTheme.iconXl,
              color: appColors.stepsAccent,
            ),
          ),
          const SizedBox(height: AppTheme.spacingLg),
          Text(
            'Start Your Journey',
            style: text.headlineSmall,
          ),
          const SizedBox(height: AppTheme.spacingSm),
          Text(
            'Choose Walk, Run, or Hike above and tap the start '
            'button to begin tracking with GPS and the Rust '
            'stride engine. Your sessions will be saved here.',
            style: text.bodyMedium?.copyWith(
              color: appColors.subtleText,
            ),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}
