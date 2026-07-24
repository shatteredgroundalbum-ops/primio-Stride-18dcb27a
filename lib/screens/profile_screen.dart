import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:provider/provider.dart';
import '../providers/auth_provider.dart';
import '../providers/profile_provider.dart';
import '../theme/theme.dart';
import '../widgets/common/section_header.dart';
import '../widgets/profile/achievements_card.dart';
import '../widgets/profile/body_stats_card.dart';
import '../widgets/profile/profile_header_card.dart';
import '../widgets/profile/settings_list.dart';

class ProfileScreen extends StatelessWidget {
  const ProfileScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<ProfileProvider>();
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;

    if (provider.isLoading) {
      return Scaffold(
        body: Center(
          child: CircularProgressIndicator(color: colors.primary),
        ),
      );
    }

    if (provider.profile == null) {
      return Scaffold(
        body: Center(
          child: Text('Unable to load profile', style: text.bodyLarge),
        ),
      );
    }

    final profile = provider.profile!;

    return Scaffold(
      appBar: AppBar(
        title: Text('Profile', style: text.titleLarge),
      ),
      body: SafeArea(
        child: RefreshIndicator(
          onRefresh: provider.loadProfile,
          color: colors.primary,
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              AppTheme.spacingMd, AppTheme.spacingSm, AppTheme.spacingMd, AppTheme.spacingLg,
            ),
            children: [
              ProfileHeaderCard(profile: profile)
                  .animate().fadeIn(duration: 500.ms).slideY(begin: -0.05, end: 0),
              const SizedBox(height: AppTheme.spacingLg),
              if (provider.bodyStats != null) ...[
                const SectionHeader(
                  title: 'Body Composition',
                  icon: Icons.monitor_weight_outlined,
                ).animate().fadeIn(duration: 400.ms, delay: 100.ms),
                BodyStatsCard(
                  stats: provider.bodyStats!,
                  bmi: profile.bmi,
                  bmiCategory: profile.bmiCategory,
                ).animate().fadeIn(duration: 500.ms, delay: 200.ms),
                const SizedBox(height: AppTheme.spacingLg),
              ],
              const SectionHeader(
                title: 'Achievements',
                icon: Icons.emoji_events,
              ).animate().fadeIn(duration: 400.ms, delay: 300.ms),
              AchievementsCard(
                achievements: provider.achievements,
                earnedCount: provider.earnedCount,
              ).animate().fadeIn(duration: 500.ms, delay: 400.ms),
              const SizedBox(height: AppTheme.spacingLg),
              const SectionHeader(
                title: 'Settings',
                icon: Icons.settings,
              ).animate().fadeIn(duration: 400.ms, delay: 500.ms),
              SettingsList(
                onLogout: () => context.read<AuthProvider>().logout(),
              ).animate().fadeIn(duration: 500.ms, delay: 600.ms),
            ],
          ),
        ),
      ),
    );
  }
}
