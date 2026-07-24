import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

@immutable
class AppColorsExtension extends ThemeExtension<AppColorsExtension> {
  final Color success;
  final Color warning;
  final Color danger;
  final Color subtleText;
  final Color cardHighlight;
  final Color fastingActive;
  final Color stepsAccent;
  final Color exerciseAccent;
  final Color nutritionAccent;
  final Color recoveryAccent;

  const AppColorsExtension({
    required this.success,
    required this.warning,
    required this.danger,
    required this.subtleText,
    required this.cardHighlight,
    required this.fastingActive,
    required this.stepsAccent,
    required this.exerciseAccent,
    required this.nutritionAccent,
    required this.recoveryAccent,
  });

  @override
  AppColorsExtension copyWith({
    Color? success,
    Color? warning,
    Color? danger,
    Color? subtleText,
    Color? cardHighlight,
    Color? fastingActive,
    Color? stepsAccent,
    Color? exerciseAccent,
    Color? nutritionAccent,
    Color? recoveryAccent,
  }) =>
      AppColorsExtension(
        success: success ?? this.success,
        warning: warning ?? this.warning,
        danger: danger ?? this.danger,
        subtleText: subtleText ?? this.subtleText,
        cardHighlight: cardHighlight ?? this.cardHighlight,
        fastingActive: fastingActive ?? this.fastingActive,
        stepsAccent: stepsAccent ?? this.stepsAccent,
        exerciseAccent: exerciseAccent ?? this.exerciseAccent,
        nutritionAccent: nutritionAccent ?? this.nutritionAccent,
        recoveryAccent: recoveryAccent ?? this.recoveryAccent,
      );

  @override
  AppColorsExtension lerp(covariant ThemeExtension<AppColorsExtension>? other, double t) {
    if (other is! AppColorsExtension) return this;
    return AppColorsExtension(
      success: Color.lerp(success, other.success, t)!,
      warning: Color.lerp(warning, other.warning, t)!,
      danger: Color.lerp(danger, other.danger, t)!,
      subtleText: Color.lerp(subtleText, other.subtleText, t)!,
      cardHighlight: Color.lerp(cardHighlight, other.cardHighlight, t)!,
      fastingActive: Color.lerp(fastingActive, other.fastingActive, t)!,
      stepsAccent: Color.lerp(stepsAccent, other.stepsAccent, t)!,
      exerciseAccent: Color.lerp(exerciseAccent, other.exerciseAccent, t)!,
      nutritionAccent: Color.lerp(nutritionAccent, other.nutritionAccent, t)!,
      recoveryAccent: Color.lerp(recoveryAccent, other.recoveryAccent, t)!,
    );
  }
}

class AppTheme {
  AppTheme._();

  static const double spacingXs = 4.0;
  static const double spacingSm = 8.0;
  static const double spacingMd = 16.0;
  static const double spacingLg = 24.0;
  static const double spacingXl = 32.0;

  static const double radiusSmall = 8.0;
  static const double radiusMedium = 16.0;
  static const double radiusLarge = 24.0;
  static const double radiusXl = 32.0;

  static const double iconSm = 16.0;
  static const double iconMd = 24.0;
  static const double iconLg = 32.0;
  static const double iconXl = 48.0;

  static const double buttonHeight = 52.0;
  static const double cardMinHeight = 80.0;

  static const double opacityDisabled = 0.38;
  static const double opacityHint = 0.5;
  static const double opacityOverlay = 0.7;
  static const double opacitySubtle = 0.08;
  static const double opacityLight = 0.12;

  static const double borderDefault = 1.0;
  static const double borderSelected = 2.0;

  static const _appColors = AppColorsExtension(
    success: Color(0xFF00E676),
    warning: Color(0xFFFFAB40),
    danger: Color(0xFFFF5252),
    subtleText: Color(0xFF8E8E93),
    cardHighlight: Color(0xFF1C1C2E),
    fastingActive: Color(0xFFFF6B35),
    stepsAccent: Color(0xFF00E5FF),
    exerciseAccent: Color(0xFFFF4081),
    nutritionAccent: Color(0xFF69F0AE),
    recoveryAccent: Color(0xFFB388FF),
  );

  static final ThemeData darkTheme = _buildTheme(
    colorScheme: ColorScheme.fromSeed(
      seedColor: const Color(0xFF6C63FF),
      brightness: Brightness.dark,
      surface: const Color(0xFF0D0D14),
      onSurface: const Color(0xFFF5F5F7),
    ),
    appColors: _appColors,
  );

  static ThemeData _buildTheme({
    required ColorScheme colorScheme,
    required AppColorsExtension appColors,
  }) {
    final textTheme = _buildTextTheme(colorScheme);
    return ThemeData(
      useMaterial3: true,
      colorScheme: colorScheme,
      scaffoldBackgroundColor: colorScheme.surface,
      textTheme: textTheme,
      appBarTheme: AppBarTheme(
        backgroundColor: Colors.transparent,
        foregroundColor: colorScheme.onSurface,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        titleTextStyle: textTheme.titleLarge,
      ),
      cardTheme: CardThemeData(
        color: const Color(0xFF16162A),
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(radiusMedium),
        ),
        margin: EdgeInsets.zero,
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          minimumSize: const Size(double.infinity, 52),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(radiusMedium),
          ),
          backgroundColor: colorScheme.primary,
          foregroundColor: colorScheme.onPrimary,
          textStyle: textTheme.labelLarge,
        ),
      ),
      navigationBarTheme: NavigationBarThemeData(
        backgroundColor: const Color(0xFF12121F),
        indicatorColor: colorScheme.primary.withOpacity(opacityLight),
        labelTextStyle: WidgetStatePropertyAll(
          textTheme.labelSmall?.copyWith(fontWeight: FontWeight.w600),
        ),
        iconTheme: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return IconThemeData(color: colorScheme.primary, size: iconMd);
          }
          return IconThemeData(color: appColors.subtleText, size: iconMd);
        }),
        height: 70,
      ),
      dividerTheme: DividerThemeData(
        color: colorScheme.onSurface.withOpacity(opacitySubtle),
        thickness: 1,
      ),
      extensions: [appColors],
    );
  }

  static TextTheme _buildTextTheme(ColorScheme colorScheme) {
    final base = GoogleFonts.interTextTheme();
    return base.copyWith(
      headlineLarge: base.headlineLarge?.copyWith(
        fontWeight: FontWeight.w800,
        color: colorScheme.onSurface,
        fontSize: 28,
        letterSpacing: -0.5,
      ),
      headlineMedium: base.headlineMedium?.copyWith(
        fontWeight: FontWeight.w700,
        color: colorScheme.onSurface,
        fontSize: 22,
      ),
      titleLarge: base.titleLarge?.copyWith(
        fontWeight: FontWeight.w700,
        color: colorScheme.onSurface,
        fontSize: 18,
      ),
      titleMedium: base.titleMedium?.copyWith(
        fontWeight: FontWeight.w600,
        color: colorScheme.onSurface,
        fontSize: 16,
      ),
      titleSmall: base.titleSmall?.copyWith(
        fontWeight: FontWeight.w600,
        color: colorScheme.onSurface,
        fontSize: 14,
      ),
      bodyLarge: base.bodyLarge?.copyWith(
        color: colorScheme.onSurface,
        fontSize: 16,
      ),
      bodyMedium: base.bodyMedium?.copyWith(
        color: colorScheme.onSurface,
        fontSize: 14,
      ),
      bodySmall: base.bodySmall?.copyWith(
        color: colorScheme.onSurface.withOpacity(opacityHint),
        fontSize: 12,
      ),
      labelLarge: base.labelLarge?.copyWith(
        fontWeight: FontWeight.w700,
        color: colorScheme.onSurface,
        fontSize: 15,
        letterSpacing: 0.5,
      ),
      labelMedium: base.labelMedium?.copyWith(
        color: colorScheme.onSurfaceVariant,
        fontSize: 12,
        fontWeight: FontWeight.w500,
      ),
      labelSmall: base.labelSmall?.copyWith(
        color: colorScheme.onSurfaceVariant,
        fontSize: 11,
        fontWeight: FontWeight.w500,
      ),
    );
  }
}
