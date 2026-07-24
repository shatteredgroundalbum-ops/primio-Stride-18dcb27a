import 'package:flutter/material.dart';
import '../../models/profile_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class ProfileHeaderCard extends StatelessWidget {
  final UserProfile profile;

  const ProfileHeaderCard({super.key, required this.profile});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final monthsActive = DateTime.now().difference(profile.memberSince).inDays ~/ 30;

    return GlassCard(
      borderColor: colors.primary.withOpacity(AppTheme.opacityLight),
      child: Row(
        children: [
          Container(
            width: 64,
            height: 64,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              gradient: LinearGradient(
                colors: [colors.primary, appColors.exerciseAccent],
              ),
            ),
            child: Center(
              child: Text(
                profile.name.substring(0, 1).toUpperCase(),
                style: text.headlineMedium?.copyWith(color: colors.onPrimary),
              ),
            ),
          ),
          const SizedBox(width: AppTheme.spacingMd),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(profile.name, style: text.titleLarge),
                const SizedBox(height: AppTheme.spacingXs),
                Row(
                  children: [
                    _InfoChip(label: profile.goal, color: colors.primary),
                    const SizedBox(width: AppTheme.spacingSm),
                    _InfoChip(label: '$monthsActive mo active', color: appColors.success),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _InfoChip extends StatelessWidget {
  final String label;
  final Color color;

  const _InfoChip({required this.label, required this.color});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    return Container(
      padding: const EdgeInsets.symmetric(
        horizontal: AppTheme.spacingSm,
        vertical: AppTheme.spacingXs,
      ),
      decoration: BoxDecoration(
        color: color.withOpacity(AppTheme.opacityLight),
        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
      ),
      child: Text(
        label,
        style: text.labelSmall?.copyWith(color: color, fontWeight: FontWeight.w600),
      ),
    );
  }
}
