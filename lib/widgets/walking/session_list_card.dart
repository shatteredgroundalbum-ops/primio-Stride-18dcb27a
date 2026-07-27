import 'package:flutter/material.dart';

import '../../models/walking_session.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';
import '../common/section_header.dart';

/// A scrollable list of past walking/running/hiking sessions read
/// from the local SQLite cache. Shows each session's date, type,
/// distance, duration, pace, calories, and steps. Renders an empty
/// state when the user has no completed sessions yet.
class SessionListCard extends StatelessWidget {
  final List<WalkingSession> sessions;

  const SessionListCard({super.key, required this.sessions});

  String _formatDate(DateTime dt) {
    final now = DateTime.now();
    final today = DateTime(now.year, now.month, now.day);
    final day = DateTime(dt.year, dt.month, dt.day);
    final diff = today.difference(day).inDays;
    if (diff == 0) return 'Today';
    if (diff == 1) return 'Yesterday';
    if (diff < 7) {
      const weekdays = [
        'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'
      ];
      return weekdays[dt.weekday - 1];
    }
    return '${dt.day}/${dt.month}/${dt.year}';
  }

  String _formatDuration(Duration d) {
    final h = d.inHours;
    final m = d.inMinutes.remainder(60);
    if (h > 0) return '${h}h ${m}m';
    if (m > 0) return '${m}m';
    return '${d.inSeconds}s';
  }

  String _typeLabel(SessionType type) {
    switch (type) {
      case SessionType.walk:
        return 'Walk';
      case SessionType.run:
        return 'Run';
      case SessionType.hike:
        return 'Hike';
    }
  }

  IconData _typeIcon(SessionType type) {
    switch (type) {
      case SessionType.walk:
        return Icons.directions_walk;
      case SessionType.run:
        return Icons.directions_run;
      case SessionType.hike:
        return Icons.hiking;
    }
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SectionHeader(
          title: 'Session History',
          icon: Icons.history,
        ),
        if (sessions.isEmpty)
          GlassCard(
            child: Padding(
              padding: const EdgeInsets.symmetric(
                vertical: AppTheme.spacingLg,
              ),
              child: Column(
                children: [
                  Icon(
                    Icons.history,
                    size: AppTheme.iconXl,
                    color: appColors.subtleText
                        .withOpacity(AppTheme.opacityHint),
                  ),
                  const SizedBox(height: AppTheme.spacingMd),
                  Text(
                    'No sessions yet',
                    style: text.titleSmall?.copyWith(
                      color: appColors.subtleText,
                    ),
                  ),
                  const SizedBox(height: AppTheme.spacingXs),
                  Text(
                    'Your completed walks, runs, and hikes will '
                    'appear here.',
                    style: text.bodySmall?.copyWith(
                      color: appColors.subtleText
                          .withOpacity(AppTheme.opacityHint),
                    ),
                    textAlign: TextAlign.center,
                  ),
                ],
              ),
            ),
          )
        else
          ListView.separated(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: sessions.length,
            separatorBuilder: (_, __) =>
                const SizedBox(height: AppTheme.spacingSm),
            itemBuilder: (context, index) {
              final s = sessions[index];
              return GlassCard(
                child: Row(
                  children: [
                    Container(
                      width: 44,
                      height: 44,
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: appColors.stepsAccent
                            .withOpacity(AppTheme.opacityLight),
                      ),
                      child: Icon(
                        _typeIcon(s.type),
                        size: AppTheme.iconMd,
                        color: appColors.stepsAccent,
                      ),
                    ),
                    const SizedBox(width: AppTheme.spacingMd),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Text(
                                _typeLabel(s.type),
                                style: text.bodyMedium?.copyWith(
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                              const SizedBox(width: AppTheme.spacingSm),
                              Text(
                                _formatDate(s.startTime),
                                style: text.bodySmall?.copyWith(
                                  color: appColors.subtleText,
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: AppTheme.spacingXs),
                          Text(
                            '${s.distanceKm.toStringAsFixed(2)} km  ·  '
                            '${_formatDuration(s.duration)}  ·  '
                            '${s.caloriesBurned.toStringAsFixed(0)} cal',
                            style: text.bodySmall?.copyWith(
                              color: appColors.subtleText,
                            ),
                          ),
                        ],
                      ),
                    ),
                    if (s.stepCount > 0)
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.end,
                        children: [
                          Text(
                            s.stepCount.toString(),
                            style: text.bodyMedium?.copyWith(
                              fontWeight: FontWeight.bold,
                              color: appColors.stepsAccent,
                            ),
                          ),
                          Text(
                            'steps',
                            style: text.labelSmall?.copyWith(
                              color: appColors.subtleText,
                            ),
                          ),
                        ],
                      ),
                  ],
                ),
              );
            },
          ),
      ],
    );
  }
}
