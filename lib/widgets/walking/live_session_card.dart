import 'package:flutter/material.dart';

import '../../models/walking_session.dart';
import '../../theme/theme.dart';
import '../common/glass_card.dart';

/// Live in-progress session card for the Walking screen. Shows the
/// active session's live stats (distance, duration, pace, calories,
/// steps) from the Rust engine via [WorkoutRecorder.sessionStream],
/// and provides the start/pause/resume/stop/cancel controls.
class LiveSessionCard extends StatelessWidget {
  final WalkingSession? activeSession;
  final bool isRecording;
  final bool isPaused;
  final double distanceKm;
  final Duration duration;
  final double paceMinPerKm;
  final double calories;
  final int steps;
  final SessionType? selectedType;
  final ValueChanged<SessionType>? onTypeSelected;
  final VoidCallback? onStart;
  final VoidCallback? onPause;
  final VoidCallback? onResume;
  final VoidCallback? onStop;
  final VoidCallback? onCancel;

  const LiveSessionCard({
    super.key,
    required this.activeSession,
    required this.isRecording,
    required this.isPaused,
    required this.distanceKm,
    required this.duration,
    required this.paceMinPerKm,
    required this.calories,
    required this.steps,
    this.selectedType,
    this.onTypeSelected,
    this.onStart,
    this.onPause,
    this.onResume,
    this.onStop,
    this.onCancel,
  });

  String _formatDuration(Duration d) {
    final h = d.inHours;
    final m = d.inMinutes.remainder(60);
    final s = d.inSeconds.remainder(60);
    if (h > 0) {
      return '${h.toString().padLeft(2, '0')}:'
          '${m.toString().padLeft(2, '0')}:'
          '${s.toString().padLeft(2, '0')}';
    }
    return '${m.toString().padLeft(2, '0')}:'
        '${s.toString().padLeft(2, '0')}';
  }

  String _formatPace(double pace) {
    if (pace <= 0) return '--';
    final mins = pace.floor();
    final secs = ((pace - mins) * 60).round();
    return '$mins:${secs.toString().padLeft(2, '0')}';
  }

  String _typeLabel(SessionType type) {
    switch (type) {
      case SessionType.walk:
        return 'Walk';
      case SessionType.run:
        return 'Run';
      case SessionType.hike:
        return 'Hike';
    }
  }

  IconData _typeIcon(SessionType type) {
    switch (type) {
      case SessionType.walk:
        return Icons.directions_walk;
      case SessionType.run:
        return Icons.directions_run;
      case SessionType.hike:
        return Icons.hiking;
    }
  }

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          isRecording ? 'Live Session' : 'Start a Session',
          style: text.titleMedium,
        ),
        const SizedBox(height: AppTheme.spacingSm),
        GlassCard(
          borderColor: isRecording
              ? appColors.stepsAccent.withOpacity(AppTheme.opacityOverlay)
              : null,
          child: isRecording
              ? _buildLiveContent(context)
              : _buildStartContent(context),
        ),
      ],
    );
  }

  Widget _buildStartContent(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: SessionType.values.map((type) {
            final isSelected = selectedType == type;
            return GestureDetector(
              onTap: () => onTypeSelected?.call(type),
              child: Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppTheme.spacingMd,
                  vertical: AppTheme.spacingSm,
                ),
                decoration: BoxDecoration(
                  color: isSelected
                      ? appColors.stepsAccent
                          .withOpacity(AppTheme.opacityLight)
                      : Colors.transparent,
                  borderRadius:
                      BorderRadius.circular(AppTheme.radiusSmall),
                  border: Border.all(
                    color: isSelected
                        ? appColors.stepsAccent
                        : appColors.subtleText
                            .withOpacity(AppTheme.opacitySubtle),
                  ),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      _typeIcon(type),
                      size: AppTheme.iconMd,
                      color: isSelected
                          ? appColors.stepsAccent
                          : appColors.subtleText,
                    ),
                    const SizedBox(width: AppTheme.spacingSm),
                    Text(
                      _typeLabel(type),
                      style: text.bodyMedium?.copyWith(
                        color: isSelected
                            ? appColors.stepsAccent
                            : appColors.subtleText,
                      ),
                    ),
                  ],
                ),
              ),
            );
          }).toList(),
        ),
        const SizedBox(height: AppTheme.spacingMd),
        SizedBox(
          width: double.infinity,
          child: ElevatedButton.icon(
            onPressed: onStart,
            icon: const Icon(Icons.play_arrow),
            label: Text(
              selectedType != null
                  ? 'Start ${_typeLabel(selectedType!)}'
                  : 'Select a type',
            ),
            style: ElevatedButton.styleFrom(
              minimumSize: const Size.fromHeight(AppTheme.buttonHeight),
              backgroundColor: appColors.stepsAccent,
              foregroundColor: const Color(0xFF0A0A0F),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildLiveContent(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;

    return Column(
      children: [
        Row(
          children: [
            Icon(
              _typeIcon(activeSession?.type ?? SessionType.walk),
              size: AppTheme.iconMd,
              color: appColors.stepsAccent,
            ),
            const SizedBox(width: AppTheme.spacingSm),
            Text(
              _typeLabel(activeSession?.type ?? SessionType.walk),
              style: text.titleSmall,
            ),
            const Spacer(),
            if (isPaused)
              Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: AppTheme.spacingSm,
                  vertical: AppTheme.spacingXs,
                ),
                decoration: BoxDecoration(
                  color: appColors.warning.withOpacity(AppTheme.opacityLight),
                  borderRadius:
                      BorderRadius.circular(AppTheme.radiusSmall),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.pause_circle, size: AppTheme.iconSm,
                        color: appColors.warning),
                    const SizedBox(width: AppTheme.spacingXs),
                    Text(
                      'Paused',
                      style: text.labelSmall?.copyWith(
                        color: appColors.warning,
                      ),
                    ),
                  ],
                ),
              ),
          ],
        ),
        const SizedBox(height: AppTheme.spacingMd),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            _liveStat(context, 'Distance',
                '${distanceKm.toStringAsFixed(2)} km'),
            _liveStat(context, 'Time', _formatDuration(duration)),
          ],
        ),
        const SizedBox(height: AppTheme.spacingMd),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            _liveStat(
                context, 'Pace', '${_formatPace(paceMinPerKm)} /km'),
            _liveStat(
                context, 'Calories', calories.toStringAsFixed(0)),
          ],
        ),
        const SizedBox(height: AppTheme.spacingMd),
        Row(
          children: [
            _liveStat(
                context, 'Steps', steps.toString()),
            const Spacer(),
            if (activeSession?.averageHeartRate != null &&
                activeSession!.averageHeartRate! > 0)
              _liveStat(
                  context,
                  'Avg HR',
                  '${activeSession!.averageHeartRate} bpm'),
          ],
        ),
        const SizedBox(height: AppTheme.spacingMd),
        Row(
          children: [
            if (isPaused)
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: onResume,
                  icon: const Icon(Icons.play_arrow),
                  label: const Text('Resume'),
                ),
              )
            else
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: onPause,
                  icon: const Icon(Icons.pause),
                  label: const Text('Pause'),
                ),
              ),
            const SizedBox(width: AppTheme.spacingSm),
            Expanded(
              child: ElevatedButton.icon(
                onPressed: onStop,
                icon: const Icon(Icons.stop),
                label: const Text('Finish'),
                style: ElevatedButton.styleFrom(
                  backgroundColor: appColors.stepsAccent,
                  foregroundColor: const Color(0xFF0A0A0F),
                ),
              ),
            ),
            const SizedBox(width: AppTheme.spacingSm),
            IconButton(
              onPressed: onCancel,
              icon: Icon(Icons.close, color: appColors.danger),
              tooltip: 'Cancel without saving',
            ),
          ],
        ),
      ],
    );
  }

  Widget _liveStat(
    BuildContext context,
    String label,
    String value,
  ) {
    final text = Theme.of(context).textTheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: text.labelSmall?.copyWith(color: appColors.subtleText),
        ),
        const SizedBox(height: AppTheme.spacingXs),
        Text(
          value,
          style: text.bodyLarge?.copyWith(fontWeight: FontWeight.bold),
        ),
      ],
    );
  }
}
