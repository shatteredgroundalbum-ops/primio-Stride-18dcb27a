import 'package:shared_preferences/shared_preferences.dart';

/// Small device-local settings stored in SharedPreferences.
///
/// Use this for values that do NOT belong in Firestore or SQLite:
/// onboarding state, measurement units, UI preferences,
/// permission flags, last sync time, temporary session flags.
class PreferencesService {
  static const _keyOnboardingComplete = 'onboarding_complete';
  static const _keyUseMetric = 'use_metric';
  static const _keyLastActivePage = 'last_active_page';
  static const _keyVoiceCoachingEnabled = 'voice_coaching_enabled';
  static const _keyMusicMode = 'music_mode';
  static const _keyThemeMode = 'theme_mode';
  static const _keyDailyReminderEnabled = 'daily_reminder_enabled';
  static const _keyDailyReminderHour = 'daily_reminder_hour';
  static const _keyDailyReminderMinute = 'daily_reminder_minute';
  static const _keyLastSyncTime = 'last_sync_time';
  static const _keyLocationPermissionExplained = 'location_perm_explained';
  static const _keyNotificationPermissionExplained =
      'notification_perm_explained';

  SharedPreferences? _prefs;

  Future<SharedPreferences> get _instance async {
    _prefs ??= await SharedPreferences.getInstance();
    return _prefs!;
  }

  // ─── Onboarding ─────────────────────────────────────────────────

  Future<bool> isOnboardingComplete() async {
    final prefs = await _instance;
    return prefs.getBool(_keyOnboardingComplete) ?? false;
  }

  Future<void> setOnboardingComplete(bool value) async {
    final prefs = await _instance;
    await prefs.setBool(_keyOnboardingComplete, value);
  }

  // ─── Measurement Units ─────────────────────────────────────────

  Future<bool> useMetric() async {
    final prefs = await _instance;
    return prefs.getBool(_keyUseMetric) ?? true;
  }

  Future<void> setUseMetric(bool value) async {
    final prefs = await _instance;
    await prefs.setBool(_keyUseMetric, value);
  }

  // ─── Navigation State ──────────────────────────────────────────

  Future<String> lastActivePage() async {
    final prefs = await _instance;
    return prefs.getString(_keyLastActivePage) ?? '/dashboard';
  }

  Future<void> setLastActivePage(String path) async {
    final prefs = await _instance;
    await prefs.setString(_keyLastActivePage, path);
  }

  // ─── Voice Coaching ────────────────────────────────────────────

  Future<bool> isVoiceCoachingEnabled() async {
    final prefs = await _instance;
    return prefs.getBool(_keyVoiceCoachingEnabled) ?? true;
  }

  Future<void> setVoiceCoachingEnabled(bool value) async {
    final prefs = await _instance;
    await prefs.setBool(_keyVoiceCoachingEnabled, value);
  }

  // ─── Music Mode ────────────────────────────────────────────────

  Future<String> musicMode() async {
    final prefs = await _instance;
    return prefs.getString(_keyMusicMode) ?? 'auto';
  }

  Future<void> setMusicMode(String mode) async {
    final prefs = await _instance;
    await prefs.setString(_keyMusicMode, mode);
  }

  // ─── Theme ─────────────────────────────────────────────────────

  Future<String> themeMode() async {
    final prefs = await _instance;
    return prefs.getString(_keyThemeMode) ?? 'dark';
  }

  Future<void> setThemeMode(String mode) async {
    final prefs = await _instance;
    await prefs.setString(_keyThemeMode, mode);
  }

  // ─── Daily Reminder ────────────────────────────────────────────

  Future<bool> isDailyReminderEnabled() async {
    final prefs = await _instance;
    return prefs.getBool(_keyDailyReminderEnabled) ?? true;
  }

  Future<void> setDailyReminderEnabled(bool value) async {
    final prefs = await _instance;
    await prefs.setBool(_keyDailyReminderEnabled, value);
  }

  Future<({int hour, int minute})> dailyReminderTime() async {
    final prefs = await _instance;
    return (
      hour: prefs.getInt(_keyDailyReminderHour) ?? 8,
      minute: prefs.getInt(_keyDailyReminderMinute) ?? 0,
    );
  }

  Future<void> setDailyReminderTime(int hour, int minute) async {
    final prefs = await _instance;
    await prefs.setInt(_keyDailyReminderHour, hour);
    await prefs.setInt(_keyDailyReminderMinute, minute);
  }

  // ─── Sync ──────────────────────────────────────────────────────

  Future<DateTime?> lastSyncTime() async {
    final prefs = await _instance;
    final iso = prefs.getString(_keyLastSyncTime);
    return iso != null ? DateTime.tryParse(iso) : null;
  }

  Future<void> setLastSyncTime(DateTime time) async {
    final prefs = await _instance;
    await prefs.setString(_keyLastSyncTime, time.toIso8601String());
  }

  // ─── Permission Explanations ───────────────────────────────────

  Future<bool> isLocationPermissionExplained() async {
    final prefs = await _instance;
    return prefs.getBool(_keyLocationPermissionExplained) ?? false;
  }

  Future<void> setLocationPermissionExplained(bool value) async {
    final prefs = await _instance;
    await prefs.setBool(_keyLocationPermissionExplained, value);
  }

  Future<bool> isNotificationPermissionExplained() async {
    final prefs = await _instance;
    return prefs.getBool(_keyNotificationPermissionExplained) ?? false;
  }

  Future<void> setNotificationPermissionExplained(bool value) async {
    final prefs = await _instance;
    await prefs.setBool(_keyNotificationPermissionExplained, value);
  }

  // ─── Clear All ─────────────────────────────────────────────────

  /// Clears all preferences (e.g. on logout).
  Future<void> clearAll() async {
    final prefs = await _instance;
    await prefs.clear();
  }
}
