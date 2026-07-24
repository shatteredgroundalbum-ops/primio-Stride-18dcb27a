import 'package:awesome_notifications/awesome_notifications.dart';
import 'package:flutter/material.dart';

class StrideNotificationService {
  static Future<void> initialize() async {
    await AwesomeNotifications().initialize(
      null,
      [
        NotificationChannel(
          channelKey: 'stride_reminders',
          channelName: 'Workout Reminders',
          channelDescription: 'Daily workout and walking reminders',
          defaultColor: const Color(0xFF6C63FF),
          importance: NotificationImportance.High,
        ),
        NotificationChannel(
          channelKey: 'stride_achievements',
          channelName: 'Achievements',
          channelDescription: 'Goal and milestone notifications',
          defaultColor: const Color(0xFF00E676),
          importance: NotificationImportance.Default,
        ),
      ],
    );
  }

  static Future<void> requestPermission() async {
    await AwesomeNotifications().requestPermissionToSendNotifications();
  }

  static Future<void> scheduleDailyReminder({
    required int hour,
    required int minute,
  }) async {
    await AwesomeNotifications().createNotification(
      content: NotificationContent(
        id: 1,
        channelKey: 'stride_reminders',
        title: 'Time to move! 🏃',
        body:
            'Your daily walking goal is waiting. Let\'s make today count!',
      ),
      schedule: NotificationCalendar(
        hour: hour,
        minute: minute,
        second: 0,
        repeats: true,
      ),
    );
  }

  static Future<void> showAchievementNotification({
    required String title,
    required String body,
  }) async {
    await AwesomeNotifications().createNotification(
      content: NotificationContent(
        id: DateTime.now().millisecondsSinceEpoch.remainder(100000),
        channelKey: 'stride_achievements',
        title: title,
        body: body,
      ),
    );
  }

  static Future<void> showWorkoutCompleteNotification({
    required double distanceKm,
    required Duration duration,
    required double calories,
  }) async {
    final mins = duration.inMinutes;
    await AwesomeNotifications().createNotification(
      content: NotificationContent(
        id: DateTime.now().millisecondsSinceEpoch.remainder(100000),
        channelKey: 'stride_achievements',
        title: 'Workout Complete! 🎉',
        body:
            '${distanceKm.toStringAsFixed(1)} km in $mins min — ${calories.toStringAsFixed(0)} cal burned',
      ),
    );
  }

  static Future<void> cancelAll() async {
    await AwesomeNotifications().cancelAll();
  }
}
