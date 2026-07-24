import 'package:flutter/material.dart';
import '../../theme/theme.dart';

class AuthTextField extends StatelessWidget {
  final TextEditingController controller;
  final String label;
  final IconData icon;
  final bool obscureText;
  final TextInputType keyboardType;
  final String? Function(String?)? validator;

  const AuthTextField({
    super.key,
    required this.controller,
    required this.label,
    required this.icon,
    this.obscureText = false,
    this.keyboardType = TextInputType.text,
    this.validator,
  });

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;

    return TextFormField(
      controller: controller,
      obscureText: obscureText,
      keyboardType: keyboardType,
      validator: validator,
      style: text.bodyLarge,
      decoration: InputDecoration(
        labelText: label,
        labelStyle: text.bodyMedium
            ?.copyWith(color: colors.onSurface.withOpacity(AppTheme.opacityHint)),
        prefixIcon:
            Icon(icon, color: colors.primary, size: AppTheme.iconSm + 2),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          borderSide: BorderSide(
              color: colors.onSurface.withOpacity(AppTheme.opacityLight)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          borderSide:
              BorderSide(color: colors.primary, width: AppTheme.borderSelected),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          borderSide: BorderSide(color: colors.error),
        ),
        focusedErrorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(AppTheme.radiusSmall),
          borderSide: BorderSide(
              color: colors.error, width: AppTheme.borderSelected),
        ),
        filled: true,
        fillColor: colors.surface.withOpacity(AppTheme.opacityOverlay),
      ),
    );
  }
}
