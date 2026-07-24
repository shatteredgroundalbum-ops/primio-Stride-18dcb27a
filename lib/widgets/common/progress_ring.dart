import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../../theme/theme.dart';

class ProgressRing extends StatelessWidget {
  final double progress;
  final double size;
  final double strokeWidth;
  final Color activeColor;
  final Widget? center;

  const ProgressRing({
    super.key,
    required this.progress,
    required this.size,
    this.strokeWidth = 6,
    required this.activeColor,
    this.center,
  });

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;

    return SizedBox(
      width: size,
      height: size,
      child: Stack(
        alignment: Alignment.center,
        children: [
          CustomPaint(
            size: Size(size, size),
            painter: _RingPainter(
              progress: progress,
              activeColor: activeColor,
              trackColor: colors.onSurface.withOpacity(AppTheme.opacitySubtle),
              strokeWidth: strokeWidth,
            ),
          ),
          if (center != null) center!,
        ],
      ),
    );
  }
}

class _RingPainter extends CustomPainter {
  final double progress;
  final Color activeColor;
  final Color trackColor;
  final double strokeWidth;

  _RingPainter({
    required this.progress,
    required this.activeColor,
    required this.trackColor,
    required this.strokeWidth,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = (size.width - strokeWidth) / 2;
    final rect = Rect.fromCircle(center: center, radius: radius);

    final trackPaint = Paint()
      ..color = trackColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..strokeCap = StrokeCap.round;

    canvas.drawCircle(center, radius, trackPaint);

    final activePaint = Paint()
      ..shader = SweepGradient(
        startAngle: -math.pi / 2,
        endAngle: 3 * math.pi / 2,
        colors: [
          activeColor.withOpacity(AppTheme.opacityOverlay),
          activeColor,
        ],
        stops: const [0.0, 1.0],
      ).createShader(rect)
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..strokeCap = StrokeCap.round;

    canvas.drawArc(
      rect,
      -math.pi / 2,
      2 * math.pi * progress.clamp(0.0, 1.0),
      false,
      activePaint,
    );
  }

  @override
  bool shouldRepaint(covariant _RingPainter oldDelegate) =>
      oldDelegate.progress != progress || oldDelegate.activeColor != activeColor;
}
