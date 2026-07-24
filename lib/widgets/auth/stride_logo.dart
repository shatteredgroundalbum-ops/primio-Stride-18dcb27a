import 'package:flutter/material.dart';
import '../../theme/theme.dart';

class StrideLogo extends StatelessWidget {
  final double size;
  const StrideLogo({super.key, this.size = 80});

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: size,
          height: size,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [colors.primary, colors.tertiary],
            ),
          ),
          child: Icon(Icons.directions_walk,
              size: size * 0.5, color: colors.onPrimary),
        ),
        const SizedBox(height: AppTheme.spacingMd),
        Text(
          'S.T.R.I.D.E.',
          style: text.headlineLarge?.copyWith(
            color: colors.primary,
            letterSpacing: 4,
          ),
        ),
      ],
    );
  }
}
