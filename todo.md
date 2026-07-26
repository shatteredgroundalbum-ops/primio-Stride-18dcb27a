# §13 Notifications — Rust Engine Implementation

## Rust Engine
- [x] Create `rust/stride_engine/src/engine/notifications.rs` with:
  - NotificationCategory enum (reminders, achievements, coaching, system, recovery)
  - NotificationType enum (11 types)
  - NotificationPriority enum (high, default, low)
  - NotificationPermission (not_requested, granted, denied, permanently_denied)
  - NotificationPreferences (per-category toggles, all default false)
  - TimezoneContext (utc_offset_seconds, is_dst, local_minute_of_day)
  - QuietHoursConfig + QuietHoursDecision (wrap-past-midnight support)
  - evaluate_quiet_hours()
  - NotificationContent + build_notification_content()
  - NotificationContextParams
  - NotificationSchedule (immediate, scheduled, daily)
  - NotificationRequest
  - CoachingDeliveryMode (none, visual, voice, both)
  - VoiceCoachingConfig + should_announce()
  - CoachingAnnouncementKind (6 kinds)
  - NotificationDecision + NotificationDecisionContext + decide_notification()
  - NotificationStatus + build_notification_status()
  - next_daily_reminder_utc_ms() + next_weekly_reminder_utc_ms()
  - 100 unit tests

## FFI Layer
- [x] Add `pub mod notifications;` to mod.rs
- [x] Add import block + 6 FFI entry points to ffi.rs:
  1. stride_notification_decide
  2. stride_notification_content
  3. stride_notification_quiet_hours
  4. stride_notification_coaching
  5. stride_notification_status
  6. stride_notification_next_reminder

## Dart Bindings
- [x] Add 6 typedefs + 6 lookups + 6 late final fields + 6 wrapper methods to stride_engine_bindings.dart
- [x] Add notification enums + classes to stride_models.dart
- [x] Add 6 ergonomic static methods to stride_engine_client.dart

## Verification & Push
- [x] Compile Rust tests pass (940 passed, 0 failed)
- [x] Verify Dart brace/paren/bracket balance (all 0)
- [ ] Commit and push to GitHub
