import 'package:flutter/material.dart';
import '../../models/nutrition_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class MicroNutrientsCard extends StatelessWidget {
  final List<MicroNutrient> nutrients;

  const MicroNutrientsCard({super.key, required this.nutrients});

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
              Icon(Icons.science_outlined, size: AppTheme.iconSm, color: appColors.recoveryAccent),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Micronutrients', style: text.titleSmall),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          GridView.builder(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: 2,
              mainAxisSpacing: AppTheme.spacingSm,
              crossAxisSpacing: AppTheme.spacingSm,
              childAspectRatio: 2.8,
            ),
            itemCount: nutrients.length,
            itemBuilder: (context, index) {
              final n = nutrients[index];
              final barColor = n.dailyPercent >= 0.75
                  ? appColors.success
                  : n.dailyPercent >= 0.5
                      ? appColors.warning
                      : appColors.danger;
              return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Flexible(child: Text(n.name, style: text.labelSmall, maxLines: 1, overflow: TextOverflow.ellipsis)),
                      Text('${(n.dailyPercent * 100).toInt()}%', style: text.labelSmall?.copyWith(color: barColor)),
                    ],
                  ),
                  const SizedBox(height: AppTheme.spacingXs),
                  ClipRRect(
                    borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                    child: LinearProgressIndicator(
                      value: n.dailyPercent.clamp(0.0, 1.0),
                      minHeight: 4,
                      backgroundColor: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                      valueColor: AlwaysStoppedAnimation(barColor),
                    ),
                  ),
                ],
              );
            },
          ),
        ],
      ),
    );
  }
}
