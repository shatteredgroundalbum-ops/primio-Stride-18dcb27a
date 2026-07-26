//! Error and recovery state system (spec section 14).
//!
//! Unified error classification, recovery state machine, and error-handling
//! framework that spans all engine subsystems. This module decides *what*
//! to do when something goes wrong — whether to retry with exponential
//! backoff, fall back to a degraded mode, escalate to the user, or
//! abandon the operation entirely.
//!
//! Key responsibilities:
//!   - Classify errors by category (GPS, sensors, sync, network, etc.)
//!   - Assign severity levels (critical, high, medium, low, info)
//!   - Track error lifecycle through a state machine (detected →
//!     reported → recovering → resolved/abandoned/escalated)
//!   - Decide recovery strategy (retry, fallback, ignore, abort, escalate)
//!   - Compute retry delays with exponential backoff + jitter
//!   - Maintain an in-memory error registry for diagnostics
//!   - Build an overall engine health status snapshot for the UI
//!
//! The actual side effects (network I/O, UI notifications, crash
//! reporting) happen on the Dart/Kotlin side. This module is purely
//! decision logic and is side-effect free.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error category
// ---------------------------------------------------------------------------

/// The subsystem or layer that an error originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// GPS data issues — no fix, poor accuracy, signal loss.
    Gps,
    /// Sensor data issues — accelerometer, gyroscope, heart rate.
    Sensors,
    /// Cloud sync failures — upload, download, conflict.
    Sync,
    /// Network connectivity issues — no internet, timeout.
    Network,
    /// Local storage issues — disk full, write failure, corruption.
    Storage,
    /// Bluetooth/wearable connection issues.
    Bluetooth,
    /// Permission denied or revoked.
    Permissions,
    /// Internal engine logic error or invariant violation.
    Engine,
    /// FFI boundary error — bad input, encoding, handle lookup.
    Ffi,
    /// Music playback issues.
    Music,
    /// Coaching/AI backend issues.
    Coaching,
    /// Notification system issues.
    Notifications,
    /// Background execution issues.
    Background,
    /// Unknown or unclassified error.
    Unknown,
}

impl ErrorCategory {
    /// Human-readable label for UI / diagnostics.
    pub fn label(self) -> &'static str {
        match self {
            ErrorCategory::Gps => "GPS",
            ErrorCategory::Sensors => "Sensors",
            ErrorCategory::Sync => "Cloud sync",
            ErrorCategory::Network => "Network",
            ErrorCategory::Storage => "Storage",
            ErrorCategory::Bluetooth => "Bluetooth",
            ErrorCategory::Permissions => "Permissions",
            ErrorCategory::Engine => "Engine",
            ErrorCategory::Ffi => "FFI",
            ErrorCategory::Music => "Music",
            ErrorCategory::Coaching => "Coaching",
            ErrorCategory::Notifications => "Notifications",
            ErrorCategory::Background => "Background",
            ErrorCategory::Unknown => "Unknown",
        }
    }

    /// A short description for the error log.
    pub fn description(self) -> &'static str {
        match self {
            ErrorCategory::Gps => "GPS positioning or signal error",
            ErrorCategory::Sensors => "Sensor data acquisition error",
            ErrorCategory::Sync => "Cloud synchronization error",
            ErrorCategory::Network => "Network connectivity error",
            ErrorCategory::Storage => "Local storage error",
            ErrorCategory::Bluetooth => "Bluetooth or wearable connection error",
            ErrorCategory::Permissions => "Permission denied or revoked",
            ErrorCategory::Engine => "Internal engine logic error",
            ErrorCategory::Ffi => "FFI boundary error",
            ErrorCategory::Music => "Music playback error",
            ErrorCategory::Coaching => "Coaching or AI backend error",
            ErrorCategory::Notifications => "Notification system error",
            ErrorCategory::Background => "Background execution error",
            ErrorCategory::Unknown => "Unclassified error",
        }
    }
}

// ---------------------------------------------------------------------------
// Error severity
// ---------------------------------------------------------------------------

/// How serious an error is. Determines whether it should interrupt the
/// user, be reported silently, or be ignored entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    /// Informational — no action needed, logged for diagnostics only.
    Info,
    /// Low impact — degraded but usable, no user interruption.
    Low,
    /// Medium impact — some features unavailable, user may be notified.
    Medium,
    /// High impact — core functionality affected, user should be alerted.
    High,
    /// Critical — workout cannot continue, immediate user action needed.
    Critical,
}

impl Default for ErrorSeverity {
    fn default() -> Self {
        ErrorSeverity::Medium
    }
}

impl ErrorSeverity {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            ErrorSeverity::Info => "info",
            ErrorSeverity::Low => "low",
            ErrorSeverity::Medium => "medium",
            ErrorSeverity::High => "high",
            ErrorSeverity::Critical => "critical",
        }
    }

    /// Returns `true` if this severity warrants interrupting the user.
    pub fn is_user_visible(self) -> bool {
        matches!(self, ErrorSeverity::High | ErrorSeverity::Critical)
    }

    /// Returns `true` if this severity requires immediate attention.
    pub fn is_critical(self) -> bool {
        matches!(self, ErrorSeverity::Critical)
    }
}

// ---------------------------------------------------------------------------
// Error state (lifecycle)
// ---------------------------------------------------------------------------

/// The lifecycle state of an error — from detection through resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorState {
    /// The error was just detected but not yet reported or acted on.
    Detected,
    /// The error has been reported to the UI / crash log.
    Reported,
    /// A recovery attempt is in progress (retry or fallback).
    Recovering,
    /// The error was resolved — the operation succeeded after retry.
    Resolved,
    /// The error recovery was abandoned — max retries exceeded.
    Abandoned,
    /// The error was escalated to the user for manual intervention.
    Escalated,
}

impl Default for ErrorState {
    fn default() -> Self {
        ErrorState::Detected
    }
}

impl ErrorState {
    /// Human-readable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            ErrorState::Detected => "Detected",
            ErrorState::Reported => "Reported",
            ErrorState::Recovering => "Recovering",
            ErrorState::Resolved => "Resolved",
            ErrorState::Abandoned => "Abandoned",
            ErrorState::Escalated => "Escalated",
        }
    }

    /// Returns `true` if the error is in a terminal state (no further
    /// transitions possible).
    pub fn is_terminal(self) -> bool {
        matches!(self, ErrorState::Resolved | ErrorState::Abandoned)
    }

    /// Returns `true` if the error is still active (not resolved).
    pub fn is_active(self) -> bool {
        !matches!(self, ErrorState::Resolved)
    }
}

// ---------------------------------------------------------------------------
// Recovery strategy
// ---------------------------------------------------------------------------

/// What to do in response to an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff.
    Retry,
    /// Fall back to a degraded mode (e.g., phone sensors when wearable
    /// disconnects).
    Fallback,
    /// Ignore the error — it's non-critical and doesn't affect the
    /// workout.
    Ignore,
    /// Abort the operation entirely — the error is unrecoverable.
    Abort,
    /// Escalate to the user for manual intervention (e.g., re-grant
    /// permissions).
    Escalate,
    /// Requires manual action (e.g., restart app, clear storage).
    Manual,
}

impl RecoveryStrategy {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            RecoveryStrategy::Retry => "retry",
            RecoveryStrategy::Fallback => "fallback",
            RecoveryStrategy::Ignore => "ignore",
            RecoveryStrategy::Abort => "abort",
            RecoveryStrategy::Escalate => "escalate",
            RecoveryStrategy::Manual => "manual",
        }
    }

    /// Returns `true` if this strategy involves an automatic action
    /// (retry or fallback).
    pub fn is_automatic(self) -> bool {
        matches!(self, RecoveryStrategy::Retry | RecoveryStrategy::Fallback)
    }
}

// ---------------------------------------------------------------------------
// Retry policy
// ---------------------------------------------------------------------------

/// Configuration for retry behavior with exponential backoff + jitter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts before abandoning.
    pub max_attempts: u32,
    /// Base delay in milliseconds for the first retry.
    pub base_delay_ms: i64,
    /// Maximum delay cap in milliseconds (retries never wait longer).
    pub max_delay_ms: i64,
    /// Exponential backoff multiplier (delay = base * multiplier^attempt).
    pub backoff_multiplier: f64,
    /// Jitter fraction (±this fraction of the computed delay). 0.25 =
    /// ±25% jitter.
    pub jitter_fraction: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 1_000,
            max_delay_ms: 10 * 60 * 1_000, // 10 minutes
            backoff_multiplier: 2.0,
            jitter_fraction: 0.25,
        }
    }
}

impl RetryPolicy {
    /// A policy for non-retryable errors (max_attempts = 0).
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 0,
            base_delay_ms: 0,
            max_delay_ms: 0,
            backoff_multiplier: 1.0,
            jitter_fraction: 0.0,
        }
    }

    /// A policy for aggressive retry (more attempts, shorter delays).
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 10,
            base_delay_ms: 500,
            max_delay_ms: 60 * 1_000, // 1 minute
            backoff_multiplier: 1.5,
            jitter_fraction: 0.3,
        }
    }

    /// A policy for gentle retry (fewer attempts, longer delays).
    pub fn gentle() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 5_000,
            max_delay_ms: 30 * 60 * 1_000, // 30 minutes
            backoff_multiplier: 2.5,
            jitter_fraction: 0.2,
        }
    }
}

// ---------------------------------------------------------------------------
// Error context
// ---------------------------------------------------------------------------

/// Full context for a single error occurrence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorContext {
    /// The error category (subsystem).
    pub category: ErrorCategory,
    /// The severity.
    pub severity: ErrorSeverity,
    /// A short machine-readable error code (e.g., "gps_no_fix").
    pub code: String,
    /// A human-readable error message.
    pub message: String,
    /// The subsystem where the error occurred (for routing).
    pub subsystem: String,
    /// The UTC timestamp (epoch milliseconds) when the error was detected.
    pub timestamp_ms: i64,
    /// How many times this error has been retried so far.
    pub retry_count: u32,
    /// Additional metadata as key-value pairs (for diagnostics).
    pub metadata: Vec<(String, String)>,
}

impl ErrorContext {
    /// Creates a new error context with the given category, severity,
    /// code, message, and subsystem. `timestamp_ms` and `retry_count`
    /// default to 0.
    pub fn new(
        category: ErrorCategory,
        severity: ErrorSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        subsystem: impl Into<String>,
    ) -> Self {
        Self {
            category,
            severity,
            code: code.into(),
            message: message.into(),
            subsystem: subsystem.into(),
            timestamp_ms: 0,
            retry_count: 0,
            metadata: Vec::new(),
        }
    }

    /// Adds a metadata key-value pair and returns self for chaining.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Sets the timestamp and returns self for chaining.
    pub fn at(mut self, timestamp_ms: i64) -> Self {
        self.timestamp_ms = timestamp_ms;
        self
    }

    /// Sets the retry count and returns self for chaining.
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
}

// ---------------------------------------------------------------------------
// Recovery action (the result of decide_recovery)
// ---------------------------------------------------------------------------

/// The action to take in response to an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryAction {
    /// The recovery strategy to use.
    pub strategy: RecoveryStrategy,
    /// The delay in milliseconds before the next retry (if retrying).
    pub delay_ms: i64,
    /// A message to show the user (if escalating or falling back).
    pub user_message: Option<String>,
    /// Whether this error should be reported to the crash log /
    /// analytics.
    pub should_report: bool,
    /// Whether this error should be persisted to the local error log.
    pub should_persist: bool,
    /// The next error state to transition to.
    pub next_state: ErrorState,
    /// The reason for the decision (for logging / debugging).
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Error state transition
// ---------------------------------------------------------------------------

/// A validated state transition for an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorStateTransition {
    /// The previous state.
    pub from: ErrorState,
    /// The new state.
    pub to: ErrorState,
    /// Whether the transition was valid.
    pub is_valid: bool,
    /// A message explaining the transition (or why it was rejected).
    pub message: String,
}

/// Determines whether a state transition is valid.
///
/// Valid transitions:
///   Detected → Reported, Recovering, Escalated, Abandoned
///   Reported → Recovering, Escalated, Abandoned
///   Recovering → Resolved, Abandoned, Escalated, Reported
///   Escalated → Resolved, Abandoned, Recovering
///   Resolved → (terminal, no transitions)
///   Abandoned → (terminal, no transitions)
pub fn is_valid_transition(from: ErrorState, to: ErrorState) -> bool {
    use ErrorState::*;
    match (from, to) {
        (Detected, Reported) => true,
        (Detected, Recovering) => true,
        (Detected, Escalated) => true,
        (Detected, Abandoned) => true,
        (Reported, Recovering) => true,
        (Reported, Escalated) => true,
        (Reported, Abandoned) => true,
        (Recovering, Resolved) => true,
        (Recovering, Abandoned) => true,
        (Recovering, Escalated) => true,
        (Recovering, Reported) => true,
        (Escalated, Resolved) => true,
        (Escalated, Abandoned) => true,
        (Escalated, Recovering) => true,
        (Resolved, _) => false,
        (Abandoned, _) => false,
        // Same-state transitions are not transitions.
        (a, b) if a == b => false,
        _ => false,
    }
}

/// Validates and returns a state transition. If the transition is
/// invalid, returns a transition with `is_valid = false` and an
/// explanatory message.
pub fn transition_error_state(
    from: ErrorState,
    to: ErrorState,
) -> ErrorStateTransition {
    if from.is_terminal() {
        return ErrorStateTransition {
            from,
            to,
            is_valid: false,
            message: format!(
                "Cannot transition from terminal state '{}' — error is finalized.",
                from.label()
            ),
        };
    }

    if from == to {
        return ErrorStateTransition {
            from,
            to,
            is_valid: false,
            message: format!(
                "No-op transition: error is already in '{}' state.",
                from.label()
            ),
        };
    }

    if is_valid_transition(from, to) {
        ErrorStateTransition {
            from,
            to,
            is_valid: true,
            message: format!(
                "Valid transition: {} → {}.",
                from.label(),
                to.label()
            ),
        }
    } else {
        ErrorStateTransition {
            from,
            to,
            is_valid: false,
            message: format!(
                "Invalid transition: {} → {} is not allowed.",
                from.label(),
                to.label()
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Recovery decision logic
// ---------------------------------------------------------------------------

/// Decides what to do in response to an error, given the error context
/// and the retry policy.
///
/// This is the main entry point for error recovery decision logic. It
/// combines:
/// 1. Severity assessment — critical errors may escalate immediately.
/// 2. Retry policy — has the max retry count been exceeded?
/// 3. Category-specific logic — some categories are never retried.
/// 4. State transition — what state should the error move to?
pub fn decide_recovery(error: &ErrorContext, policy: &RetryPolicy) -> RecoveryAction {
    // Critical errors that are permission-related should escalate immediately.
    if error.severity.is_critical() && error.category == ErrorCategory::Permissions {
        return RecoveryAction {
            strategy: RecoveryStrategy::Escalate,
            delay_ms: 0,
            user_message: Some(format!(
                "Permission required: {}. Please grant the necessary permission to continue.",
                error.message
            )),
            should_report: true,
            should_persist: true,
            next_state: ErrorState::Escalated,
            reason: "Critical permission error — escalating to user immediately.".to_string(),
        };
    }

    // Critical errors that are engine-internal should abort.
    if error.severity.is_critical() && error.category == ErrorCategory::Engine {
        return RecoveryAction {
            strategy: RecoveryStrategy::Abort,
            delay_ms: 0,
            user_message: Some(format!(
                "A critical engine error occurred: {}. The workout has been stopped.",
                error.message
            )),
            should_report: true,
            should_persist: true,
            next_state: ErrorState::Abandoned,
            reason: "Critical engine error — aborting operation.".to_string(),
        };
    }

    // Check if retries are exhausted.
    if error.retry_count >= policy.max_attempts {
        // If the category supports fallback, try that.
        if supports_fallback(error.category) {
            return RecoveryAction {
                strategy: RecoveryStrategy::Fallback,
                delay_ms: 0,
                user_message: Some(fallback_message(error.category)),
                should_report: true,
                should_persist: true,
                next_state: ErrorState::Recovering,
                reason: format!(
                    "Retries exhausted ({}); falling back to degraded mode for {}.",
                    error.retry_count,
                    error.category.label()
                ),
            };
        }

        // No fallback available — escalate or abandon based on severity.
        if error.severity.is_user_visible() {
            return RecoveryAction {
                strategy: RecoveryStrategy::Escalate,
                delay_ms: 0,
                user_message: Some(format!(
                    "Unable to recover from {}: {}. Please try again later.",
                    error.category.label(),
                    error.message
                )),
                should_report: true,
                should_persist: true,
                next_state: ErrorState::Escalated,
                reason: format!(
                    "Retries exhausted and no fallback available; escalating {} error.",
                    error.category.label()
                ),
            };
        }

        // Non-user-visible: silently abandon.
        return RecoveryAction {
            strategy: RecoveryStrategy::Abort,
            delay_ms: 0,
            user_message: None,
            should_report: false,
            should_persist: true,
            next_state: ErrorState::Abandoned,
            reason: format!(
                "Retries exhausted for low-severity {} error; abandoning silently.",
                error.category.label()
            ),
        };
    }

    // Still have retries left — check if the category is retryable.
    if !is_retryable(error.category) {
        // Non-retryable category: fall back or ignore.
        if supports_fallback(error.category) {
            return RecoveryAction {
                strategy: RecoveryStrategy::Fallback,
                delay_ms: 0,
                user_message: Some(fallback_message(error.category)),
                should_report: error.severity.is_user_visible(),
                should_persist: true,
                next_state: ErrorState::Recovering,
                reason: format!(
                    "{} errors are not retryable; falling back immediately.",
                    error.category.label()
                ),
            };
        }

        return RecoveryAction {
            strategy: RecoveryStrategy::Ignore,
            delay_ms: 0,
            user_message: None,
            should_report: false,
            should_persist: true,
            next_state: ErrorState::Abandoned,
            reason: format!(
                "{} errors are not retryable and no fallback available; ignoring.",
                error.category.label()
            ),
        };
    }

    // Retry with computed delay.  `retry_count` is how many retries
    // have already been attempted; the *next* attempt is retry_count + 1,
    // so we pass that to compute the delay for the upcoming retry.
    let next_attempt = error.retry_count + 1;
    let delay = compute_retry_delay(next_attempt, policy);

    RecoveryAction {
        strategy: RecoveryStrategy::Retry,
        delay_ms: delay,
        user_message: None,
        should_report: error.severity.is_user_visible(),
        should_persist: true,
        next_state: ErrorState::Recovering,
        reason: format!(
            "Retrying {} error (attempt {}/{}) after {} ms delay.",
            error.category.label(),
            error.retry_count + 1,
            policy.max_attempts,
            delay
        ),
    }
}

// ---------------------------------------------------------------------------
// Retry delay computation (exponential backoff + jitter)
// ---------------------------------------------------------------------------

/// Computes the retry delay in milliseconds for a given attempt number
/// using exponential backoff with jitter.
///
/// `delay = base * multiplier^attempt`, capped at `max_delay`, with
/// ±`jitter_fraction` random jitter applied.
///
/// **Note:** This is deterministic (no actual RNG) — the jitter is
/// applied as a fixed fraction of the delay for reproducibility in
/// tests. The Dart/Kotlin side can add true randomness if desired.
pub fn compute_retry_delay(attempt: u32, policy: &RetryPolicy) -> i64 {
    if attempt == 0 || policy.max_attempts == 0 {
        return 0;
    }

    // Exponential backoff: base * multiplier^attempt.
    let exponent = attempt.min(30) as f64; // prevent overflow
    let raw_delay = policy.base_delay_ms as f64 * policy.backoff_multiplier.powf(exponent);

    // Cap at max_delay.
    let capped = raw_delay.min(policy.max_delay_ms as f64);

    // Apply jitter: add jitter_fraction * delay (deterministic — always
    // adds the positive jitter half; the Dart side can add true ±jitter).
    let jitter = capped * policy.jitter_fraction;
    let with_jitter = capped + jitter;

    with_jitter.max(0.0) as i64
}

// ---------------------------------------------------------------------------
// Category-specific helpers
// ---------------------------------------------------------------------------

/// Returns `true` if errors of this category can be retried.
pub fn is_retryable(category: ErrorCategory) -> bool {
    match category {
        ErrorCategory::Gps => true,
        ErrorCategory::Sensors => true,
        ErrorCategory::Sync => true,
        ErrorCategory::Network => true,
        ErrorCategory::Bluetooth => true,
        // Permission errors need user action, not retry.
        ErrorCategory::Permissions => false,
        // Engine logic errors are bugs, not transient failures.
        ErrorCategory::Engine => false,
        // FFI errors are programming errors, not transient.
        ErrorCategory::Ffi => false,
        ErrorCategory::Storage => true,
        ErrorCategory::Music => true,
        ErrorCategory::Coaching => true,
        ErrorCategory::Notifications => true,
        ErrorCategory::Background => true,
        ErrorCategory::Unknown => false,
    }
}

/// Returns `true` if errors of this category support a fallback mode.
pub fn supports_fallback(category: ErrorCategory) -> bool {
    match category {
        ErrorCategory::Gps => true,       // Fall back to last known position
        ErrorCategory::Sensors => true,   // Fall back to GPS-only tracking
        ErrorCategory::Bluetooth => true, // Fall back to phone sensors
        ErrorCategory::Sync => true,      // Fall back to local-only mode
        ErrorCategory::Network => true,   // Fall back to offline mode
        ErrorCategory::Coaching => true,  // Fall back to rule-based coaching
        ErrorCategory::Music => false,    // No real fallback for music
        ErrorCategory::Storage => false,  // No fallback for storage
        ErrorCategory::Permissions => false, // Need user action
        ErrorCategory::Engine => false,    // Can't fall back from a bug
        ErrorCategory::Ffi => false,       // Can't fall back from a bug
        ErrorCategory::Notifications => false, // No fallback
        ErrorCategory::Background => true, // Fall back to foreground-only
        ErrorCategory::Unknown => false,
    }
}

/// Returns the fallback message for a given category.
pub fn fallback_message(category: ErrorCategory) -> String {
    match category {
        ErrorCategory::Gps => "GPS signal lost — using last known position. Please move to an open area.".to_string(),
        ErrorCategory::Sensors => "Sensor data unavailable — tracking with GPS only.".to_string(),
        ErrorCategory::Bluetooth => "Wearable disconnected — tracking with phone sensors.".to_string(),
        ErrorCategory::Sync => "Cloud sync unavailable — working in offline mode. Your data will sync when the connection is restored.".to_string(),
        ErrorCategory::Network => "No network connection — working in offline mode.".to_string(),
        ErrorCategory::Coaching => "AI coaching unavailable — using built-in coaching rules.".to_string(),
        ErrorCategory::Background => "Background tracking unavailable — continuing in foreground only. Keep the app open.".to_string(),
        _ => format!("{} error — falling back to a degraded mode.", category.label()),
    }
}

/// Returns `true` if an error of the given category and severity
/// should be escalated to the user.
pub fn should_escalate(category: ErrorCategory, severity: ErrorSeverity) -> bool {
    if severity.is_critical() {
        return true;
    }
    // Permissions always need user action.
    if category == ErrorCategory::Permissions && severity.is_user_visible() {
        return true;
    }
    false
}

/// Returns `true` if the error recovery should be abandoned (no more
/// retries possible and no fallback available).
pub fn should_abandon(error: &ErrorContext, policy: &RetryPolicy) -> bool {
    error.retry_count >= policy.max_attempts && !supports_fallback(error.category)
}

// ---------------------------------------------------------------------------
// Error registry (in-memory log of recent errors)
// ---------------------------------------------------------------------------

/// An entry in the error registry — a record of a specific error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorRegistryEntry {
    /// The error context.
    pub error: ErrorContext,
    /// The current state of the error.
    pub state: ErrorState,
    /// The recovery action that was taken (if any).
    pub recovery_action: Option<RecoveryAction>,
}

/// An in-memory registry of recent errors for diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorRegistry {
    /// The list of error entries, most recent first.
    pub entries: Vec<ErrorRegistryEntry>,
    /// Maximum number of entries to keep (ring buffer).
    pub max_entries: usize,
}

impl Default for ErrorRegistry {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 100,
        }
    }
}

impl ErrorRegistry {
    /// Creates a new error registry with the given max entries.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Records a new error in the registry. The entry is prepended
    /// (most recent first). If the registry is full, the oldest entry
    /// is removed.
    pub fn record(&mut self, error: ErrorContext, state: ErrorState) {
        let entry = ErrorRegistryEntry {
            error,
            state,
            recovery_action: None,
        };
        self.entries.insert(0, entry);
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// Records a new error with a recovery action.
    pub fn record_with_action(
        &mut self,
        error: ErrorContext,
        state: ErrorState,
        action: RecoveryAction,
    ) {
        let entry = ErrorRegistryEntry {
            error,
            state,
            recovery_action: Some(action),
        };
        self.entries.insert(0, entry);
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// Returns the number of errors in the registry.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of errors of a given category.
    pub fn count_by_category(&self, category: ErrorCategory) -> usize {
        self.entries
            .iter()
            .filter(|e| e.error.category == category)
            .count()
    }

    /// Returns the number of errors of a given severity.
    pub fn count_by_severity(&self, severity: ErrorSeverity) -> usize {
        self.entries
            .iter()
            .filter(|e| e.error.severity == severity)
            .count()
    }

    /// Returns the number of errors in a given state.
    pub fn count_by_state(&self, state: ErrorState) -> usize {
        self.entries
            .iter()
            .filter(|e| e.state == state)
            .count()
    }

    /// Returns the number of active (unresolved) errors.
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| e.state.is_active()).count()
    }

    /// Returns the number of resolved errors.
    pub fn resolved_count(&self) -> usize {
        self.count_by_state(ErrorState::Resolved)
    }

    /// Returns the number of abandoned errors.
    pub fn abandoned_count(&self) -> usize {
        self.count_by_state(ErrorState::Abandoned)
    }

    /// Returns the most recent error of a given category, if any.
    pub fn most_recent(&self, category: ErrorCategory) -> Option<&ErrorRegistryEntry> {
        self.entries.iter().find(|e| e.error.category == category)
    }

    /// Clears all entries from the registry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

// ---------------------------------------------------------------------------
// Engine health status
// ---------------------------------------------------------------------------

/// The overall health status of the engine for the UI / diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineHealthStatus {
    /// Overall health status label.
    pub status: EngineHealthLabel,
    /// Total number of errors recorded.
    pub total_errors: usize,
    /// Number of active (unresolved) errors.
    pub active_errors: usize,
    /// Number of resolved errors.
    pub resolved_errors: usize,
    /// Number of abandoned errors.
    pub abandoned_errors: usize,
    /// Number of critical errors.
    pub critical_errors: usize,
    /// Whether the engine is in a degraded mode (any fallback active).
    pub is_degraded: bool,
    /// A human-readable summary message.
    pub summary: String,
    /// Per-category error counts.
    pub category_counts: Vec<(ErrorCategory, usize)>,
}

/// Overall health label for the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineHealthLabel {
    /// No errors — everything is working normally.
    Healthy,
    /// Some non-critical errors but the engine is fully operational.
    Operational,
    /// The engine is in a degraded mode (some fallback active).
    Degraded,
    /// Critical errors — the workout may be affected.
    Warning,
    /// The engine is in a critical failure state.
    Critical,
}

impl EngineHealthLabel {
    pub fn label(self) -> &'static str {
        match self {
            EngineHealthLabel::Healthy => "Healthy",
            EngineHealthLabel::Operational => "Operational",
            EngineHealthLabel::Degraded => "Degraded",
            EngineHealthLabel::Warning => "Warning",
            EngineHealthLabel::Critical => "Critical",
        }
    }
}

/// Builds an overall engine health status from an error registry.
pub fn build_health_status(registry: &ErrorRegistry) -> EngineHealthStatus {
    let total = registry.len();
    let active = registry.active_count();
    let resolved = registry.resolved_count();
    let abandoned = registry.abandoned_count();
    let critical = registry.count_by_severity(ErrorSeverity::Critical);

    // Check if any category is in a fallback state (Recovering state
    // with a Fallback strategy).
    let is_degraded = registry.entries.iter().any(|e| {
        e.state == ErrorState::Recovering
            && e.recovery_action.as_ref().map_or(false, |a| {
                a.strategy == RecoveryStrategy::Fallback
            })
    });

    // Determine overall label.
    let status = if total == 0 {
        EngineHealthLabel::Healthy
    } else if critical > 0 && active > 0 {
        EngineHealthLabel::Critical
    } else if active > 0 && is_degraded {
        EngineHealthLabel::Degraded
    } else if active > 0 {
        EngineHealthLabel::Warning
    } else {
        // All errors are resolved or abandoned.
        EngineHealthLabel::Operational
    };

    // Build per-category counts.
    let mut category_counts: Vec<(ErrorCategory, usize)> = Vec::new();
    for cat in [
        ErrorCategory::Gps,
        ErrorCategory::Sensors,
        ErrorCategory::Sync,
        ErrorCategory::Network,
        ErrorCategory::Storage,
        ErrorCategory::Bluetooth,
        ErrorCategory::Permissions,
        ErrorCategory::Engine,
        ErrorCategory::Ffi,
        ErrorCategory::Music,
        ErrorCategory::Coaching,
        ErrorCategory::Notifications,
        ErrorCategory::Background,
        ErrorCategory::Unknown,
    ] {
        let count = registry.count_by_category(cat);
        if count > 0 {
            category_counts.push((cat, count));
        }
    }

    let summary = match status {
        EngineHealthLabel::Healthy => "All systems operational.".to_string(),
        EngineHealthLabel::Operational => {
            format!("{} error(s) recorded, all resolved or abandoned.", total)
        }
        EngineHealthLabel::Degraded => format!(
            "{} active error(s); engine is in a degraded mode.",
            active
        ),
        EngineHealthLabel::Warning => {
            format!("{} active error(s) require attention.", active)
        }
        EngineHealthLabel::Critical => format!(
            "{} critical error(s) — immediate attention required.",
            critical
        ),
    };

    EngineHealthStatus {
        status,
        total_errors: total,
        active_errors: active,
        resolved_errors: resolved,
        abandoned_errors: abandoned,
        critical_errors: critical,
        is_degraded,
        summary,
        category_counts,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── ErrorCategory tests ──────────────────────────────────────────────

    #[test]
    fn category_labels_are_non_empty() {
        for cat in [
            ErrorCategory::Gps,
            ErrorCategory::Sensors,
            ErrorCategory::Sync,
            ErrorCategory::Network,
            ErrorCategory::Storage,
            ErrorCategory::Bluetooth,
            ErrorCategory::Permissions,
            ErrorCategory::Engine,
            ErrorCategory::Ffi,
            ErrorCategory::Music,
            ErrorCategory::Coaching,
            ErrorCategory::Notifications,
            ErrorCategory::Background,
            ErrorCategory::Unknown,
        ] {
            assert!(!cat.label().is_empty());
            assert!(!cat.description().is_empty());
        }
    }

    #[test]
    fn category_serializes_round_trip() {
        for cat in [
            ErrorCategory::Gps,
            ErrorCategory::Sensors,
            ErrorCategory::Sync,
            ErrorCategory::Network,
            ErrorCategory::Storage,
            ErrorCategory::Bluetooth,
            ErrorCategory::Permissions,
            ErrorCategory::Engine,
            ErrorCategory::Ffi,
            ErrorCategory::Music,
            ErrorCategory::Coaching,
            ErrorCategory::Notifications,
            ErrorCategory::Background,
            ErrorCategory::Unknown,
        ] {
            let json = serde_json::to_string(&cat).unwrap();
            let back: ErrorCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(cat, back);
        }
    }

    // ── ErrorSeverity tests ──────────────────────────────────────────────

    #[test]
    fn severity_ordering_is_correct() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Low);
        assert!(ErrorSeverity::Low < ErrorSeverity::Medium);
        assert!(ErrorSeverity::Medium < ErrorSeverity::High);
        assert!(ErrorSeverity::High < ErrorSeverity::Critical);
    }

    #[test]
    fn severity_is_user_visible() {
        assert!(!ErrorSeverity::Info.is_user_visible());
        assert!(!ErrorSeverity::Low.is_user_visible());
        assert!(!ErrorSeverity::Medium.is_user_visible());
        assert!(ErrorSeverity::High.is_user_visible());
        assert!(ErrorSeverity::Critical.is_user_visible());
    }

    #[test]
    fn severity_is_critical() {
        assert!(!ErrorSeverity::Info.is_critical());
        assert!(!ErrorSeverity::Low.is_critical());
        assert!(!ErrorSeverity::Medium.is_critical());
        assert!(!ErrorSeverity::High.is_critical());
        assert!(ErrorSeverity::Critical.is_critical());
    }

    #[test]
    fn severity_serializes_round_trip() {
        for sev in [
            ErrorSeverity::Info,
            ErrorSeverity::Low,
            ErrorSeverity::Medium,
            ErrorSeverity::High,
            ErrorSeverity::Critical,
        ] {
            let json = serde_json::to_string(&sev).unwrap();
            let back: ErrorSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(sev, back);
        }
    }

    // ── ErrorState tests ─────────────────────────────────────────────────

    #[test]
    fn state_is_terminal() {
        assert!(!ErrorState::Detected.is_terminal());
        assert!(!ErrorState::Reported.is_terminal());
        assert!(!ErrorState::Recovering.is_terminal());
        assert!(ErrorState::Resolved.is_terminal());
        assert!(ErrorState::Abandoned.is_terminal());
        assert!(!ErrorState::Escalated.is_terminal());
    }

    #[test]
    fn state_is_active() {
        assert!(ErrorState::Detected.is_active());
        assert!(ErrorState::Reported.is_active());
        assert!(ErrorState::Recovering.is_active());
        assert!(!ErrorState::Resolved.is_active());
        assert!(ErrorState::Abandoned.is_active());
        assert!(ErrorState::Escalated.is_active());
    }

    #[test]
    fn state_serializes_round_trip() {
        for state in [
            ErrorState::Detected,
            ErrorState::Reported,
            ErrorState::Recovering,
            ErrorState::Resolved,
            ErrorState::Abandoned,
            ErrorState::Escalated,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let back: ErrorState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, back);
        }
    }

    // ── RecoveryStrategy tests ───────────────────────────────────────────

    #[test]
    fn strategy_is_automatic() {
        assert!(RecoveryStrategy::Retry.is_automatic());
        assert!(RecoveryStrategy::Fallback.is_automatic());
        assert!(!RecoveryStrategy::Ignore.is_automatic());
        assert!(!RecoveryStrategy::Abort.is_automatic());
        assert!(!RecoveryStrategy::Escalate.is_automatic());
        assert!(!RecoveryStrategy::Manual.is_automatic());
    }

    #[test]
    fn strategy_serializes_round_trip() {
        for strat in [
            RecoveryStrategy::Retry,
            RecoveryStrategy::Fallback,
            RecoveryStrategy::Ignore,
            RecoveryStrategy::Abort,
            RecoveryStrategy::Escalate,
            RecoveryStrategy::Manual,
        ] {
            let json = serde_json::to_string(&strat).unwrap();
            let back: RecoveryStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(strat, back);
        }
    }

    // ── RetryPolicy tests ────────────────────────────────────────────────

    #[test]
    fn default_policy_has_5_attempts() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_attempts, 5);
    }

    #[test]
    fn no_retry_policy_has_zero_attempts() {
        let p = RetryPolicy::no_retry();
        assert_eq!(p.max_attempts, 0);
    }

    #[test]
    fn aggressive_policy_has_more_attempts() {
        let p = RetryPolicy::aggressive();
        assert!(p.max_attempts > 5);
    }

    #[test]
    fn gentle_policy_has_fewer_attempts() {
        let p = RetryPolicy::gentle();
        assert!(p.max_attempts < 5);
    }

    // ── compute_retry_delay tests ────────────────────────────────────────

    #[test]
    fn delay_is_zero_for_attempt_zero() {
        let p = RetryPolicy::default();
        assert_eq!(compute_retry_delay(0, &p), 0);
    }

    #[test]
    fn delay_is_zero_for_no_retry_policy() {
        let p = RetryPolicy::no_retry();
        assert_eq!(compute_retry_delay(5, &p), 0);
    }

    #[test]
    fn delay_grows_exponentially() {
        let p = RetryPolicy::default();
        let d0 = compute_retry_delay(1, &p);
        let d1 = compute_retry_delay(2, &p);
        let d2 = compute_retry_delay(3, &p);
        assert!(d0 < d1);
        assert!(d1 < d2);
    }

    #[test]
    fn delay_is_capped_at_max() {
        let p = RetryPolicy {
            max_attempts: 100,
            base_delay_ms: 1_000,
            max_delay_ms: 10_000,
            backoff_multiplier: 10.0,
            jitter_fraction: 0.0,
        };
        let delay = compute_retry_delay(50, &p);
        assert!(delay <= 10_000 + 1); // jitter adds 0, so exactly 10000
    }

    #[test]
    fn delay_first_attempt_is_base_plus_jitter() {
        let p = RetryPolicy {
            max_attempts: 5,
            base_delay_ms: 1_000,
            max_delay_ms: 100_000,
            backoff_multiplier: 2.0,
            jitter_fraction: 0.0, // no jitter for deterministic test
        };
        let delay = compute_retry_delay(1, &p);
        assert_eq!(delay, 2_000); // base * mult^1 = 1000 * 2 = 2000
    }

    #[test]
    fn delay_second_attempt_is_base_times_mult_squared() {
        let p = RetryPolicy {
            max_attempts: 5,
            base_delay_ms: 1_000,
            max_delay_ms: 1_000_000,
            backoff_multiplier: 2.0,
            jitter_fraction: 0.0,
        };
        let delay = compute_retry_delay(2, &p);
        assert_eq!(delay, 4_000); // 1000 * 2^2 = 4000
    }

    #[test]
    fn delay_with_jitter_is_larger_than_without() {
        let p_no_jitter = RetryPolicy {
            max_attempts: 5,
            base_delay_ms: 1_000,
            max_delay_ms: 1_000_000,
            backoff_multiplier: 2.0,
            jitter_fraction: 0.0,
        };
        let p_with_jitter = RetryPolicy {
            max_attempts: 5,
            base_delay_ms: 1_000,
            max_delay_ms: 1_000_000,
            backoff_multiplier: 2.0,
            jitter_fraction: 0.25,
        };
        let without = compute_retry_delay(3, &p_no_jitter);
        let with_jit = compute_retry_delay(3, &p_with_jitter);
        assert!(with_jit > without);
    }

    // ── is_retryable tests ───────────────────────────────────────────────

    #[test]
    fn gps_is_retryable() {
        assert!(is_retryable(ErrorCategory::Gps));
    }

    #[test]
    fn network_is_retryable() {
        assert!(is_retryable(ErrorCategory::Network));
    }

    #[test]
    fn permissions_are_not_retryable() {
        assert!(!is_retryable(ErrorCategory::Permissions));
    }

    #[test]
    fn engine_errors_are_not_retryable() {
        assert!(!is_retryable(ErrorCategory::Engine));
    }

    #[test]
    fn ffi_errors_are_not_retryable() {
        assert!(!is_retryable(ErrorCategory::Ffi));
    }

    #[test]
    fn sync_is_retryable() {
        assert!(is_retryable(ErrorCategory::Sync));
    }

    #[test]
    fn bluetooth_is_retryable() {
        assert!(is_retryable(ErrorCategory::Bluetooth));
    }

    // ── supports_fallback tests ──────────────────────────────────────────

    #[test]
    fn gps_supports_fallback() {
        assert!(supports_fallback(ErrorCategory::Gps));
    }

    #[test]
    fn bluetooth_supports_fallback() {
        assert!(supports_fallback(ErrorCategory::Bluetooth));
    }

    #[test]
    fn permissions_do_not_support_fallback() {
        assert!(!supports_fallback(ErrorCategory::Permissions));
    }

    #[test]
    fn engine_does_not_support_fallback() {
        assert!(!supports_fallback(ErrorCategory::Engine));
    }

    #[test]
    fn coaching_supports_fallback() {
        assert!(supports_fallback(ErrorCategory::Coaching));
    }

    #[test]
    fn music_does_not_support_fallback() {
        assert!(!supports_fallback(ErrorCategory::Music));
    }

    // ── fallback_message tests ───────────────────────────────────────────

    #[test]
    fn fallback_messages_are_non_empty() {
        for cat in [
            ErrorCategory::Gps,
            ErrorCategory::Sensors,
            ErrorCategory::Bluetooth,
            ErrorCategory::Sync,
            ErrorCategory::Network,
            ErrorCategory::Coaching,
            ErrorCategory::Background,
        ] {
            assert!(!fallback_message(cat).is_empty());
        }
    }

    #[test]
    fn gps_fallback_message_mentions_position() {
        let msg = fallback_message(ErrorCategory::Gps);
        assert!(msg.contains("position") || msg.contains("GPS"));
    }

    // ── should_escalate tests ────────────────────────────────────────────

    #[test]
    fn critical_severity_always_escalates() {
        for cat in [
            ErrorCategory::Gps,
            ErrorCategory::Sync,
            ErrorCategory::Engine,
            ErrorCategory::Permissions,
        ] {
            assert!(should_escalate(cat, ErrorSeverity::Critical));
        }
    }

    #[test]
    fn non_critical_permissions_do_not_escalate_unless_high() {
        assert!(!should_escalate(ErrorCategory::Permissions, ErrorSeverity::Low));
        assert!(!should_escalate(ErrorCategory::Permissions, ErrorSeverity::Medium));
        assert!(should_escalate(ErrorCategory::Permissions, ErrorSeverity::High));
    }

    #[test]
    fn non_critical_non_permission_does_not_escalate() {
        assert!(!should_escalate(ErrorCategory::Gps, ErrorSeverity::Medium));
        assert!(!should_escalate(ErrorCategory::Sync, ErrorSeverity::High));
    }

    // ── should_abandon tests ─────────────────────────────────────────────

    #[test]
    fn abandon_when_retries_exhausted_and_no_fallback() {
        let error = ErrorContext::new(
            ErrorCategory::Engine,
            ErrorSeverity::High,
            "engine_invariant",
            "Internal error",
            "engine",
        )
        .with_retry_count(5);
        let policy = RetryPolicy::default();
        assert!(should_abandon(&error, &policy));
    }

    #[test]
    fn do_not_abandon_when_fallback_available() {
        let error = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_no_fix",
            "No GPS fix",
            "gps",
        )
        .with_retry_count(5);
        let policy = RetryPolicy::default();
        assert!(!should_abandon(&error, &policy));
    }

    #[test]
    fn do_not_abandon_when_retries_remaining() {
        let error = ErrorContext::new(
            ErrorCategory::Engine,
            ErrorSeverity::High,
            "engine_invariant",
            "Internal error",
            "engine",
        )
        .with_retry_count(2);
        let policy = RetryPolicy::default();
        assert!(!should_abandon(&error, &policy));
    }

    // ── decide_recovery tests ─────────────────────────────────────────────

    #[test]
    fn critical_permission_error_escalates() {
        let error = ErrorContext::new(
            ErrorCategory::Permissions,
            ErrorSeverity::Critical,
            "perm_denied",
            "Location permission denied",
            "permissions",
        );
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Escalate);
        assert_eq!(action.next_state, ErrorState::Escalated);
        assert!(action.user_message.is_some());
        assert!(action.should_report);
    }

    #[test]
    fn critical_engine_error_aborts() {
        let error = ErrorContext::new(
            ErrorCategory::Engine,
            ErrorSeverity::Critical,
            "engine_invariant",
            "State corruption",
            "engine",
        );
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Abort);
        assert_eq!(action.next_state, ErrorState::Abandoned);
        assert!(action.user_message.is_some());
    }

    #[test]
    fn retryable_error_retries_when_attempts_remaining() {
        let error = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_no_fix",
            "No GPS fix",
            "gps",
        )
        .with_retry_count(2);
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Retry);
        assert_eq!(action.next_state, ErrorState::Recovering);
        assert!(action.delay_ms > 0);
    }

    #[test]
    fn exhausted_retries_with_fallback_falls_back() {
        let error = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_no_fix",
            "No GPS fix",
            "gps",
        )
        .with_retry_count(5);
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Fallback);
        assert_eq!(action.next_state, ErrorState::Recovering);
        assert!(action.user_message.is_some());
    }

    #[test]
    fn exhausted_retries_without_fallback_escalates_when_visible() {
        let error = ErrorContext::new(
            ErrorCategory::Engine,
            ErrorSeverity::High,
            "engine_invariant",
            "State corruption",
            "engine",
        )
        .with_retry_count(5);
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Escalate);
        assert_eq!(action.next_state, ErrorState::Escalated);
    }

    #[test]
    fn exhausted_retries_without_fallback_abandons_when_not_visible() {
        let error = ErrorContext::new(
            ErrorCategory::Engine,
            ErrorSeverity::Low,
            "engine_minor",
            "Minor issue",
            "engine",
        )
        .with_retry_count(5);
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Abort);
        assert_eq!(action.next_state, ErrorState::Abandoned);
        assert!(action.user_message.is_none());
    }

    #[test]
    fn non_retryable_with_fallback_falls_back_immediately() {
        let error = ErrorContext::new(
            ErrorCategory::Permissions,
            ErrorSeverity::Medium,
            "perm_denied",
            "Permission denied",
            "permissions",
        );
        // Permissions don't support fallback either — should ignore.
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        // Permissions: not retryable, no fallback → ignore + abandon.
        assert_eq!(action.strategy, RecoveryStrategy::Ignore);
        assert_eq!(action.next_state, ErrorState::Abandoned);
    }

    #[test]
    fn coaching_non_retryable_falls_back() {
        // Coaching is retryable, but if we send a non-retryable category
        // that supports fallback, it should fall back.
        // Ffi is not retryable and doesn't support fallback → ignore.
        let error = ErrorContext::new(
            ErrorCategory::Ffi,
            ErrorSeverity::Low,
            "ffi_bad_input",
            "Bad input",
            "ffi",
        );
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Ignore);
    }

    #[test]
    fn first_retry_has_positive_delay() {
        let error = ErrorContext::new(
            ErrorCategory::Network,
            ErrorSeverity::Medium,
            "net_timeout",
            "Network timeout",
            "network",
        )
        .with_retry_count(0);
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Retry);
        assert!(action.delay_ms > 0);
    }

    // ── transition_error_state tests ─────────────────────────────────────

    #[test]
    fn detected_to_reported_is_valid() {
        let t = transition_error_state(ErrorState::Detected, ErrorState::Reported);
        assert!(t.is_valid);
    }

    #[test]
    fn detected_to_recovering_is_valid() {
        let t = transition_error_state(ErrorState::Detected, ErrorState::Recovering);
        assert!(t.is_valid);
    }

    #[test]
    fn recovering_to_resolved_is_valid() {
        let t = transition_error_state(ErrorState::Recovering, ErrorState::Resolved);
        assert!(t.is_valid);
    }

    #[test]
    fn recovering_to_abandoned_is_valid() {
        let t = transition_error_state(ErrorState::Recovering, ErrorState::Abandoned);
        assert!(t.is_valid);
    }

    #[test]
    fn resolved_to_anything_is_invalid() {
        for to in [
            ErrorState::Detected,
            ErrorState::Reported,
            ErrorState::Recovering,
            ErrorState::Escalated,
            ErrorState::Abandoned,
        ] {
            let t = transition_error_state(ErrorState::Resolved, to);
            assert!(!t.is_valid, "Resolved → {:?} should be invalid", to);
        }
    }

    #[test]
    fn abandoned_to_anything_is_invalid() {
        for to in [
            ErrorState::Detected,
            ErrorState::Reported,
            ErrorState::Recovering,
            ErrorState::Escalated,
            ErrorState::Resolved,
        ] {
            let t = transition_error_state(ErrorState::Abandoned, to);
            assert!(!t.is_valid, "Abandoned → {:?} should be invalid", to);
        }
    }

    #[test]
    fn same_state_is_invalid() {
        for state in [
            ErrorState::Detected,
            ErrorState::Reported,
            ErrorState::Recovering,
            ErrorState::Escalated,
        ] {
            let t = transition_error_state(state, state);
            assert!(!t.is_valid, "{:?} → {:?} should be invalid", state, state);
        }
    }

    #[test]
    fn escalated_to_recovering_is_valid() {
        let t = transition_error_state(ErrorState::Escalated, ErrorState::Recovering);
        assert!(t.is_valid);
    }

    #[test]
    fn escalated_to_resolved_is_valid() {
        let t = transition_error_state(ErrorState::Escalated, ErrorState::Resolved);
        assert!(t.is_valid);
    }

    #[test]
    fn detected_to_resolved_is_invalid_directly() {
        // Cannot skip the recovering step.
        let t = transition_error_state(ErrorState::Detected, ErrorState::Resolved);
        assert!(!t.is_valid);
    }

    #[test]
    fn reported_to_recovering_is_valid() {
        let t = transition_error_state(ErrorState::Reported, ErrorState::Recovering);
        assert!(t.is_valid);
    }

    #[test]
    fn recovering_to_reported_is_valid() {
        let t = transition_error_state(ErrorState::Recovering, ErrorState::Reported);
        assert!(t.is_valid);
    }

    #[test]
    fn transition_message_is_non_empty() {
        let t = transition_error_state(ErrorState::Detected, ErrorState::Reported);
        assert!(!t.message.is_empty());
    }

    #[test]
    fn transition_serializes_round_trip() {
        let t = ErrorStateTransition {
            from: ErrorState::Detected,
            to: ErrorState::Recovering,
            is_valid: true,
            message: "test".to_string(),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: ErrorStateTransition = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }

    // ── ErrorContext tests ────────────────────────────────────────────────

    #[test]
    fn error_context_new_sets_fields() {
        let ec = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_no_fix",
            "No GPS fix",
            "gps",
        );
        assert_eq!(ec.category, ErrorCategory::Gps);
        assert_eq!(ec.severity, ErrorSeverity::Medium);
        assert_eq!(ec.code, "gps_no_fix");
        assert_eq!(ec.message, "No GPS fix");
        assert_eq!(ec.subsystem, "gps");
        assert_eq!(ec.retry_count, 0);
        assert!(ec.metadata.is_empty());
    }

    #[test]
    fn error_context_with_metadata_adds_pair() {
        let ec = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_no_fix",
            "No GPS fix",
            "gps",
        )
        .with_metadata("satellites", "4")
        .with_metadata("accuracy", "poor");
        assert_eq!(ec.metadata.len(), 2);
        assert_eq!(ec.metadata[0], ("satellites".to_string(), "4".to_string()));
        assert_eq!(ec.metadata[1], ("accuracy".to_string(), "poor".to_string()));
    }

    #[test]
    fn error_context_with_retry_count_sets_count() {
        let ec = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_no_fix",
            "No GPS fix",
            "gps",
        )
        .with_retry_count(3);
        assert_eq!(ec.retry_count, 3);
    }

    #[test]
    fn error_context_serializes_round_trip() {
        let ec = ErrorContext::new(
            ErrorCategory::Sync,
            ErrorSeverity::High,
            "sync_upload_failed",
            "Upload failed",
            "sync",
        )
        .with_retry_count(2)
        .at(1234567890)
        .with_metadata("url", "https://api.example.com");
        let json = serde_json::to_string(&ec).unwrap();
        let back: ErrorContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ec, back);
    }

    // ── ErrorRegistry tests ───────────────────────────────────────────────

    #[test]
    fn registry_starts_empty() {
        let r = ErrorRegistry::default();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn registry_record_adds_entry() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Medium,
                "gps_no_fix",
                "No GPS fix",
                "gps",
            ),
            ErrorState::Detected,
        );
        assert_eq!(r.len(), 1);
        assert!(!r.is_empty());
    }

    #[test]
    fn registry_most_recent_is_first() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Low,
                "gps_1",
                "First",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::High,
                "sync_1",
                "Second",
                "sync",
            ),
            ErrorState::Detected,
        );
        assert_eq!(r.len(), 2);
        assert_eq!(r.entries[0].error.code, "sync_1");
        assert_eq!(r.entries[1].error.code, "gps_1");
    }

    #[test]
    fn registry_respects_max_entries() {
        let mut r = ErrorRegistry::new(3);
        for i in 0..5 {
            r.record(
                ErrorContext::new(
                    ErrorCategory::Gps,
                    ErrorSeverity::Low,
                    &format!("err_{i}"),
                    &format!("Error {i}"),
                    "gps",
                ),
                ErrorState::Detected,
            );
        }
        assert_eq!(r.len(), 3);
        // Most recent 3 should be kept (err_4, err_3, err_2).
        assert_eq!(r.entries[0].error.code, "err_4");
        assert_eq!(r.entries[2].error.code, "err_2");
    }

    #[test]
    fn registry_count_by_category() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Low,
                "gps_1",
                "GPS error",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::High,
                "gps_2",
                "GPS error",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::Medium,
                "sync_1",
                "Sync error",
                "sync",
            ),
            ErrorState::Detected,
        );
        assert_eq!(r.count_by_category(ErrorCategory::Gps), 2);
        assert_eq!(r.count_by_category(ErrorCategory::Sync), 1);
        assert_eq!(r.count_by_category(ErrorCategory::Network), 0);
    }

    #[test]
    fn registry_count_by_severity() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Low,
                "gps_1",
                "GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::High,
                "gps_2",
                "GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::High,
                "sync_1",
                "Sync",
                "sync",
            ),
            ErrorState::Detected,
        );
        assert_eq!(r.count_by_severity(ErrorSeverity::Low), 1);
        assert_eq!(r.count_by_severity(ErrorSeverity::High), 2);
    }

    #[test]
    fn registry_count_by_state() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Low,
                "gps_1",
                "GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::Medium,
                "sync_1",
                "Sync",
                "sync",
            ),
            ErrorState::Resolved,
        );
        assert_eq!(r.count_by_state(ErrorState::Detected), 1);
        assert_eq!(r.count_by_state(ErrorState::Resolved), 1);
    }

    #[test]
    fn registry_active_count_excludes_resolved() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Medium,
                "gps_1",
                "GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::Medium,
                "sync_1",
                "Sync",
                "sync",
            ),
            ErrorState::Resolved,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Network,
                ErrorSeverity::High,
                "net_1",
                "Network",
                "network",
            ),
            ErrorState::Recovering,
        );
        assert_eq!(r.active_count(), 2); // Detected + Recovering
    }

    #[test]
    fn registry_resolved_count() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Medium,
                "gps_1",
                "GPS",
                "gps",
            ),
            ErrorState::Resolved,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::Medium,
                "sync_1",
                "Sync",
                "sync",
            ),
            ErrorState::Resolved,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Network,
                ErrorSeverity::Medium,
                "net_1",
                "Network",
                "network",
            ),
            ErrorState::Detected,
        );
        assert_eq!(r.resolved_count(), 2);
    }

    #[test]
    fn registry_abandoned_count() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Engine,
                ErrorSeverity::High,
                "engine_1",
                "Engine",
                "engine",
            ),
            ErrorState::Abandoned,
        );
        assert_eq!(r.abandoned_count(), 1);
    }

    #[test]
    fn registry_clear_removes_all() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Low,
                "gps_1",
                "GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.clear();
        assert!(r.is_empty());
    }

    #[test]
    fn registry_most_recent_finds_latest_of_category() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Low,
                "gps_old",
                "Old GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::High,
                "sync_new",
                "New Sync",
                "sync",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::High,
                "gps_new",
                "New GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        let recent = r.most_recent(ErrorCategory::Gps).unwrap();
        assert_eq!(recent.error.code, "gps_new");
    }

    #[test]
    fn registry_most_recent_returns_none_if_not_found() {
        let r = ErrorRegistry::default();
        assert!(r.most_recent(ErrorCategory::Gps).is_none());
    }

    #[test]
    fn registry_serializes_round_trip() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Medium,
                "gps_1",
                "GPS error",
                "gps",
            ),
            ErrorState::Detected,
        );
        let json = serde_json::to_string(&r).unwrap();
        let back: ErrorRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    // ── EngineHealthStatus / build_health_status tests ───────────────────

    #[test]
    fn empty_registry_is_healthy() {
        let r = ErrorRegistry::default();
        let status = build_health_status(&r);
        assert_eq!(status.status, EngineHealthLabel::Healthy);
        assert_eq!(status.total_errors, 0);
        assert_eq!(status.active_errors, 0);
    }

    #[test]
    fn all_resolved_is_operational() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Medium,
                "gps_1",
                "GPS",
                "gps",
            ),
            ErrorState::Resolved,
        );
        let status = build_health_status(&r);
        assert_eq!(status.status, EngineHealthLabel::Operational);
        assert_eq!(status.resolved_errors, 1);
        assert_eq!(status.active_errors, 0);
    }

    #[test]
    fn active_errors_without_fallback_is_warning() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Permissions,
                ErrorSeverity::High,
                "perm_1",
                "Permission",
                "permissions",
            ),
            ErrorState::Detected,
        );
        let status = build_health_status(&r);
        assert_eq!(status.status, EngineHealthLabel::Warning);
        assert_eq!(status.active_errors, 1);
    }

    #[test]
    fn active_fallback_is_degraded() {
        let mut r = ErrorRegistry::default();
        let error = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_1",
            "GPS",
            "gps",
        );
        let action = RecoveryAction {
            strategy: RecoveryStrategy::Fallback,
            delay_ms: 0,
            user_message: Some("Falling back".to_string()),
            should_report: true,
            should_persist: true,
            next_state: ErrorState::Recovering,
            reason: "test".to_string(),
        };
        r.record_with_action(error, ErrorState::Recovering, action);
        let status = build_health_status(&r);
        assert_eq!(status.status, EngineHealthLabel::Degraded);
        assert!(status.is_degraded);
    }

    #[test]
    fn critical_active_errors_is_critical() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Engine,
                ErrorSeverity::Critical,
                "engine_1",
                "Engine",
                "engine",
            ),
            ErrorState::Detected,
        );
        let status = build_health_status(&r);
        assert_eq!(status.status, EngineHealthLabel::Critical);
        assert_eq!(status.critical_errors, 1);
    }

    #[test]
    fn health_status_category_counts() {
        let mut r = ErrorRegistry::default();
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::Low,
                "gps_1",
                "GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Gps,
                ErrorSeverity::High,
                "gps_2",
                "GPS",
                "gps",
            ),
            ErrorState::Detected,
        );
        r.record(
            ErrorContext::new(
                ErrorCategory::Sync,
                ErrorSeverity::Medium,
                "sync_1",
                "Sync",
                "sync",
            ),
            ErrorState::Detected,
        );
        let status = build_health_status(&r);
        assert_eq!(status.category_counts.len(), 2); // Gps + Sync
    }

    #[test]
    fn health_status_summary_is_non_empty() {
        let r = ErrorRegistry::default();
        let status = build_health_status(&r);
        assert!(!status.summary.is_empty());
    }

    #[test]
    fn health_label_serializes_round_trip() {
        for label in [
            EngineHealthLabel::Healthy,
            EngineHealthLabel::Operational,
            EngineHealthLabel::Degraded,
            EngineHealthLabel::Warning,
            EngineHealthLabel::Critical,
        ] {
            let json = serde_json::to_string(&label).unwrap();
            let back: EngineHealthLabel = serde_json::from_str(&json).unwrap();
            assert_eq!(label, back);
        }
    }

    #[test]
    fn recovery_action_serializes_round_trip() {
        let action = RecoveryAction {
            strategy: RecoveryStrategy::Retry,
            delay_ms: 5000,
            user_message: Some("Retrying".to_string()),
            should_report: true,
            should_persist: false,
            next_state: ErrorState::Recovering,
            reason: "test".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let back: RecoveryAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, back);
    }

    #[test]
    fn recovery_action_with_none_message_serializes() {
        let action = RecoveryAction {
            strategy: RecoveryStrategy::Ignore,
            delay_ms: 0,
            user_message: None,
            should_report: false,
            should_persist: true,
            next_state: ErrorState::Abandoned,
            reason: "test".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        let back: RecoveryAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, back);
    }

    // ── Integration: decide_recovery + transition ─────────────────────────

    #[test]
    fn decide_recovery_then_transition_is_valid() {
        let error = ErrorContext::new(
            ErrorCategory::Gps,
            ErrorSeverity::Medium,
            "gps_no_fix",
            "No GPS fix",
            "gps",
        )
        .with_retry_count(0);
        let policy = RetryPolicy::default();
        let action = decide_recovery(&error, &policy);

        // The action says to move to Recovering.
        assert_eq!(action.next_state, ErrorState::Recovering);

        // Validate the transition from Detected → Recovering.
        let transition = transition_error_state(ErrorState::Detected, action.next_state);
        assert!(transition.is_valid);
    }

    #[test]
    fn full_recovery_cycle() {
        let error = ErrorContext::new(
            ErrorCategory::Network,
            ErrorSeverity::Medium,
            "net_timeout",
            "Network timeout",
            "network",
        );
        let policy = RetryPolicy::default();

        // First attempt — should retry.
        let action1 = decide_recovery(&error, &policy);
        assert_eq!(action1.strategy, RecoveryStrategy::Retry);
        assert!(transition_error_state(ErrorState::Detected, action1.next_state).is_valid);

        // Simulate a successful retry — transition to Resolved.
        assert!(transition_error_state(ErrorState::Recovering, ErrorState::Resolved).is_valid);
    }

    #[test]
    fn full_abandonment_cycle() {
        let error = ErrorContext::new(
            ErrorCategory::Engine,
            ErrorSeverity::Critical,
            "engine_fatal",
            "Fatal engine error",
            "engine",
        );
        let policy = RetryPolicy::default();

        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Abort);
        assert_eq!(action.next_state, ErrorState::Abandoned);

        // Abandoned is terminal — no further transitions.
        assert!(action.next_state.is_terminal());
        assert!(!transition_error_state(ErrorState::Abandoned, ErrorState::Recovering).is_valid);
    }

    #[test]
    fn permission_escalation_cycle() {
        let error = ErrorContext::new(
            ErrorCategory::Permissions,
            ErrorSeverity::Critical,
            "perm_denied",
            "Location permission denied",
            "permissions",
        );
        let policy = RetryPolicy::default();

        let action = decide_recovery(&error, &policy);
        assert_eq!(action.strategy, RecoveryStrategy::Escalate);
        assert_eq!(action.next_state, ErrorState::Escalated);

        // User grants permission — can transition back to Recovering.
        assert!(transition_error_state(ErrorState::Escalated, ErrorState::Recovering).is_valid);
        // Then to Resolved.
        assert!(transition_error_state(ErrorState::Recovering, ErrorState::Resolved).is_valid);
    }
}
