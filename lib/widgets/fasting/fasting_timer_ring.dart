import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../common/progress_ring.dart';

class FastingTimerRing extends StatelessWidget {
  final double progress;
  final Duration elapsed;
  final Duration remaining;
  final String protocol;
  final bool isActive;

  const FastingTimerRing({
    super.key,
    required this.progress,
    required this.elapsed,
    required this.remaining,
    required this.protocol,
    required this.isActive,
  });

  String _formatDuration(Duration d) {
    final h = d.inHours;
    final m = d.inMinutes.remainder(60);
    return '${h}h ${m}m';
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Column(
      children: [
        ProgressRing(
          progress: progress,
          size: 200,
          strokeWidth: 12,
          activeColor: isActive ? appColors.fastingActive : colors.primary,
          center: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                isActive ? 'FASTING' : 'READY',
                style: text.labelMedium?.copyWith(
                  color: isActive ? appColors.fastingActive : colors.primary,
                  fontWeight: FontWeight.w700,
                  letterSpacing: 2,
                ),
              ),
              const SizedBox(height: AppTheme.spacingXs),
              Text(
                _formatDuration(isActive ? elapsed : Duration.zero),
                style: text.headlineLarge?.copyWith(
                  color: isActive ? appColors.fastingActive : colors.onSurface,
                ),
              ),
              const SizedBox(height: AppTheme.spacingXs),
              Text(
                isActive ? '${_formatDuration(remaining)} left' : protocol,
                style: text.bodySmall,
              ),
            ],
          ),
        ),
        const SizedBox(height: AppTheme.spacingLg),
        if (isActive) ...[
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              _TimerStat(label: 'Elapsed', value: _formatDuration(elapsed), color: appColors.fastingActive),
              Container(
                width: AppTheme.borderDefault,
                height: 40,
                color: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
              ),
              _TimerStat(label: 'Remaining', value: _formatDuration(remaining), color: appColors.subtleText),
              Container(
                width: AppTheme.borderDefault,
                height: 40,
                color: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
              ),
              _TimerStat(label: 'Progress', value: '${(progress * 100).toInt()}%', color: appColors.success),
            ],
          ),
        ],
      ],
    );
  }
}

class _TimerStat extends StatelessWidget {
  final String label;
  final String value;
  final Color color;

  const _TimerStat({required this.label, required this.value, required this.color});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    return Column(
      children: [
        Text(value, style: text.titleSmall?.copyWith(color: color)),
        const SizedBox(height: AppTheme.spacingXs),
        Text(label, style: text.labelSmall),
      ],
    );
  }
}
