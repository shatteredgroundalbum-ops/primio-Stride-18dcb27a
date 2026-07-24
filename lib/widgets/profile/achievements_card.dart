import 'package:flutter/material.dart';
import '../../models/profile_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class AchievementsCard extends StatelessWidget {
  final List<Achievement> achievements;
  final int earnedCount;

  const AchievementsCard({
    super.key,
    required this.achievements,
    required this.earnedCount,
  });

  Color _colorForIcon(String iconName, AppColorsExtension appColors, ColorScheme colors) {
    switch (iconName) {
      case 'steps':
        return appColors.stepsAccent;
      case 'exercise':
        return appColors.exerciseAccent;
      case 'fasting':
        return appColors.fastingActive;
      case 'nutrition':
        return appColors.nutritionAccent;
      default:
        return colors.primary;
    }
  }

  IconData _iconForName(String iconName) {
    switch (iconName) {
      case 'steps':
        return Icons.directions_walk;
      case 'exercise':
        return Icons.fitness_center;
      case 'fasting':
        return Icons.timer;
      case 'nutrition':
        return Icons.restaurant;
      default:
        return Icons.emoji_events;
    }
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return GlassCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.emoji_events, size: AppTheme.iconSm, color: appColors.warning),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Achievements', style: text.titleSmall),
              const Spacer(),
              Text('$earnedCount/${achievements.length}', style: text.labelMedium?.copyWith(color: appColors.warning)),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          ...List.generate(achievements.length, (i) {
            final a = achievements[i];
            final color = _colorForIcon(a.iconName, appColors, colors);
            return Padding(
              padding: EdgeInsets.only(top: i > 0 ? AppTheme.spacingSm : 0),
              child: Row(
                children: [
                  Container(
                    width: 40,
                    height: 40,
                    decoration: BoxDecoration(
                      color: a.earned
                          ? color.withOpacity(AppTheme.opacityLight)
                          : colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                      borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                    ),
                    child: Icon(
                      _iconForName(a.iconName),
                      color: a.earned ? color : appColors.subtleText,
                      size: AppTheme.iconMd,
                    ),
                  ),
                  const SizedBox(width: AppTheme.spacingSm + 4),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          a.title,
                          style: text.titleSmall?.copyWith(
                            color: a.earned ? colors.onSurface : appColors.subtleText,
                          ),
                        ),
                        Text(
                          a.description,
                          style: text.labelSmall,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                  if (a.earned)
                    Icon(Icons.check_circle, color: appColors.success, size: AppTheme.iconMd)
                  else
                    Icon(Icons.lock_outline, color: appColors.subtleText, size: AppTheme.iconMd),
                ],
              ),
            );
          }),
        ],
      ),
    );
  }
}
