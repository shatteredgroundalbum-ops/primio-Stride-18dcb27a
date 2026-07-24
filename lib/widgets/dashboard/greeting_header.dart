import 'package:flutter/material.dart';
import '../../theme/theme.dart';

class GreetingHeader extends StatelessWidget {
  final double recoveryScore;

  const GreetingHeader({super.key, required this.recoveryScore});

  String get _greeting {
    final hour = DateTime.now().hour;
    if (hour < 12) return 'Good Morning';
    if (hour < 17) return 'Good Afternoon';
    return 'Good Evening';
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Row(
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                _greeting,
                style: text.bodyMedium?.copyWith(
                  color: colors.onSurface.withOpacity(AppTheme.opacityHint),
                ),
              ),
              const SizedBox(height: AppTheme.spacingXs),
              Text('Let\'s crush today! 💪', style: text.headlineLarge),
            ],
          ),
        ),
        Container(
          padding: const EdgeInsets.symmetric(
            horizontal: AppTheme.spacingSm + 4,
            vertical: AppTheme.spacingSm,
          ),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(AppTheme.radiusLarge),
            color: appColors.recoveryAccent.withOpacity(AppTheme.opacityLight),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.favorite, size: AppTheme.iconSm, color: appColors.recoveryAccent),
              const SizedBox(width: AppTheme.spacingXs),
              Text(
                '${recoveryScore.toInt()}%',
                style: text.titleSmall?.copyWith(color: appColors.recoveryAccent),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
