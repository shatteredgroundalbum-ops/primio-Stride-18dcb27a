import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class WeeklyChart extends StatelessWidget {
  final List<double> weeklySteps;

  const WeeklyChart({super.key, required this.weeklySteps});

  static const _days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final maxY = weeklySteps.isEmpty
        ? 10000.0
        : weeklySteps.reduce((a, b) => a > b ? a : b).clamp(5000.0, double.infinity) * 1.2;

    return GlassCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.bar_chart, size: AppTheme.iconSm, color: appColors.stepsAccent),
              const SizedBox(width: AppTheme.spacingSm),
              Text('Weekly Steps', style: text.titleSmall),
            ],
          ),
          const SizedBox(height: AppTheme.spacingMd),
          SizedBox(
            height: 140,
            child: BarChart(
              BarChartData(
                maxY: maxY,
                barTouchData: BarTouchData(enabled: false),
                titlesData: FlTitlesData(
                  show: true,
                  bottomTitles: AxisTitles(
                    sideTitles: SideTitles(
                      showTitles: true,
                      reservedSize: 24,
                      getTitlesWidget: (value, meta) {
                        final idx = value.toInt();
                        if (idx < 0 || idx >= _days.length) return const SizedBox.shrink();
                        return Padding(
                          padding: const EdgeInsets.only(top: AppTheme.spacingXs),
                          child: Text(
                            _days[idx],
                            style: text.labelSmall,
                          ),
                        );
                      },
                    ),
                  ),
                  leftTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
                  topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
                  rightTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
                ),
                gridData: const FlGridData(show: false),
                borderData: FlBorderData(show: false),
                barGroups: List.generate(weeklySteps.length, (i) {
                  final isCurrent = i == 3;
                  return BarChartGroupData(
                    x: i,
                    barRods: [
                      BarChartRodData(
                        toY: weeklySteps[i],
                        width: 20,
                        borderRadius: BorderRadius.circular(AppTheme.spacingXs),
                        gradient: LinearGradient(
                          begin: Alignment.bottomCenter,
                          end: Alignment.topCenter,
                          colors: isCurrent
                              ? [appColors.stepsAccent.withOpacity(AppTheme.opacityOverlay), appColors.stepsAccent]
                              : [colors.primary.withOpacity(0.3), colors.primary.withOpacity(AppTheme.opacityHint)],
                        ),
                      ),
                    ],
                  );
                }),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
