import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class WaterCard extends StatelessWidget {
  final double liters;
  final double goal;

  const WaterCard({super.key, required this.liters, required this.goal});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final progress = (liters / goal).clamp(0.0, 1.0);

    return GlassCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.water_drop, size: AppTheme.iconSm, color: const Color(0xFF42A5F5)),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Hydration', style: text.titleSmall),
            ],
          ),
          const SizedBox(height: AppTheme.spacingSm + 4),
          Row(
            children: [
              Text(
                '${liters.toStringAsFixed(1)}L',
                style: text.headlineMedium?.copyWith(color: const Color(0xFF42A5F5)),
              ),
              Text(
                ' / ${goal.toStringAsFixed(1)}L',
                style: text.bodyMedium?.copyWith(
                  color: colors.onSurface.withOpacity(AppTheme.opacityHint),
                ),
              ),
            ],
          ),
          const SizedBox(height: AppTheme.spacingSm),
          ClipRRect(
            borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
            child: LinearProgressIndicator(
              value: progress,
              minHeight: 8,
              backgroundColor: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
              valueColor: const AlwaysStoppedAnimation(Color(0xFF42A5F5)),
            ),
          ),
        ],
      ),
    );
  }
}
