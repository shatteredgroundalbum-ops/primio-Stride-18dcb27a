import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';
import '../common/progress_ring.dart';

class StepsCard extends StatelessWidget {
  final int steps;
  final int goal;
  final double caloriesBurned;
  final int activeMinutes;
  final double activeProgress;

  const StepsCard({
    super.key,
    required this.steps,
    required this.goal,
    required this.caloriesBurned,
    required this.activeMinutes,
    required this.activeProgress,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final progress = (steps / goal).clamp(0.0, 1.0);

    return GlassCard(
      child: Column(
        children: [
          Row(
            children: [
              Icon(Icons.directions_walk, size: AppTheme.iconSm, color: appColors.stepsAccent),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Daily Activity', style: text.titleSmall),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          Row(
            children: [
              ProgressRing(
                progress: progress,
                size: 100,
                strokeWidth: 8,
                activeColor: appColors.stepsAccent,
                center: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      _formatSteps(steps),
                      style: text.titleLarge?.copyWith(color: appColors.stepsAccent),
                    ),
                    Text(
                      'of ${_formatSteps(goal)}',
                      style: text.labelSmall,
                    ),
                  ],
                ),
              ),
              const SizedBox(width: AppTheme.spacingMd),
              Expanded(
                child: Column(
                  children: [
                    _MetricRow(
                      icon: Icons.local_fire_department,
                      label: 'Calories',
                      value: '${caloriesBurned.toInt()} kcal',
                      color: appColors.exerciseAccent,
                    ),
                    const SizedBox(height: AppTheme.spacingSm),
                    _MetricRow(
                      icon: Icons.timer,
                      label: 'Active',
                      value: '$activeMinutes min',
                      color: colors.primary,
                    ),
                    const SizedBox(height: AppTheme.spacingSm),
                    ClipRRect(
                      borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                      child: LinearProgressIndicator(
                        value: activeProgress,
                        minHeight: 6,
                        backgroundColor: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                        valueColor: AlwaysStoppedAnimation(colors.primary),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  String _formatSteps(int value) {
    if (value >= 1000) {
      return '${(value / 1000).toStringAsFixed(1)}k';
    }
    return value.toString();
  }
}

class _MetricRow extends StatelessWidget {
  final IconData icon;
  final String label;
  final String value;
  final Color color;

  const _MetricRow({
    required this.icon,
    required this.label,
    required this.value,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;

    return Row(
      children: [
        Icon(icon, size: AppTheme.iconSm, color: color),
        const SizedBox(width: AppTheme.spacingSm),
        Expanded(child: Text(label, style: text.labelMedium)),
        Text(value, style: text.titleSmall),
      ],
    );
  }
}
