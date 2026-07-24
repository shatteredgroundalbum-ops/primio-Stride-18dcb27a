import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../providers/auth_provider.dart';
import '../../theme/theme.dart';
import '../../widgets/auth/auth_text_field.dart';
import '../../widgets/common/glass_card.dart';

class ForgotPasswordScreen extends StatefulWidget {
  const ForgotPasswordScreen({super.key});

  @override
  State<ForgotPasswordScreen> createState() => _ForgotPasswordScreenState();
}

class _ForgotPasswordScreenState extends State<ForgotPasswordScreen> {
  final _emailController = TextEditingController();
  final _formKey = GlobalKey<FormState>();
  bool _emailSent = false;

  @override
  void dispose() {
    _emailController.dispose();
    super.dispose();
  }

  Future<void> _handleReset() async {
    if (!_formKey.currentState!.validate()) return;

    final authProvider = context.read<AuthProvider>();
    final success =
        await authProvider.resetPassword(_emailController.text.trim());
    if (success && mounted) {
      setState(() => _emailSent = true);
    }
  }

  @override
  Widget build(BuildContext context) {
    final authProvider = context.watch<AuthProvider>();
    final colors = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;

    return Scaffold(
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(AppTheme.spacingLg),
            child: Form(
              key: _formKey,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.lock_reset,
                          size: AppTheme.iconXl * 1.5, color: colors.primary)
                      .animate()
                      .fadeIn(duration: 600.ms),
                  const SizedBox(height: AppTheme.spacingLg),
                  Text('Reset Password', style: text.headlineMedium)
                      .animate()
                      .fadeIn(duration: 500.ms, delay: 200.ms),
                  const SizedBox(height: AppTheme.spacingSm),
                  Text(
                    'Enter your email and we\'ll send you a reset link',
                    style: text.bodySmall,
                    textAlign: TextAlign.center,
                  ).animate().fadeIn(duration: 500.ms, delay: 300.ms),
                  const SizedBox(height: AppTheme.spacingXl),
                  if (_emailSent)
                    GlassCard(
                      borderColor: colors.primary.withOpacity(0.3),
                      child: Column(
                        children: [
                          Icon(Icons.check_circle,
                              size: AppTheme.iconXl,
                              color: Theme.of(context)
                                  .extension<AppColorsExtension>()!
                                  .success),
                          const SizedBox(height: AppTheme.spacingMd),
                          Text('Reset Email Sent',
                              style: text.titleMedium),
                          const SizedBox(height: AppTheme.spacingSm),
                          Text(
                            'Check your inbox for the password reset link.',
                            style: text.bodySmall,
                            textAlign: TextAlign.center,
                          ),
                          const SizedBox(height: AppTheme.spacingLg),
                          SizedBox(
                            width: double.infinity,
                            height: AppTheme.buttonHeight,
                            child: ElevatedButton(
                              onPressed: () => context.go('/auth/login'),
                              child: const Text('Back to Sign In'),
                            ),
                          ),
                        ],
                      ),
                    ).animate().fadeIn(duration: 500.ms).scale(
                        begin: const Offset(0.95, 0.95),
                        end: const Offset(1, 1))
                  else
                    GlassCard(
                      child: Column(
                        children: [
                          AuthTextField(
                            controller: _emailController,
                            label: 'Email',
                            icon: Icons.email_outlined,
                            keyboardType: TextInputType.emailAddress,
                            validator: (v) =>
                                v == null || !v.contains('@')
                                    ? 'Enter a valid email'
                                    : null,
                          ),
                          const SizedBox(height: AppTheme.spacingMd),
                          if (authProvider.error != null)
                            Padding(
                              padding: const EdgeInsets.only(
                                  bottom: AppTheme.spacingMd),
                              child: Text(authProvider.error!,
                                  style: text.bodySmall
                                      ?.copyWith(color: colors.error),
                                  textAlign: TextAlign.center),
                            ),
                          SizedBox(
                            width: double.infinity,
                            height: AppTheme.buttonHeight,
                            child: ElevatedButton(
                              onPressed: authProvider.isLoading
                                  ? null
                                  : _handleReset,
                              child: authProvider.isLoading
                                  ? SizedBox(
                                      width: AppTheme.iconMd,
                                      height: AppTheme.iconMd,
                                      child: CircularProgressIndicator(
                                          strokeWidth: 2,
                                          color: colors.onPrimary))
                                  : const Text('Send Reset Link'),
                            ),
                          ),
                        ],
                      ),
                    )
                        .animate()
                        .fadeIn(duration: 500.ms, delay: 400.ms)
                        .slideY(begin: 0.1, end: 0),
                  const SizedBox(height: AppTheme.spacingLg),
                  GestureDetector(
                    onTap: () => context.go('/auth/login'),
                    child: Text('Back to Sign In',
                        style: text.bodySmall?.copyWith(
                            color: colors.primary,
                            fontWeight: FontWeight.w700)),
                  ).animate().fadeIn(duration: 500.ms, delay: 500.ms),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
