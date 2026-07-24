import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'router/app_router.dart';
import 'services/health_service.dart';
import 'theme/theme.dart';

void main() {
  runApp(const StrideApp());
}

class StrideApp extends StatelessWidget {
  const StrideApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        Provider(create: (_) => HealthService()),
      ],
      child: MaterialApp.router(
        title: 'S.T.R.I.D.E.',
        theme: AppTheme.darkTheme,
        routerConfig: AppRouter.router,
      ),
    );
  }
}