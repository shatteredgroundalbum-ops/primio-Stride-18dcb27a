import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';

import '../../providers/auth_provider.dart';
import '../../theme/theme.dart';
import '../../widgets/auth/auth_text_field.dart';
import '../../widgets/auth/stride_logo.dart';
import '../../widgets/common/glass_card.dart';

class LoginScreen extends StatefulWidget {
  const LoginScreen({super.key});

  @override
  State<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends State<LoginScreen> {
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();
  final _formKey = GlobalKey<FormState>();

  @override
  void dispose() {
    _emailController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  Future<void> _handleLogin() async {
    if (!_formKey.currentState!.validate()) return;

    final authProvider = context.read<AuthProvider>();
    await authProvider.login(
      _emailController.text.trim(),
      _passwordController.text,
    );
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
                  const StrideLogo().animate().fadeIn(duration: 600.ms).scale(
                      begin: const Offset(0.8, 0.8),
                      end: const Offset(1, 1)),
                  const SizedBox(height: AppTheme.spacingXl),
                  Text('Welcome Back', style: text.headlineMedium)
                      .animate()
                      .fadeIn(duration: 500.ms, delay: 200.ms),
                  const SizedBox(height: AppTheme.spacingSm),
                  Text('Sign in to continue your journey',
                          style: text.bodySmall)
                      .animate()
                      .fadeIn(duration: 500.ms, delay: 300.ms),
                  const SizedBox(height: AppTheme.spacingXl),
                  GlassCard(
                    child: Column(
                      children: [
                        AuthTextField(
                          controller: _emailController,
                          label: 'Email',
                          icon: Icons.email_outlined,
                          keyboardType: TextInputType.emailAddress,
                          validator: (v) => v == null || !v.contains('@')
                              ? 'Enter a valid email'
                              : null,
                        ),
                        const SizedBox(height: AppTheme.spacingMd),
                        AuthTextField(
                          controller: _passwordController,
                          label: 'Password',
                          icon: Icons.lock_outline,
                          obscureText: true,
                          validator: (v) => v == null || v.length < 6
                              ? 'Minimum 6 characters'
                              : null,
                        ),
                        const SizedBox(height: AppTheme.spacingSm),
                        Align(
                          alignment: Alignment.centerRight,
                          child: TextButton(
                            onPressed: () =>
                                context.push('/auth/forgot-password'),
                            child: Text('Forgot Password?',
                                style: text.labelMedium
                                    ?.copyWith(color: colors.primary)),
                          ),
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
                            onPressed:
                                authProvider.isLoading ? null : _handleLogin,
                            child: authProvider.isLoading
                                ? SizedBox(
                                    width: AppTheme.iconMd,
                                    height: AppTheme.iconMd,
                                    child: CircularProgressIndicator(
                                        strokeWidth: 2,
                                        color: colors.onPrimary))
                                : const Text('Sign In'),
                          ),
                        ),
                      ],
                    ),
                  )
                      .animate()
                      .fadeIn(duration: 500.ms, delay: 400.ms)
                      .slideY(begin: 0.1, end: 0),
                  const SizedBox(height: AppTheme.spacingLg),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text("Don't have an account? ",
                          style: text.bodySmall),
                      GestureDetector(
                        onTap: () => context.go('/auth/register'),
                        child: Text('Sign Up',
                            style: text.bodySmall?.copyWith(
                                color: colors.primary,
                                fontWeight: FontWeight.w700)),
                      ),
                    ],
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
