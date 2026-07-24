import 'package:flutter/material.dart';
import '../../theme/theme.dart';

class PlaceholderScreen extends StatelessWidget {
  final String title;
  final IconData icon;
  final Color accentColor;
  final String description;

  const PlaceholderScreen({
    super.key,
    required this.title,
    required this.icon,
    required this.accentColor,
    required this.description,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(title: Text(title)),
      body: SafeArea(
        child: Center(
          child: Padding(
            padding: const EdgeInsets.all(AppTheme.spacingXl),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 96,
                  height: 96,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    gradient: LinearGradient(
                      colors: [
                        accentColor.withOpacity(AppTheme.opacityLight),
                        accentColor.withOpacity(AppTheme.opacitySubtle),
                      ],
                    ),
                  ),
                  child: Icon(icon, size: AppTheme.iconXl, color: accentColor),
                ),
                const SizedBox(height: AppTheme.spacingLg),
                Text('Coming Soon', style: text.headlineMedium),
                const SizedBox(height: AppTheme.spacingSm),
                Text(
                  description,
                  style: text.bodyMedium?.copyWith(
                    color: colors.onSurface.withOpacity(AppTheme.opacityHint),
                  ),
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
