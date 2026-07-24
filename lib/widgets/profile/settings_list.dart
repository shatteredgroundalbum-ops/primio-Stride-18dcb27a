import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

class SettingsList extends StatelessWidget {
  final VoidCallback? onLogout;

  const SettingsList({super.key, this.onLogout});

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    final items = [
      _SettingsItem(icon: Icons.notifications_outlined, label: 'Notifications', color: appColors.warning),
      _SettingsItem(icon: Icons.flag_outlined, label: 'Goals & Targets', color: appColors.success),
      _SettingsItem(icon: Icons.straighten, label: 'Units & Measurements', color: appColors.stepsAccent),
      _SettingsItem(icon: Icons.sync, label: 'Data Sync', color: colors.primary),
      _SettingsItem(icon: Icons.lock_outline, label: 'Privacy', color: appColors.recoveryAccent),
      _SettingsItem(icon: Icons.info_outline, label: 'About S.T.R.I.D.E.', color: appColors.subtleText),
    ];

    return GlassCard(
      padding: EdgeInsets.zero,
      child: Column(
        children: [
          ...List.generate(items.length, (i) {
            final item = items[i];
            return Column(
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: AppTheme.spacingMd,
                    vertical: AppTheme.spacingSm + 4,
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 36,
                        height: 36,
                        decoration: BoxDecoration(
                          color: item.color.withOpacity(AppTheme.opacityLight),
                          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                        ),
                        child: Icon(item.icon, color: item.color, size: AppTheme.iconSm + 2),
                      ),
                      const SizedBox(width: AppTheme.spacingSm + 4),
                      Expanded(child: Text(item.label, style: text.titleSmall)),
                      Icon(Icons.chevron_right, color: appColors.subtleText, size: AppTheme.iconMd),
                    ],
                  ),
                ),
                if (i < items.length - 1 || onLogout != null)
                  Divider(
                    height: 1,
                    indent: AppTheme.spacingMd + 36 + AppTheme.spacingSm + 4,
                    color: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                  ),
              ],
            );
          }),
          if (onLogout != null)
            InkWell(
              onTap: onLogout,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppTheme.spacingMd,
                  vertical: AppTheme.spacingSm + 4,
                ),
                child: Row(
                  children: [
                    Container(
                      width: 36,
                      height: 36,
                      decoration: BoxDecoration(
                        color: appColors.danger.withOpacity(AppTheme.opacityLight),
                        borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
                      ),
                      child: Icon(Icons.logout, color: appColors.danger, size: AppTheme.iconSm + 2),
                    ),
                    const SizedBox(width: AppTheme.spacingSm + 4),
                    Expanded(
                      child: Text('Sign Out',
                          style: text.titleSmall?.copyWith(color: appColors.danger)),
                    ),
                    Icon(Icons.chevron_right, color: appColors.subtleText, size: AppTheme.iconMd),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _SettingsItem {
  final IconData icon;
  final String label;
  final Color color;

  const _SettingsItem({required this.icon, required this.label, required this.color});
}
