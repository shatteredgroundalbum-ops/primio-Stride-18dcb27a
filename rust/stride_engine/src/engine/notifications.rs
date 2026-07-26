//! Notifications and coaching delivery (spec section 13).
//!
//! This module is the pure decision logic for *what* notifications to
//! show, *when* to show them, and *whether* the user has permitted them.
//! The actual notification posting (via `awesome_notifications` on the
//! Dart side, or the Android notification manager on the Kotlin side)
//! lives outside this module — this module centralises the *decisions* so
//! they are consistent, testable, and independent of platform APIs.
//!
//! Key concepts:
//!
//! - **Notification categories** — the user can independently enable /
//!   disable each category of notification (workout reminders,
//!   achievements, coaching, system, sync, recovery). No category is
//!   enabled by default; the user must opt in (spec: "Do not send every
//!   possible reminder by default. Users must be able to control each
//!   category.").
//! - **Notification types** — the specific notification to deliver
//!   (scheduled-workout reminder, plan reminder, missed-workout
//!   follow-up, goal milestone, streak warning, device disconnected, sync
//!   failed, workout recovered, badge earned, plan updated, voice
//!   coaching). Each type maps to a category.
//! - **Quiet hours** — a configurable time window during which
//!   non-critical notifications are suppressed (deferred to the next
//!   available time). Critical notifications (sync failed, workout
//!   recovered) may still fire during quiet hours if the user allows.
//! - **Time-zone handling** — quiet hours are evaluated in the user's
//!   local time zone, not UTC. The engine receives a UTC offset and DST
//!   flag and converts accordingly.
//! - **Coaching delivery** — voice coaching is optional. The user can
//!   choose none, visual-only, voice-only, or both. Voice coaching
//!   announcements are gated by the user's coaching preferences and
//!   quiet hours (no voice at 3 AM).
//! - **Permission** — the user must grant notification permission before
//!   any notification is delivered. If permission is not granted, no
//!   notifications fire (but the decision logic still runs so the app
//!   knows what *would* have been shown).

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────
// Notification categories
// ──────────────────────────────────────────────────────────────────────

/// The broad category of a notification. Each category can be
/// independently enabled or disabled by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    /// Scheduled workout reminders, plan reminders, missed-workout
    /// follow-ups.
    Reminders,
    /// Goal milestones, badge earned, streak warnings.
    Achievements,
    /// Voice coaching prompts, training-plan updates.
    Coaching,
    /// Device disconnected, sync failed.
    System,
    /// Workout recovered from a crash.
    Recovery,
}

impl NotificationCategory {
    /// Human-readable label for the settings UI.
    pub fn label(self) -> &'static str {
        match self {
            NotificationCategory::Reminders => "Workout reminders",
            NotificationCategory::Achievements => "Achievements",
            NotificationCategory::Coaching => "Coaching",
            NotificationCategory::System => "System alerts",
            NotificationCategory::Recovery => "Recovery alerts",
        }
    }

    /// A short description for the settings toggle.
    pub fn description(self) -> &'static str {
        match self {
            NotificationCategory::Reminders => {
                "Scheduled workout reminders, plan reminders, and missed-workout follow-ups."
            }
            NotificationCategory::Achievements => {
                "Goal milestones, badges, and streak warnings."
            }
            NotificationCategory::Coaching => {
                "Voice and visual coaching prompts during workouts."
            }
            NotificationCategory::System => {
                "Device disconnection and sync-failure alerts."
            }
            NotificationCategory::Recovery => {
                "Alerts when a workout is recovered after a crash."
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Notification types
// ──────────────────────────────────────────────────────────────────────

/// The specific notification to deliver. Each type maps to a
/// [`NotificationCategory`] and has a default priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// It's time for your scheduled workout.
    ScheduledWorkout,
    /// Your training plan has a new or updated workout today.
    PlanReminder,
    /// You missed a planned workout — follow-up encouragement.
    MissedWorkout,
    /// You reached a goal milestone (distance, duration, calories).
    GoalMilestone,
    /// Your streak is at risk — you haven't worked out recently.
    StreakWarning,
    /// Your wearable device disconnected during a workout.
    DeviceDisconnected,
    /// A sync attempt failed and will be retried.
    SyncFailed,
    /// A workout was recovered after a crash or process kill.
    WorkoutRecovered,
    /// You earned a new badge or achievement.
    BadgeEarned,
    /// Your training plan was updated by the AI coach.
    PlanUpdated,
    /// A voice coaching prompt (distance, pace, split, etc.).
    VoiceCoaching,
}

impl NotificationType {
    /// The category this notification type belongs to.
    pub fn category(self) -> NotificationCategory {
        match self {
            NotificationType::ScheduledWorkout => NotificationCategory::Reminders,
            NotificationType::PlanReminder => NotificationCategory::Reminders,
            NotificationType::MissedWorkout => NotificationCategory::Reminders,
            NotificationType::GoalMilestone => NotificationCategory::Achievements,
            NotificationType::StreakWarning => NotificationCategory::Achievements,
            NotificationType::BadgeEarned => NotificationCategory::Achievements,
            NotificationType::PlanUpdated => NotificationCategory::Coaching,
            NotificationType::VoiceCoaching => NotificationCategory::Coaching,
            NotificationType::DeviceDisconnected => NotificationCategory::System,
            NotificationType::SyncFailed => NotificationCategory::System,
            NotificationType::WorkoutRecovered => NotificationCategory::Recovery,
        }
    }

    /// Default priority for this notification type.
    pub fn default_priority(self) -> NotificationPriority {
        match self {
            NotificationType::WorkoutRecovered => NotificationPriority::High,
            NotificationType::DeviceDisconnected => NotificationPriority::High,
            NotificationType::SyncFailed => NotificationPriority::Default,
            NotificationType::ScheduledWorkout => NotificationPriority::Default,
            NotificationType::PlanReminder => NotificationPriority::Default,
            NotificationType::MissedWorkout => NotificationPriority::Default,
            NotificationType::GoalMilestone => NotificationPriority::Default,
            NotificationType::StreakWarning => NotificationPriority::Default,
            NotificationType::BadgeEarned => NotificationPriority::Default,
            NotificationType::PlanUpdated => NotificationPriority::Default,
            NotificationType::VoiceCoaching => NotificationPriority::Low,
        }
    }

    /// Whether this type is critical enough to fire during quiet hours
    /// (when the user has enabled quiet-hours bypass for critical
    /// notifications).
    pub fn is_critical(self) -> bool {
        matches!(
            self,
            NotificationType::WorkoutRecovered
                | NotificationType::DeviceDisconnected
                | NotificationType::SyncFailed
        )
    }

    /// Human-readable label for diagnostics / logging.
    pub fn label(self) -> &'static str {
        match self {
            NotificationType::ScheduledWorkout => "Scheduled workout reminder",
            NotificationType::PlanReminder => "Plan reminder",
            NotificationType::MissedWorkout => "Missed workout follow-up",
            NotificationType::GoalMilestone => "Goal milestone",
            NotificationType::StreakWarning => "Streak warning",
            NotificationType::DeviceDisconnected => "Device disconnected",
            NotificationType::SyncFailed => "Sync failed",
            NotificationType::WorkoutRecovered => "Workout recovered",
            NotificationType::BadgeEarned => "Badge earned",
            NotificationType::PlanUpdated => "Training plan updated",
            NotificationType::VoiceCoaching => "Voice coaching",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Notification priority
// ──────────────────────────────────────────────────────────────────────

/// The importance / urgency of a notification. Maps to Android
/// notification channel importance levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    /// High importance — makes a sound and appears as a heads-up
    /// notification. Used for critical alerts.
    High,
    /// Default importance — visible in the notification shade.
    Default,
    /// Low importance — minimal disruption, no sound by default.
    Low,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        NotificationPriority::Default
    }
}

impl NotificationPriority {
    pub fn label(self) -> &'static str {
        match self {
            NotificationPriority::High => "high",
            NotificationPriority::Default => "default",
            NotificationPriority::Low => "low",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Notification permission state
// ──────────────────────────────────────────────────────────────────────

/// Whether the user has granted notification permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPermission {
    /// Permission has not been requested yet.
    NotRequested,
    /// Permission was granted.
    Granted,
    /// Permission was denied.
    Denied,
    /// Permission was denied and the user selected "Don't ask again."
    PermanentlyDenied,
}

impl Default for NotificationPermission {
    fn default() -> Self {
        NotificationPermission::NotRequested
    }
}

impl NotificationPermission {
    /// Returns `true` if notifications can be delivered.
    pub fn is_granted(self) -> bool {
        matches!(self, NotificationPermission::Granted)
    }

    /// Returns `true` if the app should prompt the user to request
    /// permission (i.e., it hasn't been denied permanently).
    pub fn should_request(self) -> bool {
        matches!(
            self,
            NotificationPermission::NotRequested | NotificationPermission::Denied
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            NotificationPermission::NotRequested => "not_requested",
            NotificationPermission::Granted => "granted",
            NotificationPermission::Denied => "denied",
            NotificationPermission::PermanentlyDenied => "permanently_denied",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// User preferences (per-category toggles)
// ──────────────────────────────────────────────────────────────────────

/// The user's per-category notification preferences. Each category can
/// be independently enabled or disabled. **All categories default to
/// `false`** — the user must opt in (spec: "Do not send every possible
/// reminder by default.").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Whether workout reminders are enabled.
    pub reminders_enabled: bool,
    /// Whether achievement notifications are enabled.
    pub achievements_enabled: bool,
    /// Whether coaching notifications are enabled.
    pub coaching_enabled: bool,
    /// Whether system alerts are enabled.
    pub system_enabled: bool,
    /// Whether recovery alerts are enabled.
    pub recovery_enabled: bool,
    /// Whether critical notifications (sync failed, device
    /// disconnected, workout recovered) bypass quiet hours.
    pub critical_bypasses_quiet_hours: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        // All off by default — the user must opt in to each category.
        Self {
            reminders_enabled: false,
            achievements_enabled: false,
            coaching_enabled: false,
            system_enabled: false,
            recovery_enabled: false,
            critical_bypasses_quiet_hours: true,
        }
    }
}

impl NotificationPreferences {
    /// Returns `true` if the given category is enabled.
    pub fn is_category_enabled(&self, category: NotificationCategory) -> bool {
        match category {
            NotificationCategory::Reminders => self.reminders_enabled,
            NotificationCategory::Achievements => self.achievements_enabled,
            NotificationCategory::Coaching => self.coaching_enabled,
            NotificationCategory::System => self.system_enabled,
            NotificationCategory::Recovery => self.recovery_enabled,
        }
    }

    /// Sets a category's enabled state and returns a new preferences
    /// struct (immutable update).
    pub fn with_category_enabled(
        mut self,
        category: NotificationCategory,
        enabled: bool,
    ) -> Self {
        match category {
            NotificationCategory::Reminders => self.reminders_enabled = enabled,
            NotificationCategory::Achievements => self.achievements_enabled = enabled,
            NotificationCategory::Coaching => self.coaching_enabled = enabled,
            NotificationCategory::System => self.system_enabled = enabled,
            NotificationCategory::Recovery => self.recovery_enabled = enabled,
        }
        self
    }

    /// Returns `true` if the given notification type should be delivered,
    /// based solely on the user's category preferences (ignoring quiet
    /// hours and permission).
    pub fn is_type_enabled(&self, notif_type: NotificationType) -> bool {
        self.is_category_enabled(notif_type.category())
    }

    /// Returns the number of categories the user has enabled.
    pub fn enabled_count(&self) -> usize {
        [
            self.reminders_enabled,
            self.achievements_enabled,
            self.coaching_enabled,
            self.system_enabled,
            self.recovery_enabled,
        ]
        .iter()
        .filter(|&&b| b)
        .count()
    }

    /// Returns `true` if all categories are disabled.
    pub fn all_disabled(&self) -> bool {
        self.enabled_count() == 0
    }
}

// ──────────────────────────────────────────────────────────────────────
// Time-zone context
// ──────────────────────────────────────────────────────────────────────

/// The user's time-zone context for evaluating quiet hours and
/// scheduling notifications in local time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimezoneContext {
    /// The UTC offset in seconds (e.g., -18000 for UTC-5, 3600 for
    /// UTC+1). Positive = east of UTC, negative = west.
    pub utc_offset_seconds: i32,
    /// Whether daylight saving time is currently in effect.
    pub is_dst: bool,
}

impl Default for TimezoneContext {
    fn default() -> Self {
        Self {
            utc_offset_seconds: 0,
            is_dst: false,
        }
    }
}

impl TimezoneContext {
    /// Converts a UTC timestamp (epoch milliseconds) to local-time
    /// minutes-since-midnight.
    ///
    /// Returns the minute-of-day (0–1439) in the user's local time.
    pub fn local_minute_of_day(self, utc_ms: i64) -> u32 {
        let local_ms = utc_ms + (self.utc_offset_seconds as i64 * 1000);
        // Normalize to a single day (86400000 ms per day).
        let day_ms = 86_400_000i64;
        let normalized = local_ms.rem_euclid(day_ms);
        (normalized / 60_000) as u32
    }

    /// Returns the local hour (0–23).
    pub fn local_hour(self, utc_ms: i64) -> u32 {
        self.local_minute_of_day(utc_ms) / 60
    }

    /// A human-readable timezone label (e.g., "UTC+1", "UTC-5").
    pub fn label(self) -> String {
        let offset = self.utc_offset_seconds;
        if offset == 0 {
            "UTC".to_string()
        } else if offset > 0 {
            format!("UTC+{}", offset / 3600)
        } else {
            format!("UTC{}", offset / 3600)
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Quiet hours
// ──────────────────────────────────────────────────────────────────────

/// Configuration for quiet hours — a time window during which
/// non-critical notifications are suppressed.
///
/// Times are in local-time minutes-since-midnight (0–1439). A window
/// that wraps past midnight (e.g., 22:00–07:00) is supported: if
/// `start_minute > end_minute`, the quiet period spans midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuietHoursConfig {
    /// Whether quiet hours are enabled at all.
    pub enabled: bool,
    /// Start of the quiet period in local-time minutes (0–1439).
    pub start_minute: u32,
    /// End of the quiet period in local-time minutes (0–1439).
    pub end_minute: u32,
}

impl Default for QuietHoursConfig {
    fn default() -> Self {
        // Default: 22:00–07:00 (10 PM to 7 AM).
        Self {
            enabled: false,
            start_minute: 22 * 60,
            end_minute: 7 * 60,
        }
    }
}

impl QuietHoursConfig {
    /// Returns `true` if the given local-time minute falls within the
    /// quiet hours window.
    ///
    /// If `start_minute <= end_minute`, the window is a simple range
    /// (e.g., 600–900 = 10:00–15:00).
    /// If `start_minute > end_minute`, the window wraps past midnight
    /// (e.g., 1320–420 = 22:00–07:00).
    pub fn is_quiet_at(self, local_minute: u32) -> bool {
        if !self.enabled {
            return false;
        }
        if self.start_minute <= self.end_minute {
            // Simple range (doesn't cross midnight).
            local_minute >= self.start_minute && local_minute < self.end_minute
        } else {
            // Wraps past midnight.
            local_minute >= self.start_minute || local_minute < self.end_minute
        }
    }

    /// Returns the local-time minute when the quiet period ends (i.e.,
    /// when notifications can resume). If quiet hours are disabled,
    /// returns the current minute (no delay).
    pub fn resume_minute(self, current_local_minute: u32) -> u32 {
        if !self.enabled || !self.is_quiet_at(current_local_minute) {
            return current_local_minute;
        }
        self.end_minute
    }
}

/// The result of a quiet-hours evaluation for a specific notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuietHoursDecision {
    /// Whether the notification is currently suppressed by quiet hours.
    pub is_quiet: bool,
    /// Whether the notification should be delivered now (considering
    /// quiet hours and the critical-bypass setting).
    pub should_deliver_now: bool,
    /// If the notification is deferred, the local-time minute when it
    /// can be delivered. `None` if not deferred.
    pub reschedule_to_minute: Option<u32>,
    /// A human-readable explanation.
    pub reason: String,
}

/// Evaluates whether a notification should be delivered now, given the
/// quiet-hours configuration, the notification type (for critical
/// bypass), and the current local-time minute.
pub fn evaluate_quiet_hours(
    config: QuietHoursConfig,
    notif_type: NotificationType,
    local_minute: u32,
    critical_bypass: bool,
) -> QuietHoursDecision {
    if !config.enabled {
        return QuietHoursDecision {
            is_quiet: false,
            should_deliver_now: true,
            reschedule_to_minute: None,
            reason: "Quiet hours are not enabled.".to_string(),
        };
    }

    let is_quiet = config.is_quiet_at(local_minute);

    if !is_quiet {
        return QuietHoursDecision {
            is_quiet: false,
            should_deliver_now: true,
            reschedule_to_minute: None,
            reason: "Outside quiet hours.".to_string(),
        };
    }

    // It's quiet. Check if this is a critical notification that bypasses.
    if notif_type.is_critical() && critical_bypass {
        return QuietHoursDecision {
            is_quiet: true,
            should_deliver_now: true,
            reschedule_to_minute: None,
            reason: "Critical notification bypasses quiet hours.".to_string(),
        };
    }

    // Suppress and defer.
    let resume = config.resume_minute(local_minute);
    QuietHoursDecision {
        is_quiet: true,
        should_deliver_now: false,
        reschedule_to_minute: Some(resume),
        reason: format!(
            "Deferred to {:02}:{:02} (quiet hours).",
            resume / 60,
            resume % 60
        ),
    }
}

// ──────────────────────────────────────────────────────────────────────
// Notification content
// ──────────────────────────────────────────────────────────────────────

/// The content of a notification to display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationContent {
    /// The notification type.
    pub notif_type: NotificationType,
    /// The category.
    pub category: NotificationCategory,
    /// The priority.
    pub priority: NotificationPriority,
    /// The title text.
    pub title: String,
    /// The body text.
    pub body: String,
    /// An optional action button label (e.g., "Start workout").
    pub action_label: Option<String>,
    /// Optional data payload for the action (e.g., workout ID).
    pub action_data: Option<String>,
}

/// Builds the notification content (title, body, action label) for a
/// given notification type, with optional context parameters.
///
/// `context_params` is a map of optional context values that can be
/// used to customize the notification text (e.g., workout time, goal
/// name, streak count, device name, etc.).
pub fn build_notification_content(
    notif_type: NotificationType,
    context_params: &NotificationContextParams,
) -> NotificationContent {
    let (title, body, action_label) = match notif_type {
        NotificationType::ScheduledWorkout => (
            "Time to move! 🏃".to_string(),
            format!(
                "Your scheduled workout{} is waiting. Let's make today count!",
                context_params
                    .workout_time
                    .as_ref()
                    .map(|t| format!(" at {t}"))
                    .unwrap_or_default()
            ),
            Some("Start workout".to_string()),
        ),
        NotificationType::PlanReminder => (
            "Today's plan 📋".to_string(),
            format!(
                "You have a workout scheduled for today{}.",
                context_params
                    .workout_type
                    .as_ref()
                    .map(|t| format!(": {t}"))
                    .unwrap_or_default()
            ),
            Some("View plan".to_string()),
        ),
        NotificationType::MissedWorkout => (
            "Missed workout 💪".to_string(),
            "You missed your last planned workout. Don't worry — let's get back on track tomorrow!".to_string(),
            Some("Reschedule".to_string()),
        ),
        NotificationType::GoalMilestone => (
            "Goal reached! 🎯".to_string(),
            format!(
                "Congratulations! You've reached your goal{}!",
                context_params
                    .goal_description
                    .as_ref()
                    .map(|g| format!(": {g}"))
                    .unwrap_or_default()
            ),
            Some("View details".to_string()),
        ),
        NotificationType::StreakWarning => (
            "Streak at risk! ⚠️".to_string(),
            format!(
                "Your {}-day streak is at risk. Take a walk today to keep it alive!",
                context_params.streak_days.unwrap_or(1)
            ),
            Some("Start workout".to_string()),
        ),
        NotificationType::DeviceDisconnected => (
            "Device disconnected 📴".to_string(),
            format!(
                "Your {} disconnected. Workout data may be incomplete.",
                context_params
                    .device_name
                    .as_ref()
                    .map(|d| d.clone())
                    .unwrap_or_else(|| "wearable device".to_string())
            ),
            Some("Reconnect".to_string()),
        ),
        NotificationType::SyncFailed => (
            "Sync failed 🔄".to_string(),
            "Your last workout couldn't be synced to the cloud. We'll retry automatically. Your data is safe locally.".to_string(),
            Some("Retry now".to_string()),
        ),
        NotificationType::WorkoutRecovered => (
            "Workout recovered! ✅".to_string(),
            "We recovered your workout after an unexpected interruption. Review and save it to keep your progress.".to_string(),
            Some("Review workout".to_string()),
        ),
        NotificationType::BadgeEarned => (
            "New badge! 🏅".to_string(),
            format!(
                "You earned a new badge{}!",
                context_params
                    .badge_name
                    .as_ref()
                    .map(|b| format!(": {b}"))
                    .unwrap_or_default()
            ),
            Some("View badges".to_string()),
        ),
        NotificationType::PlanUpdated => (
            "Plan updated 📝".to_string(),
            "Your training plan has been updated. Check out what's next!".to_string(),
            Some("View plan".to_string()),
        ),
        NotificationType::VoiceCoaching => (
            "Coaching".to_string(),
            context_params
                .coaching_message
                .clone()
                .unwrap_or_else(|| "Keep going!".to_string()),
            None,
        ),
    };

    NotificationContent {
        notif_type,
        category: notif_type.category(),
        priority: notif_type.default_priority(),
        title,
        body,
        action_label,
        action_data: context_params.action_data.clone(),
    }
}

/// Optional context parameters for building notification content.
/// Any field can be `None` if not relevant to the notification type.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationContextParams {
    /// Scheduled workout time (e.g., "6:00 PM").
    pub workout_time: Option<String>,
    /// Workout type for plan reminders (e.g., "Morning walk").
    pub workout_type: Option<String>,
    /// Goal description for goal milestones (e.g., "5K in under 30 min").
    pub goal_description: Option<String>,
    /// Number of days in the current streak.
    pub streak_days: Option<u32>,
    /// Name of the disconnected device.
    pub device_name: Option<String>,
    /// Name of the earned badge.
    pub badge_name: Option<String>,
    /// Voice coaching message text.
    pub coaching_message: Option<String>,
    /// Optional action data payload (e.g., workout ID, badge ID).
    pub action_data: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────
// Notification scheduling
// ──────────────────────────────────────────────────────────────────────

/// When a notification should be delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSchedule {
    /// Deliver immediately (or as soon as quiet hours allow).
    Immediate,
    /// Deliver at a specific UTC timestamp (epoch milliseconds).
    Scheduled { at_utc_ms: i64 },
    /// Repeat daily at a specific local-time minute.
    Daily { at_local_minute: u32 },
}

/// A fully specified notification request: what to show, when, and
/// with what content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationRequest {
    /// The notification type.
    pub notif_type: NotificationType,
    /// The scheduling mode.
    pub schedule: NotificationSchedule,
    /// Optional context for building content.
    pub context: NotificationContextParams,
}

// ──────────────────────────────────────────────────────────────────────
// Coaching delivery
// ──────────────────────────────────────────────────────────────────────

/// How coaching prompts are delivered to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachingDeliveryMode {
    /// No coaching prompts.
    None,
    /// Visual-only (on-screen prompts, no audio).
    Visual,
    /// Voice-only (spoken prompts, no on-screen).
    Voice,
    /// Both visual and voice.
    Both,
}

impl Default for CoachingDeliveryMode {
    fn default() -> Self {
        CoachingDeliveryMode::None
    }
}

impl CoachingDeliveryMode {
    /// Returns `true` if voice coaching is enabled.
    pub fn has_voice(self) -> bool {
        matches!(self, CoachingDeliveryMode::Voice | CoachingDeliveryMode::Both)
    }

    /// Returns `true` if visual coaching is enabled.
    pub fn has_visual(self) -> bool {
        matches!(
            self,
            CoachingDeliveryMode::Visual | CoachingDeliveryMode::Both
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            CoachingDeliveryMode::None => "none",
            CoachingDeliveryMode::Visual => "visual",
            CoachingDeliveryMode::Voice => "voice",
            CoachingDeliveryMode::Both => "both",
        }
    }
}

/// Configuration for voice coaching announcements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceCoachingConfig {
    /// The delivery mode (none, visual, voice, both).
    pub mode: CoachingDeliveryMode,
    /// Whether to announce distance milestones.
    pub announce_distance: bool,
    /// Whether to announce time milestones.
    pub announce_time: bool,
    /// Whether to announce pace/speed.
    pub announce_pace: bool,
    /// Whether to announce split completions.
    pub announce_splits: bool,
    /// Whether to announce goal progress.
    pub announce_goal_progress: bool,
    /// The interval (in meters) between distance announcements.
    pub distance_interval_m: u32,
    /// The interval (in seconds) between time announcements.
    pub time_interval_s: u32,
}

impl Default for VoiceCoachingConfig {
    fn default() -> Self {
        Self {
            mode: CoachingDeliveryMode::None,
            announce_distance: true,
            announce_time: true,
            announce_pace: false,
            announce_splits: true,
            announce_goal_progress: true,
            distance_interval_m: 1000, // every 1 km
            time_interval_s: 300,      // every 5 min
        }
    }
}

/// Decides whether a voice coaching announcement should be made now,
/// given the coaching config, quiet hours, and the workout context.
///
/// `last_announcement_distance_m` and `last_announcement_time_s` are the
/// distance/time since the last announcement (or 0 if none yet).
/// `local_minute` is the current local-time minute for quiet hours.
pub fn should_announce(
    config: &VoiceCoachingConfig,
    quiet: QuietHoursConfig,
    kind: CoachingAnnouncementKind,
    last_announcement_distance_m: f64,
    last_announcement_time_s: u64,
    local_minute: u32,
) -> bool {
    // If voice coaching is disabled entirely, no announcements.
    if !config.mode.has_voice() {
        return false;
    }

    // Don't announce during quiet hours (no voice at 3 AM).
    if quiet.enabled && quiet.is_quiet_at(local_minute) {
        return false;
    }

    match kind {
        CoachingAnnouncementKind::Distance => {
            config.announce_distance
                && last_announcement_distance_m >= config.distance_interval_m as f64
        }
        CoachingAnnouncementKind::Time => {
            config.announce_time
                && last_announcement_time_s >= config.time_interval_s as u64
        }
        CoachingAnnouncementKind::Pace => config.announce_pace,
        CoachingAnnouncementKind::Split => config.announce_splits,
        CoachingAnnouncementKind::GoalProgress => config.announce_goal_progress,
        CoachingAnnouncementKind::Encouragement => true, // always allowed if voice is on
    }
}

/// The kind of voice coaching announcement being evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachingAnnouncementKind {
    /// Distance milestone (e.g., "You've reached 1 km").
    Distance,
    /// Time milestone (e.g., "5 minutes completed").
    Time,
    /// Pace/speed update (e.g., "Current pace: 6 min per km").
    Pace,
    /// Split completion (e.g., "Mile 2 complete").
    Split,
    /// Goal progress (e.g., "80% of your 5K goal").
    GoalProgress,
    /// Encouragement message (e.g., "Keep going, you're doing great!").
    Encouragement,
}

// ──────────────────────────────────────────────────────────────────────
// Notification decision (the main entry point)
// ───────────────────────────────────────────────────────────────--------

/// The overall decision for a notification request: should it be
/// delivered, and if so, how and when?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationDecision {
    /// The notification type.
    pub notif_type: NotificationType,
    /// Whether the notification should be delivered at all.
    pub should_deliver: bool,
    /// Whether it should be delivered now (vs. deferred).
    pub deliver_now: bool,
    /// The content to display (if delivering).
    pub content: Option<NotificationContent>,
    /// If deferred, the local-time minute to reschedule to.
    pub reschedule_to_minute: Option<u32>,
    /// The reason for the decision (for logging / debugging).
    pub reason: String,
    /// The priority.
    pub priority: NotificationPriority,
}

/// The full context for making a notification decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationDecisionContext {
    /// The notification type to evaluate.
    pub notif_type: NotificationType,
    /// The user's notification preferences (per-category toggles).
    pub preferences: NotificationPreferences,
    /// Whether the user has granted notification permission.
    pub permission: NotificationPermission,
    /// Quiet hours configuration.
    pub quiet_hours: QuietHoursConfig,
    /// The current UTC time (epoch milliseconds).
    pub now_utc_ms: i64,
    /// The user's timezone context.
    pub timezone: TimezoneContext,
    /// Optional context params for building content.
    pub context_params: NotificationContextParams,
}

/// Makes the complete notification decision: should it be delivered,
/// when, and with what content.
///
/// This is the main entry point for the notification decision logic.
/// It combines:
/// 1. Permission check — if not granted, no notification.
/// 2. Category preference — if the category is disabled, no notification.
/// 3. Quiet hours — if in quiet hours and not critical, defer.
/// 4. Content building — construct the title/body/action.
pub fn decide_notification(ctx: &NotificationDecisionContext) -> NotificationDecision {
    let notif_type = ctx.notif_type;
    let priority = notif_type.default_priority();

    // 1. Permission check.
    if !ctx.permission.is_granted() {
        return NotificationDecision {
            notif_type,
            should_deliver: false,
            deliver_now: false,
            content: None,
            reschedule_to_minute: None,
            reason: format!(
                "Notification permission is {} — not delivering.",
                ctx.permission.label()
            ),
            priority,
        };
    }

    // 2. Category preference check.
    if !ctx.preferences.is_type_enabled(notif_type) {
        return NotificationDecision {
            notif_type,
            should_deliver: false,
            deliver_now: false,
            content: None,
            reschedule_to_minute: None,
            reason: format!(
                "Category '{}' is disabled by the user.",
                notif_type.category().label()
            ),
            priority,
        };
    }

    // 3. Quiet hours evaluation.
    let local_minute = ctx.timezone.local_minute_of_day(ctx.now_utc_ms);
    let quiet_decision = evaluate_quiet_hours(
        ctx.quiet_hours,
        notif_type,
        local_minute,
        ctx.preferences.critical_bypasses_quiet_hours,
    );

    if !quiet_decision.should_deliver_now {
        // Deferred — build content but mark as not-now.
        let content = build_notification_content(notif_type, &ctx.context_params);
        return NotificationDecision {
            notif_type,
            should_deliver: true,
            deliver_now: false,
            content: Some(content),
            reschedule_to_minute: quiet_decision.reschedule_to_minute,
            reason: quiet_decision.reason,
            priority,
        };
    }

    // 4. Build and deliver.
    let content = build_notification_content(notif_type, &ctx.context_params);
    NotificationDecision {
        notif_type,
        should_deliver: true,
        deliver_now: true,
        content: Some(content),
        reschedule_to_minute: None,
        reason: quiet_decision.reason,
        priority,
    }
}

// ──────────────────────────────────────────────────────────────────────
// Notification status (for UI / diagnostics)
// ──────────────────────────────────────────────────────────────────────

/// A full status snapshot of the notification system for the UI or
/// diagnostics screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationStatus {
    /// Whether notification permission is granted.
    pub permission_granted: bool,
    /// The permission state.
    pub permission: NotificationPermission,
    /// Number of categories enabled.
    pub enabled_category_count: usize,
    /// Whether quiet hours are enabled.
    pub quiet_hours_enabled: bool,
    /// The quiet hours window (local time, "HH:MM–HH:MM").
    pub quiet_hours_window: String,
    /// Whether voice coaching is enabled.
    pub voice_coaching_enabled: bool,
    /// The coaching delivery mode.
    pub coaching_mode: CoachingDeliveryMode,
    /// The timezone label.
    pub timezone_label: String,
}

/// Builds a full notification status snapshot.
pub fn build_notification_status(
    permission: NotificationPermission,
    preferences: NotificationPreferences,
    quiet_hours: QuietHoursConfig,
    voice_config: &VoiceCoachingConfig,
    timezone: TimezoneContext,
) -> NotificationStatus {
    let quiet_window = if quiet_hours.enabled {
        format!(
            "{:02}:{:02}–{:02}:{:02}",
            quiet_hours.start_minute / 60,
            quiet_hours.start_minute % 60,
            quiet_hours.end_minute / 60,
            quiet_hours.end_minute % 60
        )
    } else {
        "Disabled".to_string()
    };

    NotificationStatus {
        permission_granted: permission.is_granted(),
        permission,
        enabled_category_count: preferences.enabled_count(),
        quiet_hours_enabled: quiet_hours.enabled,
        quiet_hours_window: quiet_window,
        voice_coaching_enabled: voice_config.mode.has_voice(),
        coaching_mode: voice_config.mode,
        timezone_label: timezone.label(),
    }
}

// ──────────────────────────────────────────────────────────────────────
// Scheduled reminder helpers
// ──────────────────────────────────────────────────────────────────────

/// Computes the UTC timestamp (epoch milliseconds) for the next
/// occurrence of a daily local-time reminder.
///
/// Given the current UTC time, the local-time minute of the reminder,
/// and the timezone context, returns the UTC ms timestamp for the next
/// occurrence. If the target time has already passed today, it returns
/// tomorrow's occurrence.
pub fn next_daily_reminder_utc_ms(
    now_utc_ms: i64,
    local_minute: u32,
    timezone: TimezoneContext,
) -> i64 {
    let day_ms = 86_400_000i64;
    let today_local_minute = timezone.local_minute_of_day(now_utc_ms);
    let local_minute_target = local_minute as i64;

    // How many minutes from now (in local time) is the target?
    let minutes_diff = local_minute_target - today_local_minute as i64;
    let target_utc_ms = if minutes_diff > 0 {
        // Target is later today.
        now_utc_ms + minutes_diff * 60_000
    } else {
        // Target has passed today; schedule for tomorrow.
        now_utc_ms + (minutes_diff + 1440) * 60_000
    };
    // Snap to the correct local minute (handle edge cases).
    let snapped = timezone.local_minute_of_day(target_utc_ms) as i64;
    if snapped == local_minute_target {
        target_utc_ms
    } else {
        // Adjust by the difference.
        target_utc_ms + (local_minute_target - snapped) * 60_000
    }
}

/// Computes the UTC timestamp for the next occurrence of a weekly
/// reminder at a specific day-of-week and local-time minute.
///
/// `day_of_week` is 0=Sunday, 1=Monday, ..., 6=Saturday.
pub fn next_weekly_reminder_utc_ms(
    now_utc_ms: i64,
    day_of_week: u32,
    local_minute: u32,
    timezone: TimezoneContext,
) -> i64 {
    let day_ms = 86_400_000i64;
    let local_ms = now_utc_ms + (timezone.utc_offset_seconds as i64 * 1000);
    let days_since_epoch = local_ms.rem_euclid(day_ms) / day_ms;
    let current_dow = ((days_since_epoch + 4) % 7) as u32; // Jan 1 1970 was Thursday (4)

    let days_ahead = if day_of_week >= current_dow {
        day_of_week - current_dow
    } else {
        7 - current_dow + day_of_week
    };

    let today_local_minute = timezone.local_minute_of_day(now_utc_ms);
    let days_ahead = if days_ahead == 0 && local_minute <= today_local_minute {
        7 // same day but time passed — next week
    } else {
        days_ahead
    };

    now_utc_ms + (days_ahead as i64 * day_ms) + (local_minute as i64 - today_local_minute as i64) * 60_000
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── NotificationCategory ──

    #[test]
    fn category_labels_are_human_readable() {
        assert_eq!(NotificationCategory::Reminders.label(), "Workout reminders");
        assert_eq!(NotificationCategory::Achievements.label(), "Achievements");
        assert_eq!(NotificationCategory::Coaching.label(), "Coaching");
        assert_eq!(NotificationCategory::System.label(), "System alerts");
        assert_eq!(NotificationCategory::Recovery.label(), "Recovery alerts");
    }

    #[test]
    fn category_descriptions_are_nonempty() {
        for cat in [
            NotificationCategory::Reminders,
            NotificationCategory::Achievements,
            NotificationCategory::Coaching,
            NotificationCategory::System,
            NotificationCategory::Recovery,
        ] {
            assert!(!cat.description().is_empty());
        }
    }

    // ── NotificationType → category mapping ──

    #[test]
    fn scheduled_workout_maps_to_reminders() {
        assert_eq!(
            NotificationType::ScheduledWorkout.category(),
            NotificationCategory::Reminders
        );
    }

    #[test]
    fn plan_reminder_maps_to_reminders() {
        assert_eq!(
            NotificationType::PlanReminder.category(),
            NotificationCategory::Reminders
        );
    }

    #[test]
    fn missed_workout_maps_to_reminders() {
        assert_eq!(
            NotificationType::MissedWorkout.category(),
            NotificationCategory::Reminders
        );
    }

    #[test]
    fn goal_milestone_maps_to_achievements() {
        assert_eq!(
            NotificationType::GoalMilestone.category(),
            NotificationCategory::Achievements
        );
    }

    #[test]
    fn streak_warning_maps_to_achievements() {
        assert_eq!(
            NotificationType::StreakWarning.category(),
            NotificationCategory::Achievements
        );
    }

    #[test]
    fn badge_earned_maps_to_achievements() {
        assert_eq!(
            NotificationType::BadgeEarned.category(),
            NotificationCategory::Achievements
        );
    }

    #[test]
    fn plan_updated_maps_to_coaching() {
        assert_eq!(
            NotificationType::PlanUpdated.category(),
            NotificationCategory::Coaching
        );
    }

    #[test]
    fn voice_coaching_maps_to_coaching() {
        assert_eq!(
            NotificationType::VoiceCoaching.category(),
            NotificationCategory::Coaching
        );
    }

    #[test]
    fn device_disconnected_maps_to_system() {
        assert_eq!(
            NotificationType::DeviceDisconnected.category(),
            NotificationCategory::System
        );
    }

    #[test]
    fn sync_failed_maps_to_system() {
        assert_eq!(
            NotificationType::SyncFailed.category(),
            NotificationCategory::System
        );
    }

    #[test]
    fn workout_recovered_maps_to_recovery() {
        assert_eq!(
            NotificationType::WorkoutRecovered.category(),
            NotificationCategory::Recovery
        );
    }

    // ── NotificationType → priority ──

    #[test]
    fn workout_recovered_is_high_priority() {
        assert_eq!(
            NotificationType::WorkoutRecovered.default_priority(),
            NotificationPriority::High
        );
    }

    #[test]
    fn device_disconnected_is_high_priority() {
        assert_eq!(
            NotificationType::DeviceDisconnected.default_priority(),
            NotificationPriority::High
        );
    }

    #[test]
    fn voice_coaching_is_low_priority() {
        assert_eq!(
            NotificationType::VoiceCoaching.default_priority(),
            NotificationPriority::Low
        );
    }

    #[test]
    fn most_types_are_default_priority() {
        assert_eq!(
            NotificationType::ScheduledWorkout.default_priority(),
            NotificationPriority::Default
        );
        assert_eq!(
            NotificationType::PlanReminder.default_priority(),
            NotificationPriority::Default
        );
        assert_eq!(
            NotificationType::MissedWorkout.default_priority(),
            NotificationPriority::Default
        );
        assert_eq!(
            NotificationType::GoalMilestone.default_priority(),
            NotificationPriority::Default
        );
        assert_eq!(
            NotificationType::StreakWarning.default_priority(),
            NotificationPriority::Default
        );
        assert_eq!(
            NotificationType::BadgeEarned.default_priority(),
            NotificationPriority::Default
        );
        assert_eq!(
            NotificationType::PlanUpdated.default_priority(),
            NotificationPriority::Default
        );
        assert_eq!(
            NotificationType::SyncFailed.default_priority(),
            NotificationPriority::Default
        );
    }

    // ── NotificationType::is_critical ──

    #[test]
    fn workout_recovered_is_critical() {
        assert!(NotificationType::WorkoutRecovered.is_critical());
    }

    #[test]
    fn device_disconnected_is_critical() {
        assert!(NotificationType::DeviceDisconnected.is_critical());
    }

    #[test]
    fn sync_failed_is_critical() {
        assert!(NotificationType::SyncFailed.is_critical());
    }

    #[test]
    fn scheduled_workout_is_not_critical() {
        assert!(!NotificationType::ScheduledWorkout.is_critical());
    }

    #[test]
    fn voice_coaching_is_not_critical() {
        assert!(!NotificationType::VoiceCoaching.is_critical());
    }

    // ── NotificationPermission ──

    #[test]
    fn permission_granted_is_granted() {
        assert!(NotificationPermission::Granted.is_granted());
    }

    #[test]
    fn permission_not_requested_is_not_granted() {
        assert!(!NotificationPermission::NotRequested.is_granted());
    }

    #[test]
    fn permission_denied_is_not_granted() {
        assert!(!NotificationPermission::Denied.is_granted());
    }

    #[test]
    fn permission_permanently_denied_is_not_granted() {
        assert!(!NotificationPermission::PermanentlyDenied.is_granted());
    }

    #[test]
    fn should_request_when_not_requested() {
        assert!(NotificationPermission::NotRequested.should_request());
    }

    #[test]
    fn should_request_when_denied() {
        assert!(NotificationPermission::Denied.should_request());
    }

    #[test]
    fn should_not_request_when_granted() {
        assert!(!NotificationPermission::Granted.should_request());
    }

    #[test]
    fn should_not_request_when_permanently_denied() {
        assert!(!NotificationPermission::PermanentlyDenied.should_request());
    }

    // ── NotificationPreferences ──

    #[test]
    fn default_preferences_all_disabled() {
        let prefs = NotificationPreferences::default();
        assert!(!prefs.reminders_enabled);
        assert!(!prefs.achievements_enabled);
        assert!(!prefs.coaching_enabled);
        assert!(!prefs.system_enabled);
        assert!(!prefs.recovery_enabled);
    }

    #[test]
    fn default_preferences_critical_bypass_on() {
        let prefs = NotificationPreferences::default();
        assert!(prefs.critical_bypasses_quiet_hours);
    }

    #[test]
    fn is_category_enabled_reflects_flags() {
        let prefs = NotificationPreferences {
            reminders_enabled: true,
            achievements_enabled: false,
            coaching_enabled: true,
            system_enabled: false,
            recovery_enabled: true,
            critical_bypasses_quiet_hours: true,
        };
        assert!(prefs.is_category_enabled(NotificationCategory::Reminders));
        assert!(!prefs.is_category_enabled(NotificationCategory::Achievements));
        assert!(prefs.is_category_enabled(NotificationCategory::Coaching));
        assert!(!prefs.is_category_enabled(NotificationCategory::System));
        assert!(prefs.is_category_enabled(NotificationCategory::Recovery));
    }

    #[test]
    fn with_category_enabled_updates_correct_flag() {
        let prefs = NotificationPreferences::default();
        let updated = prefs.with_category_enabled(NotificationCategory::Reminders, true);
        assert!(updated.reminders_enabled);
        assert!(!updated.achievements_enabled);
    }

    #[test]
    fn is_type_enabled_uses_category() {
        let prefs = NotificationPreferences {
            reminders_enabled: true,
            ..Default::default()
        };
        assert!(prefs.is_type_enabled(NotificationType::ScheduledWorkout));
        assert!(!prefs.is_type_enabled(NotificationType::BadgeEarned));
    }

    #[test]
    fn enabled_count_counts_correctly() {
        let prefs = NotificationPreferences {
            reminders_enabled: true,
            achievements_enabled: true,
            coaching_enabled: false,
            system_enabled: false,
            recovery_enabled: false,
            critical_bypasses_quiet_hours: true,
        };
        assert_eq!(prefs.enabled_count(), 2);
    }

    #[test]
    fn all_disabled_when_nothing_enabled() {
        let prefs = NotificationPreferences::default();
        assert!(prefs.all_disabled());
    }

    #[test]
    fn all_disabled_false_when_something_enabled() {
        let prefs = NotificationPreferences {
            reminders_enabled: true,
            ..Default::default()
        };
        assert!(!prefs.all_disabled());
    }

    // ── TimezoneContext ──

    #[test]
    fn local_minute_of_day_utc_zero() {
        let tz = TimezoneContext {
            utc_offset_seconds: 0,
            is_dst: false,
        };
        // UTC midnight = local midnight = minute 0.
        assert_eq!(tz.local_minute_of_day(0), 0);
        // 1 hour into the day = minute 60.
        assert_eq!(tz.local_minute_of_day(3_600_000), 60);
        // 12 hours = minute 720.
        assert_eq!(tz.local_minute_of_day(43_200_000), 720);
    }

    #[test]
    fn local_minute_of_day_positive_offset() {
        let tz = TimezoneContext {
            utc_offset_seconds: 3600, // UTC+1
            is_dst: false,
        };
        // UTC midnight = 01:00 local = minute 60.
        assert_eq!(tz.local_minute_of_day(0), 60);
    }

    #[test]
    fn local_minute_of_day_negative_offset() {
        let tz = TimezoneContext {
            utc_offset_seconds: -18_000, // UTC-5
            is_dst: false,
        };
        // UTC midnight = 19:00 local (previous day) = minute 1140.
        assert_eq!(tz.local_minute_of_day(0), 1140);
    }

    #[test]
    fn local_hour_works() {
        let tz = TimezoneContext {
            utc_offset_seconds: 3600,
            is_dst: false,
        };
        assert_eq!(tz.local_hour(0), 1); // 01:00 local
    }

    #[test]
    fn timezone_label_utc() {
        let tz = TimezoneContext {
            utc_offset_seconds: 0,
            is_dst: false,
        };
        assert_eq!(tz.label(), "UTC");
    }

    #[test]
    fn timezone_label_positive() {
        let tz = TimezoneContext {
            utc_offset_seconds: 3600,
            is_dst: false,
        };
        assert_eq!(tz.label(), "UTC+1");
    }

    #[test]
    fn timezone_label_negative() {
        let tz = TimezoneContext {
            utc_offset_seconds: -18_000,
            is_dst: false,
        };
        assert_eq!(tz.label(), "UTC-5");
    }

    #[test]
    fn local_minute_wraps_past_midnight() {
        let tz = TimezoneContext {
            utc_offset_seconds: -36_000, // UTC-10
            is_dst: false,
        };
        // UTC 12:00 = local 02:00 = minute 120
        assert_eq!(tz.local_minute_of_day(43_200_000), 120);
    }

    // ── QuietHoursConfig ──

    #[test]
    fn quiet_hours_disabled_never_quiet() {
        let q = QuietHoursConfig {
            enabled: false,
            start_minute: 1320,
            end_minute: 420,
        };
        assert!(!q.is_quiet_at(0));
        assert!(!q.is_quiet_at(1320));
        assert!(!q.is_quiet_at(420));
    }

    #[test]
    fn quiet_hours_wrap_past_midnight() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320, // 22:00
            end_minute: 420,     // 07:00
        };
        // 22:00 is quiet.
        assert!(q.is_quiet_at(1320));
        // 03:00 is quiet.
        assert!(q.is_quiet_at(180));
        // 06:59 is quiet.
        assert!(q.is_quiet_at(419));
        // 07:00 is not quiet (end is exclusive).
        assert!(!q.is_quiet_at(420));
        // 21:59 is not quiet.
        assert!(!q.is_quiet_at(1319));
        // 12:00 is not quiet.
        assert!(!q.is_quiet_at(720));
    }

    #[test]
    fn quiet_hours_simple_range_no_midnight_wrap() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 600, // 10:00
            end_minute: 900,   // 15:00
        };
        assert!(q.is_quiet_at(600));
        assert!(q.is_quiet_at(750));
        assert!(q.is_quiet_at(899));
        assert!(!q.is_quiet_at(900));
        assert!(!q.is_quiet_at(599));
        assert!(!q.is_quiet_at(0));
    }

    #[test]
    fn resume_minute_when_not_quiet_returns_current() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        assert_eq!(q.resume_minute(720), 720); // noon, not quiet
    }

    #[test]
    fn resume_minute_when_quiet_returns_end() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        assert_eq!(q.resume_minute(1320), 420); // 22:00, resumes at 07:00
    }

    #[test]
    fn resume_minute_when_disabled_returns_current() {
        let q = QuietHoursConfig {
            enabled: false,
            start_minute: 1320,
            end_minute: 420,
        };
        assert_eq!(q.resume_minute(1320), 1320);
    }

    // ── evaluate_quiet_hours ──

    #[test]
    fn evaluate_quiet_hours_disabled_delivers_now() {
        let q = QuietHoursConfig {
            enabled: false,
            ..Default::default()
        };
        let decision = evaluate_quiet_hours(q, NotificationType::ScheduledWorkout, 1320, true);
        assert!(!decision.is_quiet);
        assert!(decision.should_deliver_now);
        assert!(decision.reschedule_to_minute.is_none());
    }

    #[test]
    fn evaluate_quiet_hours_outside_window_delivers_now() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        let decision = evaluate_quiet_hours(q, NotificationType::ScheduledWorkout, 720, true);
        assert!(!decision.is_quiet);
        assert!(decision.should_deliver_now);
    }

    #[test]
    fn evaluate_quiet_hours_inside_window_defers() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        let decision = evaluate_quiet_hours(q, NotificationType::ScheduledWorkout, 180, true);
        assert!(decision.is_quiet);
        assert!(!decision.should_deliver_now);
        assert_eq!(decision.reschedule_to_minute, Some(420));
    }

    #[test]
    fn evaluate_quiet_hours_critical_bypasses() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        let decision = evaluate_quiet_hours(
            q,
            NotificationType::WorkoutRecovered,
            180, // 03:00, inside quiet hours
            true, // critical bypass enabled
        );
        assert!(decision.is_quiet);
        assert!(decision.should_deliver_now); // bypassed
    }

    #[test]
    fn evaluate_quiet_hours_critical_no_bypass_defers() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        let decision = evaluate_quiet_hours(
            q,
            NotificationType::WorkoutRecovered,
            180,  // 03:00
            false, // critical bypass disabled
        );
        assert!(decision.is_quiet);
        assert!(!decision.should_deliver_now); // not bypassed
    }

    #[test]
    fn evaluate_quiet_hours_non_critical_inside_window_defers() {
        let q = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        let decision = evaluate_quiet_hours(
            q,
            NotificationType::ScheduledWorkout,
            180,
            true, // bypass enabled, but this type is not critical
        );
        assert!(decision.is_quiet);
        assert!(!decision.should_deliver_now);
    }

    // ── build_notification_content ──

    #[test]
    fn scheduled_workout_content_has_title_and_body() {
        let content = build_notification_content(
            NotificationType::ScheduledWorkout,
            &NotificationContextParams::default(),
        );
        assert!(!content.title.is_empty());
        assert!(!content.body.is_empty());
        assert_eq!(content.category, NotificationCategory::Reminders);
        assert!(content.action_label.is_some());
    }

    #[test]
    fn scheduled_workout_content_with_time_includes_it() {
        let params = NotificationContextParams {
            workout_time: Some("6:00 PM".to_string()),
            ..Default::default()
        };
        let content = build_notification_content(NotificationType::ScheduledWorkout, &params);
        assert!(content.body.contains("6:00 PM"));
    }

    #[test]
    fn streak_warning_content_includes_streak_days() {
        let params = NotificationContextParams {
            streak_days: Some(7),
            ..Default::default()
        };
        let content = build_notification_content(NotificationType::StreakWarning, &params);
        assert!(content.body.contains("7"));
    }

    #[test]
    fn device_disconnected_content_includes_device_name() {
        let params = NotificationContextParams {
            device_name: Some("Galaxy Watch Ultra".to_string()),
            ..Default::default()
        };
        let content = build_notification_content(NotificationType::DeviceDisconnected, &params);
        assert!(content.body.contains("Galaxy Watch Ultra"));
    }

    #[test]
    fn badge_earned_content_includes_badge_name() {
        let params = NotificationContextParams {
            badge_name: Some("First 5K".to_string()),
            ..Default::default()
        };
        let content = build_notification_content(NotificationType::BadgeEarned, &params);
        assert!(content.body.contains("First 5K"));
    }

    #[test]
    fn voice_coaching_content_uses_coaching_message() {
        let params = NotificationContextParams {
            coaching_message: Some("You've reached 1 kilometer!".to_string()),
            ..Default::default()
        };
        let content = build_notification_content(NotificationType::VoiceCoaching, &params);
        assert_eq!(content.body, "You've reached 1 kilometer!");
    }

    #[test]
    fn voice_coaching_content_falls_back_when_no_message() {
        let content = build_notification_content(
            NotificationType::VoiceCoaching,
            &NotificationContextParams::default(),
        );
        assert_eq!(content.body, "Keep going!");
    }

    #[test]
    fn all_types_have_nonempty_content() {
        let types = [
            NotificationType::ScheduledWorkout,
            NotificationType::PlanReminder,
            NotificationType::MissedWorkout,
            NotificationType::GoalMilestone,
            NotificationType::StreakWarning,
            NotificationType::DeviceDisconnected,
            NotificationType::SyncFailed,
            NotificationType::WorkoutRecovered,
            NotificationType::BadgeEarned,
            NotificationType::PlanUpdated,
            NotificationType::VoiceCoaching,
        ];
        for t in types {
            let content = build_notification_content(t, &NotificationContextParams::default());
            assert!(!content.title.is_empty(), "title empty for {:?}", t);
            assert!(!content.body.is_empty(), "body empty for {:?}", t);
            assert_eq!(content.notif_type, t);
            assert_eq!(content.category, t.category());
            assert_eq!(content.priority, t.default_priority());
        }
    }

    // ── CoachingDeliveryMode ──

    #[test]
    fn coaching_mode_none_has_no_voice_no_visual() {
        let m = CoachingDeliveryMode::None;
        assert!(!m.has_voice());
        assert!(!m.has_visual());
    }

    #[test]
    fn coaching_mode_visual_has_visual_no_voice() {
        let m = CoachingDeliveryMode::Visual;
        assert!(m.has_visual());
        assert!(!m.has_voice());
    }

    #[test]
    fn coaching_mode_voice_has_voice_no_visual() {
        let m = CoachingDeliveryMode::Voice;
        assert!(m.has_voice());
        assert!(!m.has_visual());
    }

    #[test]
    fn coaching_mode_both_has_both() {
        let m = CoachingDeliveryMode::Both;
        assert!(m.has_voice());
        assert!(m.has_visual());
    }

    // ── should_announce ──

    #[test]
    fn should_announce_false_when_voice_disabled() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::None,
            ..Default::default()
        };
        assert!(!should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Distance,
            1000.0,
            0,
            720
        ));
    }

    #[test]
    fn should_announce_distance_when_interval_reached() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Voice,
            announce_distance: true,
            distance_interval_m: 1000,
            ..Default::default()
        };
        assert!(should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Distance,
            1000.0,
            0,
            720
        ));
    }

    #[test]
    fn should_announce_distance_false_when_interval_not_reached() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Voice,
            announce_distance: true,
            distance_interval_m: 1000,
            ..Default::default()
        };
        assert!(!should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Distance,
            500.0,
            0,
            720
        ));
    }

    #[test]
    fn should_announce_false_during_quiet_hours() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Voice,
            announce_distance: true,
            ..Default::default()
        };
        let quiet = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        assert!(!should_announce(
            &config,
            quiet,
            CoachingAnnouncementKind::Distance,
            1000.0,
            0,
            180 // 03:00, inside quiet hours
        ));
    }

    #[test]
    fn should_announce_time_when_interval_reached() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Both,
            announce_time: true,
            time_interval_s: 300,
            ..Default::default()
        };
        assert!(should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Time,
            0.0,
            300,
            720
        ));
    }

    #[test]
    fn should_announce_time_false_when_disabled() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Voice,
            announce_time: false,
            ..Default::default()
        };
        assert!(!should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Time,
            0.0,
            600,
            720
        ));
    }

    #[test]
    fn should_announce_encouragement_when_voice_on() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Voice,
            ..Default::default()
        };
        assert!(should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Encouragement,
            0.0,
            0,
            720
        ));
    }

    #[test]
    fn should_announce_pace_false_when_disabled() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Voice,
            announce_pace: false,
            ..Default::default()
        };
        assert!(!should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Pace,
            0.0,
            0,
            720
        ));
    }

    #[test]
    fn should_announce_pace_true_when_enabled() {
        let config = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Voice,
            announce_pace: true,
            ..Default::default()
        };
        assert!(should_announce(
            &config,
            QuietHoursConfig::default(),
            CoachingAnnouncementKind::Pace,
            0.0,
            0,
            720
        ));
    }

    // ── decide_notification ──

    #[test]
    fn decide_notification_not_delivered_when_permission_denied() {
        let ctx = NotificationDecisionContext {
            notif_type: NotificationType::ScheduledWorkout,
            preferences: NotificationPreferences {
                reminders_enabled: true,
                ..Default::default()
            },
            permission: NotificationPermission::Denied,
            quiet_hours: QuietHoursConfig::default(),
            now_utc_ms: 0,
            timezone: TimezoneContext::default(),
            context_params: NotificationContextParams::default(),
        };
        let decision = decide_notification(&ctx);
        assert!(!decision.should_deliver);
        assert!(!decision.deliver_now);
        assert!(decision.content.is_none());
    }

    #[test]
    fn decide_notification_not_delivered_when_category_disabled() {
        let ctx = NotificationDecisionContext {
            notif_type: NotificationType::ScheduledWorkout,
            preferences: NotificationPreferences::default(), // all disabled
            permission: NotificationPermission::Granted,
            quiet_hours: QuietHoursConfig::default(),
            now_utc_ms: 0,
            timezone: TimezoneContext::default(),
            context_params: NotificationContextParams::default(),
        };
        let decision = decide_notification(&ctx);
        assert!(!decision.should_deliver);
        assert!(!decision.deliver_now);
    }

    #[test]
    fn decide_notification_delivered_when_all_conditions_met() {
        let ctx = NotificationDecisionContext {
            notif_type: NotificationType::ScheduledWorkout,
            preferences: NotificationPreferences {
                reminders_enabled: true,
                ..Default::default()
            },
            permission: NotificationPermission::Granted,
            quiet_hours: QuietHoursConfig::default(), // disabled
            now_utc_ms: 0,
            timezone: TimezoneContext::default(),
            context_params: NotificationContextParams::default(),
        };
        let decision = decide_notification(&ctx);
        assert!(decision.should_deliver);
        assert!(decision.deliver_now);
        assert!(decision.content.is_some());
    }

    #[test]
    fn decide_notification_deferred_during_quiet_hours() {
        let ctx = NotificationDecisionContext {
            notif_type: NotificationType::ScheduledWorkout,
            preferences: NotificationPreferences {
                reminders_enabled: true,
                ..Default::default()
            },
            permission: NotificationPermission::Granted,
            quiet_hours: QuietHoursConfig {
                enabled: true,
                start_minute: 1320,
                end_minute: 420,
            },
            now_utc_ms: 0, // UTC midnight; with UTC+0 that's local midnight (minute 0, inside quiet 22:00-07:00)
            timezone: TimezoneContext::default(),
            context_params: NotificationContextParams::default(),
        };
        let decision = decide_notification(&ctx);
        assert!(decision.should_deliver);
        assert!(!decision.deliver_now); // deferred
        assert!(decision.reschedule_to_minute.is_some());
    }

    #[test]
    fn decide_notification_critical_delivered_during_quiet_hours_with_bypass() {
        let ctx = NotificationDecisionContext {
            notif_type: NotificationType::WorkoutRecovered,
            preferences: NotificationPreferences {
                recovery_enabled: true,
                critical_bypasses_quiet_hours: true,
                ..Default::default()
            },
            permission: NotificationPermission::Granted,
            quiet_hours: QuietHoursConfig {
                enabled: true,
                start_minute: 1320,
                end_minute: 420,
            },
            now_utc_ms: 0, // local midnight, inside quiet hours
            timezone: TimezoneContext::default(),
            context_params: NotificationContextParams::default(),
        };
        let decision = decide_notification(&ctx);
        assert!(decision.should_deliver);
        assert!(decision.deliver_now); // bypassed quiet hours
    }

    #[test]
    fn decide_notification_priority_matches_type_default() {
        let ctx = NotificationDecisionContext {
            notif_type: NotificationType::WorkoutRecovered,
            preferences: NotificationPreferences {
                recovery_enabled: true,
                ..Default::default()
            },
            permission: NotificationPermission::Granted,
            quiet_hours: QuietHoursConfig::default(),
            now_utc_ms: 0,
            timezone: TimezoneContext::default(),
            context_params: NotificationContextParams::default(),
        };
        let decision = decide_notification(&ctx);
        assert_eq!(decision.priority, NotificationPriority::High);
    }

    // ── build_notification_status ──

    #[test]
    fn build_status_reflects_permission() {
        let status = build_notification_status(
            NotificationPermission::Granted,
            NotificationPreferences::default(),
            QuietHoursConfig::default(),
            &VoiceCoachingConfig::default(),
            TimezoneContext::default(),
        );
        assert!(status.permission_granted);
    }

    #[test]
    fn build_status_reflects_enabled_count() {
        let prefs = NotificationPreferences {
            reminders_enabled: true,
            achievements_enabled: true,
            ..Default::default()
        };
        let status = build_notification_status(
            NotificationPermission::Granted,
            prefs,
            QuietHoursConfig::default(),
            &VoiceCoachingConfig::default(),
            TimezoneContext::default(),
        );
        assert_eq!(status.enabled_category_count, 2);
    }

    #[test]
    fn build_status_reflects_quiet_hours() {
        let quiet = QuietHoursConfig {
            enabled: true,
            start_minute: 1320,
            end_minute: 420,
        };
        let status = build_notification_status(
            NotificationPermission::Granted,
            NotificationPreferences::default(),
            quiet,
            &VoiceCoachingConfig::default(),
            TimezoneContext::default(),
        );
        assert!(status.quiet_hours_enabled);
        assert_eq!(status.quiet_hours_window, "22:00–07:00");
    }

    #[test]
    fn build_status_quiet_hours_disabled_shows_disabled() {
        let status = build_notification_status(
            NotificationPermission::Granted,
            NotificationPreferences::default(),
            QuietHoursConfig::default(), // disabled
            &VoiceCoachingConfig::default(),
            TimezoneContext::default(),
        );
        assert!(!status.quiet_hours_enabled);
        assert_eq!(status.quiet_hours_window, "Disabled");
    }

    #[test]
    fn build_status_reflects_voice_coaching() {
        let vc = VoiceCoachingConfig {
            mode: CoachingDeliveryMode::Both,
            ..Default::default()
        };
        let status = build_notification_status(
            NotificationPermission::Granted,
            NotificationPreferences::default(),
            QuietHoursConfig::default(),
            &vc,
            TimezoneContext::default(),
        );
        assert!(status.voice_coaching_enabled);
        assert_eq!(status.coaching_mode, CoachingDeliveryMode::Both);
    }

    #[test]
    fn build_status_reflects_timezone() {
        let tz = TimezoneContext {
            utc_offset_seconds: -18_000,
            is_dst: false,
        };
        let status = build_notification_status(
            NotificationPermission::Granted,
            NotificationPreferences::default(),
            QuietHoursConfig::default(),
            &VoiceCoachingConfig::default(),
            tz,
        );
        assert_eq!(status.timezone_label, "UTC-5");
    }

    // ── next_daily_reminder_utc_ms ──

    #[test]
    fn next_daily_reminder_same_day_if_future() {
        let tz = TimezoneContext {
            utc_offset_seconds: 0,
            is_dst: false,
        };
        // UTC 10:00 (600 min). Target is 14:00 (840 min). Should be 4 hours later.
        let now = 10 * 3_600_000i64; // 10:00 UTC
        let result = next_daily_reminder_utc_ms(now, 840, tz); // 14:00 local
        assert_eq!(result, now + 4 * 3_600_000);
    }

    #[test]
    fn next_daily_reminder_tomorrow_if_past() {
        let tz = TimezoneContext {
            utc_offset_seconds: 0,
            is_dst: false,
        };
        // UTC 14:00 (840 min). Target is 10:00 (600 min). Should be tomorrow 10:00.
        let now = 14 * 3_600_000i64;
        let result = next_daily_reminder_utc_ms(now, 600, tz);
        // Tomorrow 10:00 = now + 20 hours (from 14:00 today to 10:00 tomorrow)
        assert_eq!(result, now + 20 * 3_600_000);
    }

    // ── NotificationSchedule serialization ──

    #[test]
    fn notification_schedule_immediate_serializes() {
        let s = NotificationSchedule::Immediate;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"immediate\"");
        let back: NotificationSchedule = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn notification_schedule_scheduled_serializes() {
        let s = NotificationSchedule::Scheduled { at_utc_ms: 12345 };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"at_utc_ms\""));
        let back: NotificationSchedule = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn notification_schedule_daily_serializes() {
        let s = NotificationSchedule::Daily {
            at_local_minute: 600,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"at_local_minute\""));
        let back: NotificationSchedule = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    // ── Full round-trip serialization ──

    #[test]
    fn notification_preferences_round_trip() {
        let prefs = NotificationPreferences {
            reminders_enabled: true,
            achievements_enabled: false,
            coaching_enabled: true,
            system_enabled: true,
            recovery_enabled: false,
            critical_bypasses_quiet_hours: true,
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: NotificationPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(prefs, back);
    }

    #[test]
    fn notification_decision_round_trip() {
        let ctx = NotificationDecisionContext {
            notif_type: NotificationType::ScheduledWorkout,
            preferences: NotificationPreferences {
                reminders_enabled: true,
                ..Default::default()
            },
            permission: NotificationPermission::Granted,
            quiet_hours: QuietHoursConfig::default(),
            now_utc_ms: 0,
            timezone: TimezoneContext::default(),
            context_params: NotificationContextParams::default(),
        };
        let decision = decide_notification(&ctx);
        let json = serde_json::to_string(&decision).unwrap();
        let back: NotificationDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(decision, back);
    }

    #[test]
    fn notification_status_round_trip() {
        let status = build_notification_status(
            NotificationPermission::Granted,
            NotificationPreferences::default(),
            QuietHoursConfig::default(),
            &VoiceCoachingConfig::default(),
            TimezoneContext::default(),
        );
        let json = serde_json::to_string(&status).unwrap();
        let back: NotificationStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back);
    }

    #[test]
    fn all_enums_use_snake_case_serde() {
        // Verify serde naming convention for all enums.
        let cat = NotificationCategory::Reminders;
        assert_eq!(
            serde_json::to_string(&cat).unwrap(),
            "\"reminders\""
        );

        let nt = NotificationType::ScheduledWorkout;
        assert_eq!(
            serde_json::to_string(&nt).unwrap(),
            "\"scheduled_workout\""
        );

        let pri = NotificationPriority::High;
        assert_eq!(serde_json::to_string(&pri).unwrap(), "\"high\"");

        let perm = NotificationPermission::NotRequested;
        assert_eq!(
            serde_json::to_string(&perm).unwrap(),
            "\"not_requested\""
        );

        let mode = CoachingDeliveryMode::Voice;
        assert_eq!(serde_json::to_string(&mode).unwrap(), "\"voice\"");

        let kind = CoachingAnnouncementKind::Distance;
        assert_eq!(
            serde_json::to_string(&kind).unwrap(),
            "\"distance\""
        );
    }
}
