import 'package:flutter/material.dart';
import '../../services/fasting_service.dart';
import '../../theme/theme.dart';

class FastingProtocolSelector extends StatelessWidget {
  final int selectedIndex;
  final ValueChanged<int> onSelect;

  const FastingProtocolSelector({
    super.key,
    required this.selectedIndex,
    required this.onSelect,
  });

  @override
  Widget build(BuildContext context) {
    final text = Theme.of(context).textTheme;
    final colors = Theme.of(context).colorScheme;
    final appColors = Theme.of(context).extension<AppColorsExtension>()!;
    final protocols = FastingService.protocols;

    return SizedBox(
      height: 100,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        itemCount: protocols.length,
        separatorBuilder: (_, __) => const SizedBox(width: AppTheme.spacingSm),
        itemBuilder: (context, index) {
          final p = protocols[index];
          final isSelected = index == selectedIndex;
          return GestureDetector(
            onTap: () => onSelect(index),
            child: Container(
              width: 100,
              padding: const EdgeInsets.all(AppTheme.spacingSm + 2),
              decoration: BoxDecoration(
                color: isSelected
                    ? appColors.fastingActive.withOpacity(AppTheme.opacityLight)
                    : colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                borderRadius: BorderRadius.circular(AppTheme.radiusMedium),
                border: Border.all(
                  color: isSelected
                      ? appColors.fastingActive.withOpacity(0.4)
                      : colors.onSurface.withOpacity(AppTheme.opacitySubtle),
                  width: isSelected ? AppTheme.borderSelected : AppTheme.borderDefault,
                ),
              ),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(
                    p.label,
                    style: text.titleMedium?.copyWith(
                      color: isSelected ? appColors.fastingActive : colors.onSurface,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                  const SizedBox(height: AppTheme.spacingXs),
                  Text(
                    p.difficulty,
                    style: text.labelSmall?.copyWith(
                      color: isSelected
                          ? appColors.fastingActive.withOpacity(AppTheme.opacityOverlay)
                          : appColors.subtleText,
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}
