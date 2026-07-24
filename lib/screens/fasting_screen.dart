import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:provider/provider.dart';
import '../providers/fasting_provider.dart';
import '../services/fasting_service.dart';
import '../theme/theme.dart';
import '../widgets/common/section_header.dart';
import '../widgets/fasting/fasting_history_list.dart';
import '../widgets/fasting/fasting_protocol_selector.dart';
import '../widgets/fasting/fasting_stats_card.dart';
import '../widgets/fasting/fasting_timer_ring.dart';

class FastingScreen extends StatelessWidget {
  const FastingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<FastingProvider>();
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final isActive = provider.activeFast != null;

    if (provider.isLoading) {
      return Scaffold(
        body: Center(
          child: CircularProgressIndicator(color: appColors.fastingActive),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: Text('Fasting', style: text.titleLarge?.copyWith(color: appColors.fastingActive)),
      ),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(
            AppTheme.spacingMd, AppTheme.spacingSm, AppTheme.spacingMd, AppTheme.spacingLg,
          ),
          children: [
            Center(
              child: FastingTimerRing(
                progress: provider.progress,
                elapsed: provider.elapsed,
                remaining: provider.remaining,
                protocol: isActive
                    ? provider.activeFast!.protocol.label
                    : FastingService.protocols[provider.selectedProtocolIndex].label,
                isActive: isActive,
              ),
            ).animate().fadeIn(duration: 600.ms).scale(begin: const Offset(0.9, 0.9), end: const Offset(1, 1)),
            const SizedBox(height: AppTheme.spacingLg),
            if (!isActive) ...[
              const SectionHeader(
                title: 'Choose Protocol',
                icon: Icons.tune,
              ).animate().fadeIn(duration: 400.ms, delay: 200.ms),
              FastingProtocolSelector(
                selectedIndex: provider.selectedProtocolIndex,
                onSelect: provider.selectProtocol,
              ).animate().fadeIn(duration: 500.ms, delay: 300.ms),
              const SizedBox(height: AppTheme.spacingMd),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: AppTheme.spacingXl),
                child: Text(
                  FastingService.protocols[provider.selectedProtocolIndex].description,
                  style: text.bodySmall,
                  textAlign: TextAlign.center,
                ),
              ).animate().fadeIn(duration: 400.ms, delay: 350.ms),
              const SizedBox(height: AppTheme.spacingMd),
            ],
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: AppTheme.spacingXl),
              child: ElevatedButton.icon(
                onPressed: isActive ? provider.endFast : provider.startFast,
                icon: Icon(isActive ? Icons.stop : Icons.play_arrow),
                label: Text(isActive ? 'End Fast' : 'Start Fast'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: isActive ? appColors.danger : appColors.fastingActive,
                  foregroundColor: colors.onPrimary,
                ),
              ),
            ).animate().fadeIn(duration: 500.ms, delay: 400.ms),
            const SizedBox(height: AppTheme.spacingLg),
            if (provider.stats != null) ...[
              FastingStatsCard(stats: provider.stats!)
                  .animate().fadeIn(duration: 500.ms, delay: 500.ms),
              const SizedBox(height: AppTheme.spacingLg),
            ],
            const SectionHeader(
              title: 'History',
              icon: Icons.history,
            ).animate().fadeIn(duration: 400.ms, delay: 600.ms),
            FastingHistoryList(logs: provider.recentLogs)
                .animate().fadeIn(duration: 500.ms, delay: 700.ms),
          ],
        ),
      ),
    );
  }
}
