//! Background execution and foreground service management (spec section 12).
//!
//! This module is the pure decision logic for keeping a workout alive when
//! the app goes to the background, the screen turns off, the process is
//! killed, or the device restarts. The actual Android foreground service,
//! notification posting, and checkpoint persistence happen on the Kotlin /
//! Dart side — this module centralises the *decisions* so they are
//! consistent, testable, and independent of platform APIs.
//!
//! Key concepts:
//!
//! - **Foreground service state machine** — tracks whether the foreground
//!   service is running, what notification it should show, and when to
//!   start / stop / upgrade / downgrade it.
//! - **Checkpoint scheduling** — decides how often to write a checkpoint
//!   to local storage so a crash or process-kill doesn't lose more than
//!   a small amount of workout data.
//! - **Background execution policy** — decides whether background work is
//!   permitted right now, given user preferences, battery state, and
//!   workout state.
//! - **Process-kill resilience** — evaluates whether a recovered workout
//!   can continue or must be restarted after the process was killed.
//! - **User controls** — the user can explicitly pause, resume, or stop
//!   background tracking. Their preference is respected.
//! - **Reduced-power mode** — when battery is low or saver is on, the
//!   engine can reduce GPS / sensor frequency (delegated to
//!   [`battery::choose_sampling_profile`]) and this module decides whether
//!   to *also* reduce checkpoint frequency and notification updates.

use serde::{Deserialize, Serialize};

// ─── Foreground service state ──────────────────────────────────────

/// The state of the Android foreground workout service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForegroundServiceState {
    /// No workout is active; the service is not running.
    Stopped,
    /// The service is starting; the persistent notification is being posted.
    Starting,
    /// The service is running with an active workout; the persistent
    /// notification is visible and updated periodically.
    Running,
    /// The workout is paused; the service stays alive (so it can
    /// resume) but the notification shows a paused state.
    Paused,
    /// The service is being shut down; cleanup is in progress.
    Stopping,
}

impl Default for ForegroundServiceState {
    fn default() -> Self {
        ForegroundServiceState::Stopped
    }
}

impl ForegroundServiceState {
    /// Returns `true` if the foreground service process is alive.
    pub fn is_alive(self) -> bool {
        matches!(
            self,
            ForegroundServiceState::Starting
                | ForegroundServiceState::Running
                | ForegroundServiceState::Paused
        )
    }

    /// Returns `true` if the service is actively tracking (not paused).
    pub fn is_tracking(self) -> bool {
        matches!(
            self,
            ForegroundServiceState::Starting | ForegroundServiceState::Running
        )
    }

    /// Human-readable label for the notification / UI.
    pub fn label(self) -> &'static str {
        match self {
            ForegroundServiceState::Stopped => "Workout stopped",
            ForegroundServiceState::Starting => "Starting workout…",
            ForegroundServiceState::Running => "Workout in progress",
            ForegroundServiceState::Paused => "Workout paused",
            ForegroundServiceState::Stopping => "Stopping workout…",
        }
    }
}

/// Commands that can be sent to the foreground service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForegroundServiceCommand {
    /// Start the service and begin tracking.
    Start,
    /// Pause tracking (keep service alive for resume).
    Pause,
    /// Resume tracking from a pause.
    Resume,
    /// Stop the service and release resources.
    Stop,
}

/// The result of a foreground service state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForegroundServiceTransition {
    pub previous_state: ForegroundServiceState,
    pub new_state: ForegroundServiceState,
    pub accepted: bool,
    pub notification_text: String,
    pub should_keep_service_alive: bool,
    pub message: String,
}

/// Transitions the foreground service state machine.
///
/// Valid transitions:
/// - Start: Stopped → Starting → Running
/// - Pause: Running → Paused
/// - Resume: Paused → Running
/// - Stop: Running/Paused/Starting → Stopping → Stopped
pub fn transition_service(
    current: ForegroundServiceState,
    command: ForegroundServiceCommand,
) -> ForegroundServiceTransition {
    let (new_state, accepted, message) = match (current, command) {
        (ForegroundServiceState::Stopped, ForegroundServiceCommand::Start) => (
            ForegroundServiceState::Starting,
            true,
            "Starting foreground workout service.".to_string(),
        ),
        (ForegroundServiceState::Starting, ForegroundServiceCommand::Start) => (
            ForegroundServiceState::Starting,
            false,
            "Service is already starting.".to_string(),
        ),
        (ForegroundServiceState::Starting, ForegroundServiceCommand::Pause) => (
            ForegroundServiceState::Paused,
            true,
            "Pausing during startup; service stays alive for resume.".to_string(),
        ),
        (ForegroundServiceState::Starting, ForegroundServiceCommand::Resume) => (
            ForegroundServiceState::Running,
            true,
            "Resuming; completing startup and entering running state.".to_string(),
        ),
        (ForegroundServiceState::Running, ForegroundServiceCommand::Start) => (
            ForegroundServiceState::Running,
            false,
            "Service is already running.".to_string(),
        ),
        (ForegroundServiceState::Paused, ForegroundServiceCommand::Start) => (
            ForegroundServiceState::Paused,
            false,
            "Cannot start a paused service; use Resume instead.".to_string(),
        ),
        (ForegroundServiceState::Running, ForegroundServiceCommand::Pause) => (
            ForegroundServiceState::Paused,
            true,
            "Workout paused; service stays alive for resume.".to_string(),
        ),
        (ForegroundServiceState::Paused, ForegroundServiceCommand::Resume) => (
            ForegroundServiceState::Running,
            true,
            "Resuming workout tracking.".to_string(),
        ),
        (ForegroundServiceState::Paused, ForegroundServiceCommand::Pause) => (
            ForegroundServiceState::Paused,
            false,
            "Workout is already paused.".to_string(),
        ),
        (ForegroundServiceState::Running, ForegroundServiceCommand::Resume) => (
            ForegroundServiceState::Running,
            false,
            "Workout is already running.".to_string(),
        ),
        (
            ForegroundServiceState::Running
            | ForegroundServiceState::Paused
            | ForegroundServiceState::Starting,
            ForegroundServiceCommand::Stop,
        ) => (
            ForegroundServiceState::Stopping,
            true,
            "Stopping foreground workout service.".to_string(),
        ),
        (ForegroundServiceState::Stopped, ForegroundServiceCommand::Stop) => (
            ForegroundServiceState::Stopped,
            false,
            "Service is already stopped.".to_string(),
        ),
        (ForegroundServiceState::Stopped, ForegroundServiceCommand::Pause) => (
            ForegroundServiceState::Stopped,
            false,
            "Cannot pause a stopped service.".to_string(),
        ),
        (ForegroundServiceState::Stopped, ForegroundServiceCommand::Resume) => (
            ForegroundServiceState::Stopped,
            false,
            "Cannot resume a stopped service.".to_string(),
        ),
        (ForegroundServiceState::Stopping, _) => (
            ForegroundServiceState::Stopping,
            false,
            "Service is shutting down; ignoring command.".to_string(),
        ),
    };

    ForegroundServiceTransition {
        notification_text: new_state.label().to_string(),
        should_keep_service_alive: new_state.is_alive(),
        previous_state: current,
        new_state,
        accepted,
        message,
    }
}

// ─── Checkpoint scheduling ─────────────────────────────────────────

/// Decides how often to write a checkpoint to local storage.
///
/// The checkpoint interval is the maximum amount of data the user is
/// willing to lose if the process is killed. A shorter interval means
/// less data loss but more I/O. The interval is adjusted based on the
/// workout phase and battery state:
///
/// - **Normal (active, good battery)**: 30 seconds — at most 30s of data
///   lost on a crash.
/// - **Reduced power (battery saver / low battery)**: 60 seconds — less
///   I/O to save battery, at the cost of up to 1 minute of data loss.
/// - **Paused**: 120 seconds — while paused, GPS is off; checkpoints
///   are mainly to preserve the paused state itself.
/// - **Cool-down / ending phase**: 15 seconds — as the workout is
///   wrapping up, write more frequently so the final save is clean.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSchedule {
    /// How often to write a checkpoint, in milliseconds.
    pub interval_ms: u64,
    /// Maximum acceptable data loss on a crash, in milliseconds.
    pub max_data_loss_ms: u64,
    /// Human-readable reason for the chosen interval.
    pub reason: String,
}

/// The phase of the workout, which affects checkpoint frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkoutPhase {
    /// Workout is actively recording GPS / sensor data.
    Active,
    /// Workout is paused (auto-pause or user pause).
    Paused,
    /// Workout is in the final cool-down phase (approaching stop).
    Ending,
}

/// Battery state relevant to checkpoint scheduling.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CheckpointBatteryContext {
    pub battery_percent: u8,
    pub battery_saver_enabled: bool,
    pub is_charging: bool,
}

impl Default for CheckpointBatteryContext {
    fn default() -> Self {
        CheckpointBatteryContext {
            battery_percent: 100,
            battery_saver_enabled: false,
            is_charging: true,
        }
    }
}

/// Decides the checkpoint write interval.
pub fn decide_checkpoint_interval(
    phase: WorkoutPhase,
    battery: CheckpointBatteryContext,
) -> CheckpointSchedule {
    match phase {
        WorkoutPhase::Ending => CheckpointSchedule {
            interval_ms: 15_000,
            max_data_loss_ms: 15_000,
            reason: "Ending phase: frequent checkpoints for a clean final save.".to_string(),
        },
        WorkoutPhase::Paused => CheckpointSchedule {
            interval_ms: 120_000,
            max_data_loss_ms: 120_000,
            reason: "Paused: infrequent checkpoints; GPS is off.".to_string(),
        },
        WorkoutPhase::Active => {
            // Charging overrides low-power considerations.
            if battery.is_charging {
                return CheckpointSchedule {
                    interval_ms: 30_000,
                    max_data_loss_ms: 30_000,
                    reason: "Active (charging): standard checkpoint interval (max 30s data loss).".to_string(),
                };
            }
            let low_power = battery.battery_saver_enabled || battery.battery_percent < 20;
            if low_power {
                CheckpointSchedule {
                    interval_ms: 60_000,
                    max_data_loss_ms: 60_000,
                    reason: "Active (low power): reduced checkpoint frequency to save battery.".to_string(),
                }
            } else {
                CheckpointSchedule {
                    interval_ms: 30_000,
                    max_data_loss_ms: 30_000,
                    reason: "Active: standard checkpoint interval (max 30s data loss).".to_string(),
                }
            }
        }
    }
}

// ─── Background execution policy ───────────────────────────────────

/// User preference for background tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundTrackingPreference {
    /// User has not yet chosen; default to enabled with explanation.
    Unset,
    /// User explicitly enabled background tracking.
    Enabled,
    /// User explicitly disabled background tracking (foreground-only).
    Disabled,
}

impl Default for BackgroundTrackingPreference {
    fn default() -> Self {
        BackgroundTrackingPreference::Unset
    }
}

/// The decision on whether background execution is permitted right now.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundExecutionDecision {
    /// Whether background tracking is allowed.
    pub allowed: bool,
    /// Whether to keep the foreground service alive.
    pub keep_service_alive: bool,
    /// What to tell the user (for the notification / UI).
    pub user_message: String,
    /// Whether to prompt the user for a decision.
    pub should_prompt_user: bool,
    /// The reason for the decision.
    pub reason: String,
}

/// The full context for a background execution decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundExecutionContext {
    /// User's preference for background tracking.
    pub preference: BackgroundTrackingPreference,
    /// Whether the foreground service is currently running.
    pub service_running: bool,
    /// Whether background location permission is granted.
    pub background_location_granted: bool,
    /// Whether the workout is currently active (not paused).
    pub workout_active: bool,
    /// Battery context.
    pub battery: CheckpointBatteryContext,
}

/// Decides whether background execution is permitted right now.
///
/// Rules:
/// 1. If the user explicitly **disabled** background tracking, no
///    background work is permitted. The workout can continue in
///    foreground-only mode until the user backgrounds the app, at which
///    point it pauses.
/// 2. If background location permission is **not granted**, background
///    tracking is not possible; the workout degrades to foreground-only.
/// 3. If battery is critically low (<5%) and not charging, background
///    tracking is suspended to preserve emergency battery; the user
///    is prompted to charge.
/// 4. Otherwise, background tracking is permitted.
pub fn evaluate_background_execution(
    ctx: &BackgroundExecutionContext,
) -> BackgroundExecutionDecision {
    // User explicitly disabled.
    if ctx.preference == BackgroundTrackingPreference::Disabled {
        return BackgroundExecutionDecision {
            allowed: false,
            keep_service_alive: ctx.workout_active,
            user_message: "Background tracking is disabled. Workout will pause when you leave the app.".to_string(),
            should_prompt_user: false,
            reason: "User disabled background tracking.".to_string(),
        };
    }

    // No background location permission.
    if !ctx.background_location_granted {
        return BackgroundExecutionDecision {
            allowed: false,
            keep_service_alive: ctx.workout_active,
            user_message: "Background location permission is not granted. Tracking will pause when the app goes to the background.".to_string(),
            should_prompt_user: true,
            reason: "Background location permission missing; foreground-only fallback.".to_string(),
        };
    }

    // Critically low battery and not charging.
    if ctx.battery.battery_percent < 5 && !ctx.battery.is_charging {
        return BackgroundExecutionDecision {
            allowed: false,
            keep_service_alive: true,
            user_message: "Battery is critically low. Background tracking is suspended to preserve emergency battery. Please charge your device.".to_string(),
            should_prompt_user: true,
            reason: "Critical battery (<5%) and not charging; suspending background work.".to_string(),
        };
    }

    // Preference is unset — allow but prompt the user to choose.
    if ctx.preference == BackgroundTrackingPreference::Unset {
        return BackgroundExecutionDecision {
            allowed: true,
            keep_service_alive: true,
            user_message: "Background tracking is enabled by default. You can change this in Settings.".to_string(),
            should_prompt_user: true,
            reason: "Preference unset; defaulting to enabled with user prompt.".to_string(),
        };
    }

    // All checks pass — background tracking is permitted.
    BackgroundExecutionDecision {
        allowed: true,
        keep_service_alive: true,
        user_message: "Background tracking is active.".to_string(),
        should_prompt_user: false,
        reason: "All conditions met for background tracking.".to_string(),
    }
}

// ─── Process-kill resilience ───────────────────────────────────────

/// The result of evaluating whether a workout can resume after a
/// process kill or device restart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessKillRecoveryDecision {
    /// Whether the workout can be resumed.
    pub can_resume: bool,
    /// Whether the checkpoint data is fresh enough to trust.
    pub checkpoint_is_fresh: bool,
    /// Maximum data loss in milliseconds (time since last checkpoint).
    pub estimated_data_loss_ms: i64,
    /// Recommended action for the user.
    pub recommendation: ProcessKillRecommendation,
    /// Human-readable explanation.
    pub message: String,
}

/// What to recommend to the user after a process kill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessKillRecommendation {
    /// Resume the workout from the checkpoint.
    Resume,
    /// Finish and save the workout as-is (data loss too large for resume).
    FinishAndSave,
    /// Discard the workout (checkpoint is corrupt or too old).
    Discard,
    /// No recovery needed (no checkpoint was found).
    NothingToRecover,
}

/// Evaluates whether a workout can be resumed after a process kill.
///
/// `checkpoint_age_ms` is the time elapsed since the last checkpoint was
/// written. `was_active` indicates whether the workout was actively
/// recording when the checkpoint was written.
pub fn evaluate_process_kill_recovery(
    checkpoint_age_ms: i64,
    was_active: bool,
) -> ProcessKillRecoveryDecision {
    // No active workout to recover.
    if !was_active {
        return ProcessKillRecoveryDecision {
            can_resume: false,
            checkpoint_is_fresh: false,
            estimated_data_loss_ms: checkpoint_age_ms,
            recommendation: ProcessKillRecommendation::NothingToRecover,
            message: "No active workout was found in the checkpoint.".to_string(),
        };
    }

    // Negative age means clock skew / corrupt timestamp.
    if checkpoint_age_ms < 0 {
        return ProcessKillRecoveryDecision {
            can_resume: false,
            checkpoint_is_fresh: false,
            estimated_data_loss_ms: 0,
            recommendation: ProcessKillRecommendation::Discard,
            message: "Checkpoint timestamp is invalid (clock skew detected).".to_string(),
        };
    }

    // Fresh checkpoint: less than 5 minutes old.
    if checkpoint_age_ms <= 5 * 60 * 1000 {
        return ProcessKillRecoveryDecision {
            can_resume: true,
            checkpoint_is_fresh: true,
            estimated_data_loss_ms: checkpoint_age_ms,
            recommendation: ProcessKillRecommendation::Resume,
            message: "Recent checkpoint found; workout can be resumed with minimal data loss.".to_string(),
        };
    }

    // Stale but recoverable: 5 minutes to 6 hours.
    if checkpoint_age_ms <= 6 * 60 * 60 * 1000 {
        let minutes = checkpoint_age_ms / 60_000;
        return ProcessKillRecoveryDecision {
            can_resume: true,
            checkpoint_is_fresh: false,
            estimated_data_loss_ms: checkpoint_age_ms,
            recommendation: ProcessKillRecommendation::Resume,
            message: format!(
                "Checkpoint is {} minutes old. You may resume, but some tracking data may be missing for that period.",
                minutes
            ),
        };
    }

    // Very stale: more than 6 hours. Recommend finishing.
    let hours = checkpoint_age_ms / 3_600_000;
    ProcessKillRecoveryDecision {
        can_resume: false,
        checkpoint_is_fresh: false,
        estimated_data_loss_ms: checkpoint_age_ms,
        recommendation: ProcessKillRecommendation::FinishAndSave,
        message: format!(
            "Checkpoint is {} hours old; resuming live tracking is unreliable. Please finish and save the workout.",
            hours
        ),
    }
}

// ─── Reduced-power mode ────────────────────────────────────────────

/// Reduced-power mode decision. When the battery is low or battery saver
/// is enabled, the engine can reduce the frequency of GPS updates, sensor
/// sampling, checkpoint writes, and notification updates. The actual GPS /
/// sensor sampling profile is decided by [`battery::choose_sampling_profile`];
/// this module decides the *additional* reductions for background
/// execution (checkpoint frequency and notification update interval).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerMode {
    /// Full power: all systems at normal frequency.
    Full,
    /// Reduced power: longer intervals, less I/O.
    Reduced,
    /// Critical power: minimal background work.
    Critical,
}

/// Decides the power mode based on battery state and charging status.
pub fn decide_power_mode(battery: &CheckpointBatteryContext) -> PowerMode {
    if battery.is_charging {
        return PowerMode::Full;
    }
    if battery.battery_percent < 5 {
        return PowerMode::Critical;
    }
    if battery.battery_saver_enabled || battery.battery_percent < 20 {
        return PowerMode::Reduced;
    }
    PowerMode::Full
}

/// Returns the notification update interval for the given power mode.
///
/// The persistent notification doesn't need to update every second — it
/// just needs to show the current distance / time / pace. In reduced
/// power mode, we update it less frequently.
pub fn notification_update_interval_ms(mode: PowerMode) -> u64 {
    match mode {
        PowerMode::Full => 1_000,
        PowerMode::Reduced => 5_000,
        PowerMode::Critical => 15_000,
    }
}

// ─── Background location explanation ────────────────────────────────

/// Returns the human-readable explanation for why the app needs
/// background location permission. This text is used in:
/// - The in-app rationale dialog before the permission prompt.
/// - The Google Play Store data safety form.
/// - The app's privacy policy.
///
/// The explanation must be clear, specific, and honest about what the
/// data is used for — per Google Play policy, background location may
/// only be requested for a legitimate, clearly explained purpose.
pub fn background_location_explanation() -> &'static str {
    "S.T.R.I.D.E. uses background location to keep tracking your walk or run \
    when your phone screen is off or you switch to another app. This allows the \
    app to record your route, distance, pace, and calories continuously, even \
    when you are listening to music or using other apps. Your location data is \
    stored locally on your device and is used only to calculate your workout \
    metrics. It is never sold or shared with third parties. You can disable \
    background tracking at any time in Settings, in which case the app will \
    only track while it is open on your screen."
}

/// Returns a shorter version of the explanation for the notification
/// or a permission rationale dialog.
pub fn background_location_short_explanation() -> &'static str {
    "S.T.R.I.D.E. needs background location to track your workout when the \
    screen is off or you're using another app. Your route data stays on your \
    device and is never shared. You can turn this off anytime in Settings."
}

// ─── Interruption handling ──────────────────────────────────────────

/// An interruption event that can affect background tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptionEvent {
    /// The screen turned off.
    ScreenOff,
    /// The screen turned back on.
    ScreenOn,
    /// The user switched to another app.
    AppBackgrounded,
    /// The user returned to the app.
    AppForegrounded,
    /// An incoming phone call.
    IncomingCall,
    /// The phone call ended.
    CallEnded,
    /// A low-memory warning from the OS.
    LowMemory,
    /// The device is shutting down.
    DeviceShutdown,
}

/// The action to take in response to an interruption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptionAction {
    /// Continue tracking as normal.
    Continue,
    /// Pause tracking (but keep the service alive for resume).
    Pause,
    /// Write a checkpoint immediately and continue.
    WriteCheckpointAndContinue,
    /// Write a checkpoint immediately and stop.
    WriteCheckpointAndStop,
    /// Reduce power mode.
    ReducePower,
}

/// Decides what to do when an interruption event occurs during a
/// background workout.
///
/// - Screen off / app backgrounded: continue (that's the whole point of
///   background tracking).
/// - Incoming call: pause tracking (the user is on a call; resume after).
/// - Low memory: write a checkpoint immediately and continue (in case the
///   process is killed).
/// - Device shutdown: write a checkpoint and stop.
pub fn handle_interruption(
    event: InterruptionEvent,
    service_state: ForegroundServiceState,
    background_allowed: bool,
) -> InterruptionAction {
    match event {
        InterruptionEvent::ScreenOff | InterruptionEvent::AppBackgrounded => {
            if background_allowed {
                InterruptionAction::Continue
            } else {
                // No background permission — pause until the user returns.
                InterruptionAction::Pause
            }
        }
        InterruptionEvent::ScreenOn | InterruptionEvent::AppForegrounded => {
            InterruptionAction::Continue
        }
        InterruptionEvent::IncomingCall => {
            if service_state.is_tracking() {
                InterruptionAction::Pause
            } else {
                InterruptionAction::Continue
            }
        }
        InterruptionEvent::CallEnded => InterruptionAction::Continue,
        InterruptionEvent::LowMemory => InterruptionAction::WriteCheckpointAndContinue,
        InterruptionEvent::DeviceShutdown => InterruptionAction::WriteCheckpointAndStop,
    }
}

// ─── Battery-use testing ───────────────────────────────────────────

/// The result of a battery-use assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryUseAssessment {
    /// Whether the current battery usage is acceptable.
    pub is_acceptable: bool,
    /// Estimated battery drain per hour, in percentage points.
    pub estimated_drain_per_hour: f64,
    /// Whether to recommend the user enable battery saver.
    pub recommend_battery_saver: bool,
    /// Human-readable assessment.
    pub message: String,
}

/// Assesses the current battery usage and whether it's acceptable.
///
/// `gps_interval_ms` and `sensor_interval_ms` are the current sampling
/// intervals (from `battery::choose_sampling_profile`). `battery_percent`
/// is the current battery level.
pub fn assess_battery_use(
    gps_interval_ms: u32,
    sensor_interval_ms: u32,
    battery_percent: u8,
    is_charging: bool,
) -> BatteryUseAssessment {
    // Rough model: shorter intervals = more battery drain.
    // GPS at 1s = ~12%/hr, at 3s = ~6%/hr, at 10s = ~3%/hr.
    // Sensor at 500ms adds ~2%/hr, at 2s adds ~1%/hr, at 5s adds ~0.5%/hr.
    let gps_drain = match gps_interval_ms {
        0..=1_000 => 12.0,
        1_001..=3_000 => 6.0,
        _ => 3.0,
    };
    let sensor_drain = match sensor_interval_ms {
        0..=500 => 2.0,
        501..=2_000 => 1.0,
        _ => 0.5,
    };
    let estimated_drain = gps_drain + sensor_drain;

    if is_charging {
        return BatteryUseAssessment {
            is_acceptable: true,
            estimated_drain_per_hour: estimated_drain,
            recommend_battery_saver: false,
            message: "Device is charging; battery drain is not a concern.".to_string(),
        };
    }

    let is_acceptable = estimated_drain < 15.0 || battery_percent > 30;
    let recommend_battery_saver = battery_percent < 30 && estimated_drain > 8.0;

    let message = if is_acceptable {
        format!(
            "Battery drain is estimated at {:.1}%/hr. This is within acceptable limits.",
            estimated_drain
        )
    } else {
        format!(
            "Battery drain is estimated at {:.1}%/hr. Consider enabling battery saver to extend your workout.",
            estimated_drain
        )
    };

    BatteryUseAssessment {
        is_acceptable,
        estimated_drain_per_hour: estimated_drain,
        recommend_battery_saver,
        message,
    }
}

// ─── Full background status snapshot ────────────────────────────────

/// A full background execution status snapshot for the UI / diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundStatus {
    pub service_state: ForegroundServiceState,
    pub power_mode: PowerMode,
    pub background_allowed: bool,
    pub checkpoint_interval_ms: u64,
    pub notification_interval_ms: u64,
    pub preference: BackgroundTrackingPreference,
    pub estimated_drain_per_hour: f64,
    pub is_charging: bool,
    pub battery_percent: u8,
}

/// Builds a full background status snapshot from the current state.
#[allow(clippy::too_many_arguments)]
pub fn build_background_status(
    service_state: ForegroundServiceState,
    battery: &CheckpointBatteryContext,
    preference: BackgroundTrackingPreference,
    background_location_granted: bool,
    workout_active: bool,
    gps_interval_ms: u32,
    sensor_interval_ms: u32,
) -> BackgroundStatus {
    let exec_ctx = BackgroundExecutionContext {
        preference,
        service_running: service_state.is_alive(),
        background_location_granted,
        workout_active,
        battery: *battery,
    };
    let exec_decision = evaluate_background_execution(&exec_ctx);
    let power_mode = decide_power_mode(battery);
    let phase = if workout_active {
        WorkoutPhase::Active
    } else {
        WorkoutPhase::Paused
    };
    let checkpoint_schedule = decide_checkpoint_interval(phase, *battery);
    let battery_assessment =
        assess_battery_use(gps_interval_ms, sensor_interval_ms, battery.battery_percent, battery.is_charging);

    BackgroundStatus {
        service_state,
        power_mode,
        background_allowed: exec_decision.allowed,
        checkpoint_interval_ms: checkpoint_schedule.interval_ms,
        notification_interval_ms: notification_update_interval_ms(power_mode),
        preference,
        estimated_drain_per_hour: battery_assessment.estimated_drain_per_hour,
        is_charging: battery.is_charging,
        battery_percent: battery.battery_percent,
    }
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Foreground service state machine ──

    #[test]
    fn stopped_to_starting_on_start() {
        let r = transition_service(ForegroundServiceState::Stopped, ForegroundServiceCommand::Start);
        assert!(r.accepted);
        assert_eq!(r.new_state, ForegroundServiceState::Starting);
    }

    #[test]
    fn running_to_paused_on_pause() {
        let r = transition_service(ForegroundServiceState::Running, ForegroundServiceCommand::Pause);
        assert!(r.accepted);
        assert_eq!(r.new_state, ForegroundServiceState::Paused);
        assert!(r.should_keep_service_alive);
    }

    #[test]
    fn paused_to_running_on_resume() {
        let r = transition_service(ForegroundServiceState::Paused, ForegroundServiceCommand::Resume);
        assert!(r.accepted);
        assert_eq!(r.new_state, ForegroundServiceState::Running);
    }

    #[test]
    fn running_to_stopping_on_stop() {
        let r = transition_service(ForegroundServiceState::Running, ForegroundServiceCommand::Stop);
        assert!(r.accepted);
        assert_eq!(r.new_state, ForegroundServiceState::Stopping);
    }

    #[test]
    fn paused_to_stopping_on_stop() {
        let r = transition_service(ForegroundServiceState::Paused, ForegroundServiceCommand::Stop);
        assert!(r.accepted);
        assert_eq!(r.new_state, ForegroundServiceState::Stopping);
    }

    #[test]
    fn stopped_to_stopped_on_stop_is_rejected() {
        let r = transition_service(ForegroundServiceState::Stopped, ForegroundServiceCommand::Stop);
        assert!(!r.accepted);
        assert_eq!(r.new_state, ForegroundServiceState::Stopped);
    }

    #[test]
    fn cannot_pause_a_stopped_service() {
        let r = transition_service(ForegroundServiceState::Stopped, ForegroundServiceCommand::Pause);
        assert!(!r.accepted);
    }

    #[test]
    fn cannot_resume_a_stopped_service() {
        let r = transition_service(ForegroundServiceState::Stopped, ForegroundServiceCommand::Resume);
        assert!(!r.accepted);
    }

    #[test]
    fn starting_to_start_is_rejected() {
        let r = transition_service(ForegroundServiceState::Starting, ForegroundServiceCommand::Start);
        assert!(!r.accepted);
    }

    #[test]
    fn running_to_start_is_rejected() {
        let r = transition_service(ForegroundServiceState::Running, ForegroundServiceCommand::Start);
        assert!(!r.accepted);
    }

    #[test]
    fn stopping_ignores_all_commands() {
        let r = transition_service(ForegroundServiceState::Stopping, ForegroundServiceCommand::Pause);
        assert!(!r.accepted);
        assert_eq!(r.new_state, ForegroundServiceState::Stopping);
    }

    #[test]
    fn is_alive_correct() {
        assert!(!ForegroundServiceState::Stopped.is_alive());
        assert!(ForegroundServiceState::Starting.is_alive());
        assert!(ForegroundServiceState::Running.is_alive());
        assert!(ForegroundServiceState::Paused.is_alive());
        assert!(!ForegroundServiceState::Stopping.is_alive());
    }

    #[test]
    fn is_tracking_correct() {
        assert!(ForegroundServiceState::Starting.is_tracking());
        assert!(ForegroundServiceState::Running.is_tracking());
        assert!(!ForegroundServiceState::Paused.is_tracking());
        assert!(!ForegroundServiceState::Stopped.is_tracking());
    }

    // ── Checkpoint scheduling ──

    #[test]
    fn active_good_battery_30s_checkpoint() {
        let bat = CheckpointBatteryContext {
            battery_percent: 80,
            battery_saver_enabled: false,
            is_charging: false,
        };
        let s = decide_checkpoint_interval(WorkoutPhase::Active, bat);
        assert_eq!(s.interval_ms, 30_000);
    }

    #[test]
    fn active_low_battery_60s_checkpoint() {
        let bat = CheckpointBatteryContext {
            battery_percent: 10,
            battery_saver_enabled: false,
            is_charging: false,
        };
        let s = decide_checkpoint_interval(WorkoutPhase::Active, bat);
        assert_eq!(s.interval_ms, 60_000);
    }

    #[test]
    fn active_battery_saver_60s_checkpoint() {
        let bat = CheckpointBatteryContext {
            battery_percent: 50,
            battery_saver_enabled: true,
            is_charging: false,
        };
        let s = decide_checkpoint_interval(WorkoutPhase::Active, bat);
        assert_eq!(s.interval_ms, 60_000);
    }

    #[test]
    fn paused_120s_checkpoint() {
        let bat = CheckpointBatteryContext {
            battery_percent: 80,
            battery_saver_enabled: false,
            is_charging: false,
        };
        let s = decide_checkpoint_interval(WorkoutPhase::Paused, bat);
        assert_eq!(s.interval_ms, 120_000);
    }

    #[test]
    fn ending_15s_checkpoint() {
        let bat = CheckpointBatteryContext {
            battery_percent: 80,
            battery_saver_enabled: false,
            is_charging: false,
        };
        let s = decide_checkpoint_interval(WorkoutPhase::Ending, bat);
        assert_eq!(s.interval_ms, 15_000);
    }

    #[test]
    fn charging_overrides_low_battery_for_checkpoint() {
        let bat = CheckpointBatteryContext {
            battery_percent: 5,
            battery_saver_enabled: true,
            is_charging: true,
        };
        let s = decide_checkpoint_interval(WorkoutPhase::Active, bat);
        assert_eq!(s.interval_ms, 30_000);
    }

    // ── Background execution policy ──

    #[test]
    fn disabled_preference_blocks_background() {
        let ctx = BackgroundExecutionContext {
            preference: BackgroundTrackingPreference::Disabled,
            service_running: true,
            background_location_granted: true,
            workout_active: true,
            battery: CheckpointBatteryContext {
                battery_percent: 80,
                battery_saver_enabled: false,
                is_charging: false,
            },
        };
        let d = evaluate_background_execution(&ctx);
        assert!(!d.allowed);
        assert!(!d.should_prompt_user);
    }

    #[test]
    fn no_background_permission_blocks_background() {
        let ctx = BackgroundExecutionContext {
            preference: BackgroundTrackingPreference::Enabled,
            service_running: true,
            background_location_granted: false,
            workout_active: true,
            battery: CheckpointBatteryContext {
                battery_percent: 80,
                battery_saver_enabled: false,
                is_charging: false,
            },
        };
        let d = evaluate_background_execution(&ctx);
        assert!(!d.allowed);
        assert!(d.should_prompt_user);
    }

    #[test]
    fn critical_battery_blocks_background() {
        let ctx = BackgroundExecutionContext {
            preference: BackgroundTrackingPreference::Enabled,
            service_running: true,
            background_location_granted: true,
            workout_active: true,
            battery: CheckpointBatteryContext {
                battery_percent: 3,
                battery_saver_enabled: false,
                is_charging: false,
            },
        };
        let d = evaluate_background_execution(&ctx);
        assert!(!d.allowed);
        assert!(d.should_prompt_user);
    }

    #[test]
    fn unset_preference_allows_but_prompts() {
        let ctx = BackgroundExecutionContext {
            preference: BackgroundTrackingPreference::Unset,
            service_running: true,
            background_location_granted: true,
            workout_active: true,
            battery: CheckpointBatteryContext {
                battery_percent: 80,
                battery_saver_enabled: false,
                is_charging: false,
            },
        };
        let d = evaluate_background_execution(&ctx);
        assert!(d.allowed);
        assert!(d.should_prompt_user);
    }

    #[test]
    fn enabled_with_all_conditions_allows_background() {
        let ctx = BackgroundExecutionContext {
            preference: BackgroundTrackingPreference::Enabled,
            service_running: true,
            background_location_granted: true,
            workout_active: true,
            battery: CheckpointBatteryContext {
                battery_percent: 80,
                battery_saver_enabled: false,
                is_charging: false,
            },
        };
        let d = evaluate_background_execution(&ctx);
        assert!(d.allowed);
        assert!(!d.should_prompt_user);
    }

    #[test]
    fn critical_battery_with_charging_allows_background() {
        let ctx = BackgroundExecutionContext {
            preference: BackgroundTrackingPreference::Enabled,
            service_running: true,
            background_location_granted: true,
            workout_active: true,
            battery: CheckpointBatteryContext {
                battery_percent: 3,
                battery_saver_enabled: false,
                is_charging: true,
            },
        };
        let d = evaluate_background_execution(&ctx);
        assert!(d.allowed);
    }

    // ── Process-kill recovery ──

    #[test]
    fn fresh_checkpoint_resumes() {
        let d = evaluate_process_kill_recovery(60_000, true);
        assert!(d.can_resume);
        assert!(d.checkpoint_is_fresh);
        assert_eq!(d.recommendation, ProcessKillRecommendation::Resume);
    }

    #[test]
    fn stale_checkpoint_5min_boundary_still_fresh() {
        let d = evaluate_process_kill_recovery(5 * 60 * 1000, true);
        assert!(d.can_resume);
        assert!(d.checkpoint_is_fresh);
    }

    #[test]
    fn stale_checkpoint_10min_resumes_with_warning() {
        let d = evaluate_process_kill_recovery(10 * 60 * 1000, true);
        assert!(d.can_resume);
        assert!(!d.checkpoint_is_fresh);
        assert_eq!(d.recommendation, ProcessKillRecommendation::Resume);
    }

    #[test]
    fn very_stale_checkpoint_8hr_recommends_finish() {
        let d = evaluate_process_kill_recovery(8 * 60 * 60 * 1000, true);
        assert!(!d.can_resume);
        assert_eq!(d.recommendation, ProcessKillRecommendation::FinishAndSave);
    }

    #[test]
    fn inactive_workout_nothing_to_recover() {
        let d = evaluate_process_kill_recovery(60_000, false);
        assert!(!d.can_resume);
        assert_eq!(d.recommendation, ProcessKillRecommendation::NothingToRecover);
    }

    #[test]
    fn negative_age_discards() {
        let d = evaluate_process_kill_recovery(-1000, true);
        assert!(!d.can_resume);
        assert_eq!(d.recommendation, ProcessKillRecommendation::Discard);
    }

    // ── Power mode ──

    #[test]
    fn charging_is_full_power() {
        let bat = CheckpointBatteryContext {
            battery_percent: 5,
            battery_saver_enabled: true,
            is_charging: true,
        };
        assert_eq!(decide_power_mode(&bat), PowerMode::Full);
    }

    #[test]
    fn low_battery_is_critical() {
        let bat = CheckpointBatteryContext {
            battery_percent: 3,
            battery_saver_enabled: false,
            is_charging: false,
        };
        assert_eq!(decide_power_mode(&bat), PowerMode::Critical);
    }

    #[test]
    fn battery_saver_is_reduced() {
        let bat = CheckpointBatteryContext {
            battery_percent: 50,
            battery_saver_enabled: true,
            is_charging: false,
        };
        assert_eq!(decide_power_mode(&bat), PowerMode::Reduced);
    }

    #[test]
    fn low_battery_under_20_is_reduced() {
        let bat = CheckpointBatteryContext {
            battery_percent: 15,
            battery_saver_enabled: false,
            is_charging: false,
        };
        assert_eq!(decide_power_mode(&bat), PowerMode::Reduced);
    }

    #[test]
    fn normal_battery_is_full() {
        let bat = CheckpointBatteryContext {
            battery_percent: 80,
            battery_saver_enabled: false,
            is_charging: false,
        };
        assert_eq!(decide_power_mode(&bat), PowerMode::Full);
    }

    #[test]
    fn notification_interval_full_is_1s() {
        assert_eq!(notification_update_interval_ms(PowerMode::Full), 1_000);
    }

    #[test]
    fn notification_interval_reduced_is_5s() {
        assert_eq!(notification_update_interval_ms(PowerMode::Reduced), 5_000);
    }

    #[test]
    fn notification_interval_critical_is_15s() {
        assert_eq!(notification_update_interval_ms(PowerMode::Critical), 15_000);
    }

    // ── Interruption handling ──

    #[test]
    fn screen_off_with_permission_continues() {
        let a = handle_interruption(
            InterruptionEvent::ScreenOff,
            ForegroundServiceState::Running,
            true,
        );
        assert_eq!(a, InterruptionAction::Continue);
    }

    #[test]
    fn screen_off_without_permission_pauses() {
        let a = handle_interruption(
            InterruptionEvent::ScreenOff,
            ForegroundServiceState::Running,
            false,
        );
        assert_eq!(a, InterruptionAction::Pause);
    }

    #[test]
    fn incoming_call_pauses_tracking() {
        let a = handle_interruption(
            InterruptionEvent::IncomingCall,
            ForegroundServiceState::Running,
            true,
        );
        assert_eq!(a, InterruptionAction::Pause);
    }

    #[test]
    fn incoming_call_when_paused_continues() {
        let a = handle_interruption(
            InterruptionEvent::IncomingCall,
            ForegroundServiceState::Paused,
            true,
        );
        assert_eq!(a, InterruptionAction::Continue);
    }

    #[test]
    fn low_memory_writes_checkpoint() {
        let a = handle_interruption(
            InterruptionEvent::LowMemory,
            ForegroundServiceState::Running,
            true,
        );
        assert_eq!(a, InterruptionAction::WriteCheckpointAndContinue);
    }

    #[test]
    fn device_shutdown_writes_checkpoint_and_stops() {
        let a = handle_interruption(
            InterruptionEvent::DeviceShutdown,
            ForegroundServiceState::Running,
            true,
        );
        assert_eq!(a, InterruptionAction::WriteCheckpointAndStop);
    }

    #[test]
    fn app_foregrounded_continues() {
        let a = handle_interruption(
            InterruptionEvent::AppForegrounded,
            ForegroundServiceState::Running,
            true,
        );
        assert_eq!(a, InterruptionAction::Continue);
    }

    #[test]
    fn call_ended_continues() {
        let a = handle_interruption(
            InterruptionEvent::CallEnded,
            ForegroundServiceState::Paused,
            true,
        );
        assert_eq!(a, InterruptionAction::Continue);
    }

    // ── Battery-use assessment ──

    #[test]
    fn charging_makes_battery_acceptable() {
        let a = assess_battery_use(1_000, 500, 10, true);
        assert!(a.is_acceptable);
        assert!(!a.recommend_battery_saver);
    }

    #[test]
    fn high_drain_low_battery_recommends_saver() {
        let a = assess_battery_use(1_000, 500, 20, false);
        assert!(a.recommend_battery_saver);
    }

    #[test]
    fn high_drain_high_battery_is_acceptable() {
        let a = assess_battery_use(1_000, 500, 80, false);
        assert!(a.is_acceptable);
    }

    #[test]
    fn low_drain_is_acceptable() {
        let a = assess_battery_use(10_000, 5_000, 50, false);
        assert!(a.is_acceptable);
        assert!(!a.recommend_battery_saver);
    }

    // ── Background status ──

    #[test]
    fn build_status_full_running() {
        let bat = CheckpointBatteryContext {
            battery_percent: 80,
            battery_saver_enabled: false,
            is_charging: false,
        };
        let s = build_background_status(
            ForegroundServiceState::Running,
            &bat,
            BackgroundTrackingPreference::Enabled,
            true,
            true,
            1_000,
            500,
        );
        assert_eq!(s.service_state, ForegroundServiceState::Running);
        assert_eq!(s.power_mode, PowerMode::Full);
        assert!(s.background_allowed);
        assert_eq!(s.checkpoint_interval_ms, 30_000);
        assert_eq!(s.notification_interval_ms, 1_000);
    }

    #[test]
    fn build_status_paused_low_battery() {
        let bat = CheckpointBatteryContext {
            battery_percent: 10,
            battery_saver_enabled: true,
            is_charging: false,
        };
        let s = build_background_status(
            ForegroundServiceState::Paused,
            &bat,
            BackgroundTrackingPreference::Enabled,
            true,
            false,
            10_000,
            5_000,
        );
        assert_eq!(s.service_state, ForegroundServiceState::Paused);
        assert_eq!(s.power_mode, PowerMode::Reduced);
        assert_eq!(s.checkpoint_interval_ms, 120_000);
    }

    #[test]
    fn build_status_disabled_preference() {
        let bat = CheckpointBatteryContext {
            battery_percent: 80,
            battery_saver_enabled: false,
            is_charging: false,
        };
        let s = build_background_status(
            ForegroundServiceState::Running,
            &bat,
            BackgroundTrackingPreference::Disabled,
            true,
            true,
            1_000,
            500,
        );
        assert!(!s.background_allowed);
    }

    // ── Explanation text ──

    #[test]
    fn explanation_is_not_empty() {
        assert!(!background_location_explanation().is_empty());
        assert!(!background_location_short_explanation().is_empty());
    }

    #[test]
    fn explanation_mentions_tracking_and_privacy() {
        let full = background_location_explanation();
        assert!(full.contains("tracking"));
        assert!(full.contains("device") || full.contains("local"));
        assert!(full.contains("Settings") || full.contains("disable"));
    }

    // ── Serialization round-trip ──

    #[test]
    fn foreground_service_state_serializes_round_trip() {
        for s in [
            ForegroundServiceState::Stopped,
            ForegroundServiceState::Starting,
            ForegroundServiceState::Running,
            ForegroundServiceState::Paused,
            ForegroundServiceState::Stopping,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: ForegroundServiceState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn checkpoint_schedule_serializes_round_trip() {
        let s = CheckpointSchedule {
            interval_ms: 30_000,
            max_data_loss_ms: 30_000,
            reason: "test".to_string(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: CheckpointSchedule = serde_json::from_str(&json).unwrap();
        assert_eq!(s.interval_ms, back.interval_ms);
        assert_eq!(s.reason, back.reason);
    }

    #[test]
    fn background_execution_decision_serializes_round_trip() {
        let d = BackgroundExecutionDecision {
            allowed: true,
            keep_service_alive: true,
            user_message: "test".to_string(),
            should_prompt_user: false,
            reason: "test reason".to_string(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: BackgroundExecutionDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(d.allowed, back.allowed);
        assert_eq!(d.reason, back.reason);
    }

    #[test]
    fn process_kill_recovery_decision_serializes_round_trip() {
        let d = ProcessKillRecoveryDecision {
            can_resume: true,
            checkpoint_is_fresh: true,
            estimated_data_loss_ms: 60_000,
            recommendation: ProcessKillRecommendation::Resume,
            message: "test".to_string(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: ProcessKillRecoveryDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(d.can_resume, back.can_resume);
        assert_eq!(d.recommendation, back.recommendation);
    }

    #[test]
    fn power_mode_serializes_round_trip() {
        for m in [PowerMode::Full, PowerMode::Reduced, PowerMode::Critical] {
            let json = serde_json::to_string(&m).unwrap();
            let back: PowerMode = serde_json::from_str(&json).unwrap();
            assert_eq!(m, back);
        }
    }

    #[test]
    fn background_status_serializes_round_trip() {
        let s = BackgroundStatus {
            service_state: ForegroundServiceState::Running,
            power_mode: PowerMode::Full,
            background_allowed: true,
            checkpoint_interval_ms: 30_000,
            notification_interval_ms: 1_000,
            preference: BackgroundTrackingPreference::Enabled,
            estimated_drain_per_hour: 14.0,
            is_charging: false,
            battery_percent: 80,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: BackgroundStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s.background_allowed, back.background_allowed);
        assert_eq!(s.battery_percent, back.battery_percent);
    }

    #[test]
    fn foreground_service_transition_serializes_round_trip() {
        let t = ForegroundServiceTransition {
            previous_state: ForegroundServiceState::Stopped,
            new_state: ForegroundServiceState::Starting,
            accepted: true,
            notification_text: "Starting workout…".to_string(),
            should_keep_service_alive: true,
            message: "Starting foreground workout service.".to_string(),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: ForegroundServiceTransition = serde_json::from_str(&json).unwrap();
        assert_eq!(t.accepted, back.accepted);
        assert_eq!(t.new_state, back.new_state);
    }
}
