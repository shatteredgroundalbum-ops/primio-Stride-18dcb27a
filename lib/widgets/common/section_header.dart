import 'package:flutter/material.dart';
import '../../theme/theme.dart';

class SectionHeader extends StatelessWidget {
  final String title;
  final String? actionLabel;
  final VoidCallback? onAction;
  final IconData? icon;

  const SectionHeader({
    super.key,
    required this.title,
    this.actionLabel,
    this.onAction,
    this.icon,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;

    return Padding(
      padding: const EdgeInsets.only(bottom: AppTheme.spacingSm),
      child: Row(
        children: [
          if (icon != null) ...[
            Icon(icon, size: AppTheme.iconSm, color: colors.primary),
            const SizedBox(width: AppTheme.spacingSm),
          ],
          Text(title, style: text.titleMedium),
          const Spacer(),
          if (actionLabel != null)
            GestureDetector(
              onTap: onAction,
              child: Text(
                actionLabel!,
                style: text.labelMedium?.copyWith(color: colors.primary),
              ),
            ),
        ],
      ),
    );
  }
}
