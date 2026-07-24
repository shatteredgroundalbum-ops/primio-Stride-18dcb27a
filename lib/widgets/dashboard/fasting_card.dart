import 'package:flutter/material.dart';
import '../../models/health_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';
import '../common/progress_ring.dart';

class FastingCard extends StatelessWidget {
  final FastingState fasting;

  const FastingCard({super.key, required this.fasting});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return GlassCard(
      borderColor: fasting.isActive
          ? appColors.fastingActive.withOpacity(AppTheme.opacityLight)
          : null,
      child: Column(
        children: [
          Row(
            children: [
              Icon(Icons.timer, size: AppTheme.iconSm, color: appColors.fastingActive),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Intermittent Fasting', style: text.titleSmall),
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppTheme.spacingSm,
                  vertical: AppTheme.spacingXs,
                ),
                decoration: BoxDecoration(
                  color: fasting.isActive
                      ? appColors.fastingActive.withOpacity(AppTheme.opacityLight)
                      : appColors.success.withOpacity(AppTheme.opacityLight),
                  borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                ),
                child: Text(
                  fasting.isActive ? 'Fasting' : 'Eating Window',
                  style: text.labelSmall?.copyWith(
                    color: fasting.isActive ? appColors.fastingActive : appColors.success,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          Row(
            children: [
              ProgressRing(
                progress: fasting.progress,
                size: 80,
                strokeWidth: 7,
                activeColor: appColors.fastingActive,
                center: Text(
                  fasting.protocol,
                  style: text.titleSmall?.copyWith(color: appColors.fastingActive),
                ),
              ),
              const SizedBox(width: AppTheme.spacingMd),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Elapsed',
                      style: text.labelSmall,
                    ),
                    Text(fasting.elapsedFormatted, style: text.titleMedium),
                    const SizedBox(height: AppTheme.spacingSm),
                    Text(
                      'Remaining',
                      style: text.labelSmall,
                    ),
                    Text(
                      fasting.remainingFormatted,
                      style: text.titleMedium?.copyWith(
                        color: colors.onSurface.withOpacity(AppTheme.opacityHint),
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
}
