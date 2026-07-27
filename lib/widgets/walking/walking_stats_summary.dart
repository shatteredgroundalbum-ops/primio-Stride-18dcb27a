import 'package:flutter/material.dart';

import '../../models/walking_session.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';
import '../common/progress_ring.dart';
import '../common/section_header.dart';

/// A weekly stats summary for the Walking screen. Shows the total
/// distance, duration, steps, and calories across real sessions.
/// All values come from [WalkingProvider] and are zero when the user
/// has no sessions yet.
class WalkingStatsSummary extends StatelessWidget {
  final double totalDistanceKm;
  final Duration totalDuration;
  final int totalSteps;
  final double totalCalories;
  final double thisWeekDistanceKm;
  final double weeklyGoalKm;

  const WalkingStatsSummary({
    super.key,
    required this.totalDistanceKm,
    required this.totalDuration,
    required this.totalSteps,
    required this.totalCalories,
    required this.thisWeekDistanceKm,
    this.weeklyGoalKm = 25,
  });

  String _formatDuration(Duration d) {
    final h = d.inHours;
    final m = d.inMinutes.remainder(60);
    if (h > 0) return '${h}h ${m}m';
    return '${m}m';
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final weeklyProgress =
        weeklyGoalKm > 0 ? (thisWeekDistanceKm / weeklyGoalKm).clamp(0.0, 1.0) : 0.0;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SectionHeader(
          title: 'This Week',
          icon: Icons.directions_walk,
        ),
        GlassCard(
          child: Row(
            children: [
              ProgressRing(
                progress: weeklyProgress,
                size: 96,
                activeColor: appColors.stepsAccent,
                center: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      thisWeekDistanceKm.toStringAsFixed(1),
                      style: text.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    Text(
                      'km',
                      style: text.labelSmall?.copyWith(
                        color: appColors.subtleText,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: AppTheme.spacingLg),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _statRow(
                      context,
                      icon: Icons.straighten,
                      label: 'Total',
                      value: '${totalDistanceKm.toStringAsFixed(1)} km',
                    ),
                    const SizedBox(height: AppTheme.spacingSm),
                    _statRow(
                      context,
                      icon: Icons.timer,
                      label: 'Time',
                      value: _formatDuration(totalDuration),
                    ),
                    const SizedBox(height: AppTheme.spacingSm),
                    _statRow(
                      context,
                      icon: Icons.footprint,
                      label: 'Steps',
                      value: totalSteps.toString(),
                    ),
                    const SizedBox(height: AppTheme.spacingSm),
                    _statRow(
                      context,
                      icon: Icons.local_fire_department,
                      label: 'Calories',
                      value: totalCalories.toStringAsFixed(0),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _statRow(
    BuildContext context, {
    required IconData icon,
    required String label,
    required String value,
  }) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    return Row(
      children: [
        Icon(icon, size: AppTheme.iconSm, color: appColors.subtleText),
        const SizedBox(width: AppTheme.spacingSm),
        Text(
          label,
          style: text.bodySmall?.copyWith(color: appColors.subtleText),
        ),
        const Spacer(),
        Text(value, style: text.bodyMedium?.copyWith(
          fontWeight: FontWeight.w600,
        )),
      ],
    );
  }
}
