import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../models/fasting_data.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class FastingHistoryList extends StatelessWidget {
  final List<FastingLog> logs;

  const FastingHistoryList({super.key, required this.logs});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final dateFormat = DateFormat('MMM d');

    return Column(
      children: [
        for (int i = 0; i < logs.length; i++) ...[
          if (i > 0) const SizedBox(height: AppTheme.spacingSm),
          GlassCard(
            padding: const EdgeInsets.symmetric(
              horizontal: AppTheme.spacingMd,
              vertical: AppTheme.spacingSm + 4,
            ),
            child: Row(
              children: [
                Container(
                  width: 40,
                  height: 40,
                  decoration: BoxDecoration(
                    color: (logs[i].completed ? appColors.success : appColors.danger)
                        .withOpacity(AppTheme.opacityLight),
                    borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                  ),
                  child: Icon(
                    logs[i].completed ? Icons.check_circle : Icons.cancel,
                    color: logs[i].completed ? appColors.success : appColors.danger,
                    size: AppTheme.iconMd,
                  ),
                ),
                const SizedBox(width: AppTheme.spacingSm + 4),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '${logs[i].protocol.label} Fast',
                        style: text.titleSmall,
                      ),
                      Text(
                        dateFormat.format(logs[i].date),
                        style: text.labelSmall,
                      ),
                    ],
                  ),
                ),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      '${logs[i].actualDuration.inHours}h ${logs[i].actualDuration.inMinutes.remainder(60)}m',
                      style: text.titleSmall?.copyWith(
                        color: logs[i].completed ? appColors.success : appColors.subtleText,
                      ),
                    ),
                    Text(
                      logs[i].completed ? 'Completed' : 'Incomplete',
                      style: text.labelSmall?.copyWith(
                        color: logs[i].completed ? appColors.success : appColors.danger,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ],
    );
  }
}
