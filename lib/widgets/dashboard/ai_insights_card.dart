import 'package:flutter/material.dart';
import '../../models/health_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class AiInsightsCard extends StatelessWidget {
  final List<AiInsight> insights;

  const AiInsightsCard({super.key, required this.insights});

  @override
  Widget build(BuildContext context) {
    if (insights.isEmpty) return const SizedBox.shrink();

    return Column(
      children: [
        for (int i = 0; i < insights.length; i++) ...[
          if (i > 0) const SizedBox(height: AppTheme.spacingSm),
          _InsightTile(insight: insights[i]),
        ],
      ],
    );
  }
}

class _InsightTile extends StatelessWidget {
  final AiInsight insight;

  const _InsightTile({required this.insight});

  Color _accentColor(BuildContext context) {
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    switch (insight.type) {
      case InsightType.nutrition:
        return appColors.nutritionAccent;
      case InsightType.exercise:
        return appColors.exerciseAccent;
      case InsightType.recovery:
        return appColors.recoveryAccent;
      case InsightType.fasting:
        return appColors.fastingActive;
      case InsightType.steps:
        return appColors.stepsAccent;
    }
  }

  IconData get _icon {
    switch (insight.type) {
      case InsightType.nutrition:
        return Icons.restaurant;
      case InsightType.exercise:
        return Icons.fitness_center;
      case InsightType.recovery:
        return Icons.self_improvement;
      case InsightType.fasting:
        return Icons.timer;
      case InsightType.steps:
        return Icons.directions_walk;
    }
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final accent = _accentColor(context);

    return GlassCard(
      borderColor: accent.withOpacity(AppTheme.opacityLight),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
              gradient: LinearGradient(
                colors: [
                  accent.withOpacity(0.25),
                  accent.withOpacity(AppTheme.opacitySubtle),
                ],
              ),
            ),
            child: Icon(_icon, size: AppTheme.iconSm + 2, color: accent),
          ),
          const SizedBox(width: AppTheme.spacingSm + 4),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(insight.title, style: text.titleSmall?.copyWith(color: accent)),
                const SizedBox(height: AppTheme.spacingXs),
                Text(
                  insight.message,
                  style: text.bodySmall,
                  maxLines: 3,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
