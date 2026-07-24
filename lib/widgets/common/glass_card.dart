import 'package:flutter/material.dart';
import '../../theme/theme.dart';

class GlassCard extends StatelessWidget {
  final Widget child;
  final EdgeInsets? padding;
  final VoidCallback? onTap;
  final Color? borderColor;

  const GlassCard({
    super.key,
    required this.child,
    this.padding,
    this.onTap,
    this.borderColor,
  });

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;

    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: padding ?? const EdgeInsets.all(AppTheme.spacingMd),
        decoration: BoxDecoration(
          color: colors.surface.withOpacity(AppTheme.opacityOverlay),
          borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
          border: Border.all(
            color: borderColor ?? colors.onSurface.withOpacity(AppTheme.opacitySubtle),
            width: AppTheme.borderDefault,
          ),
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [
              const Color(0xFF1A1A30),
              const Color(0xFF16162A),
            ],
          ),
        ),
        child: child,
      ),
    );
  }
}
