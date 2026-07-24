import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'providers/auth_provider.dart';
import 'repositories/auth_repository.dart';
import 'repositories/user_repository.dart';
import 'router/app_router.dart';
import 'services/auth_service.dart';
import 'services/health_service.dart';
import 'services/stride_notification_service.dart';
import 'theme/theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await StrideNotificationService.initialize();

  final authRepository = MockAuthRepository();
  final userRepository = UserRepository();
  final authService = AuthService(
    authRepository: authRepository,
    userRepository: userRepository,
  );
  final authProvider = AuthProvider(authService: authService);

  runApp(StrideApp(authProvider: authProvider));
}

class StrideApp extends StatelessWidget {
  final AuthProvider authProvider;

  const StrideApp({super.key, required this.authProvider});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider.value(value: authProvider),
        Provider(create: (_) => HealthService()),
      ],
      child: MaterialApp.router(
        title: 'S.T.R.I.D.E.',
        theme: AppTheme.darkTheme,
        routerConfig: AppRouter.createRouter(authProvider),
      ),
    );
  }
}