import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import '../../models/profile_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class BodyStatsCard extends StatelessWidget {
  final BodyStats stats;
  final double bmi;
  final String bmiCategory;

  const BodyStatsCard({
    super.key,
    required this.stats,
    required this.bmi,
    required this.bmiCategory,
  });

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
              Icon(Icons.monitor_weight_outlined, size: AppTheme.iconSm, color: appColors.stepsAccent),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Body Stats', style: text.titleSmall),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          Row(
            children: [
              Expanded(
                child: _StatColumn(
                  label: 'Weight',
                  value: '${stats.weightKg} kg',
                  color: appColors.stepsAccent,
                ),
              ),
              Expanded(
                child: _StatColumn(
                  label: 'Body Fat',
                  value: '${stats.bodyFatPercent}%',
                  color: appColors.warning,
                ),
              ),
              Expanded(
                child: _StatColumn(
                  label: 'Muscle',
                  value: '${stats.muscleMassKg} kg',
                  color: appColors.exerciseAccent,
                ),
              ),
              Expanded(
                child: _StatColumn(
                  label: 'BMI',
                  value: bmi.toStringAsFixed(1),
                  color: appColors.nutritionAccent,
                ),
              ),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          Text('Weight Trend', style: text.labelMedium),
          const SizedBox(height: AppTheme.spacingSm),
          SizedBox(
            height: 100,
            child: LineChart(
              LineChartData(
                gridData: const FlGridData(show: false),
                titlesData: const FlTitlesData(show: false),
                borderData: FlBorderData(show: false),
                lineTouchData: const LineTouchData(enabled: false),
                minY: stats.weightHistory.map((e) => e.weightKg).reduce((a, b) => a < b ? a : b) - 1,
                maxY: stats.weightHistory.map((e) => e.weightKg).reduce((a, b) => a > b ? a : b) + 1,
                lineBarsData: [
                  LineChartBarData(
                    spots: List.generate(
                      stats.weightHistory.length,
                      (i) => FlSpot(i.toDouble(), stats.weightHistory[i].weightKg),
                    ),
                    isCurved: true,
                    color: appColors.stepsAccent,
                    barWidth: 2.5,
                    dotData: FlDotData(
                      show: true,
                      getDotPainter: (spot, percent, barData, index) => FlDotCirclePainter(
                        radius: index == stats.weightHistory.length - 1 ? 4 : 2,
                        color: appColors.stepsAccent,
                        strokeWidth: 0,
                      ),
                    ),
                    belowBarData: BarAreaData(
                      show: true,
                      color: appColors.stepsAccent.withOpacity(AppTheme.opacitySubtle),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _StatColumn extends StatelessWidget {
  final String label;
  final String value;
  final Color color;

  const _StatColumn({required this.label, required this.value, required this.color});

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
