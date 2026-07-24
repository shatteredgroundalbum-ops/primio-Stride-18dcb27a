import 'package:flutter/material.dart';
import '../../models/fasting_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class FastingStatsCard extends StatelessWidget {
  final FastingStats stats;

  const FastingStatsCard({super.key, required this.stats});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return GlassCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.insights, size: AppTheme.iconSm, color: appColors.fastingActive),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Fasting Stats', style: text.titleSmall),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          Row(
            children: [
              Expanded(child: _StatTile(label: 'Total Fasts', value: '${stats.totalFasts}', color: appColors.fastingActive)),
              Expanded(child: _StatTile(label: 'Completed', value: '${stats.completedFasts}', color: appColors.success)),
              Expanded(child: _StatTile(label: 'Success', value: '${(stats.completionRate * 100).toInt()}%', color: appColors.nutritionAccent)),
            ],
          ),
          const SizedBox(height: AppTheme.spacingSm),
          Row(
            children: [
              Expanded(child: _StatTile(label: 'Streak', value: '${stats.currentStreak} days', color: appColors.warning)),
              Expanded(child: _StatTile(label: 'Best Streak', value: '${stats.longestStreak} days', color: appColors.exerciseAccent)),
              Expanded(child: _StatTile(label: 'Avg Duration', value: '${stats.averageDuration.inHours}h ${stats.averageDuration.inMinutes.remainder(60)}m', color: appColors.recoveryAccent)),
            ],
          ),
        ],
      ),
    );
  }
}

class _StatTile extends StatelessWidget {
  final String label;
  final String value;
  final Color color;

  const _StatTile({required this.label, required this.value, required this.color});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;

    return Container(
      padding: const EdgeInsets.all(AppTheme.spacingSm),
      margin: const EdgeInsets.all(2),
      decoration: BoxDecoration(
        color: color.withOpacity(AppTheme.opacitySubtle),
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
      ),
      child: Column(
        children: [
          Text(value, style: text.titleSmall?.copyWith(color: color)),
          const SizedBox(height: AppTheme.spacingXs),
          Text(label, style: text.labelSmall, maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.center),
        ],
      ),
    );
  }
}
