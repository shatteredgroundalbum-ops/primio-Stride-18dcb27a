//! Monitoring and operations (spec section 16).
//!
//! Structured event logging, crash reporting, performance tracing,
//! sync-failure metrics, AI cost/error tracking, Cloud Function
//! alerts, Firestore/Storage usage alerts, billing-budget alerts,
//! uptime monitoring, and release-health dashboard aggregation.
//!
//! This module defines the *structure* of monitoring data — what
//! events are logged, what severity they carry, what thresholds
//! trigger alerts, and how metrics aggregate into a dashboard. The
//! actual side effects (sending to Crashlytics, writing to
//! Cloud Logging, calling billing APIs) happen on the server/Dart
//! side. This module is purely data definitions and decision logic,
//! and is side-effect free.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Log level
// ---------------------------------------------------------------------------

/// The severity level of a structured log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// Debug — verbose, dev-only, never in production.
    Debug,
    /// Info — routine operation (user signed in, workout started).
    Info,
    /// Warning — degraded but functional (weak GPS, slow sync).
    Warning,
    /// Error — a feature failed but the app continues (sync failed,
    /// AI unavailable).
    Error,
    /// Critical — the app or a core subsystem is unusable (crash,
    /// data corruption, no GPS at all).
    Critical,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl LogLevel {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }

    /// Returns `true` if this level should be sent to Crashlytics.
    pub fn reports_to_crashlytics(self) -> bool {
        matches!(self, LogLevel::Error | LogLevel::Critical)
    }

    /// Returns `true` if this level is production-safe (Info and
    /// above; Debug is stripped in release builds).
    pub fn is_production_safe(self) -> bool {
        matches!(
            self,
            LogLevel::Info | LogLevel::Warning | LogLevel::Error | LogLevel::Critical
        )
    }

    /// Numeric priority for sorting / filtering (higher = more
    /// severe).
    pub fn priority(self) -> u8 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warning => 2,
            LogLevel::Error => 3,
            LogLevel::Critical => 4,
        }
    }
}

// ---------------------------------------------------------------------------
// Structured log entry
// ---------------------------------------------------------------------------

/// A structured log entry with a timestamp, level, category, message,
/// and optional key-value context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    /// UTC timestamp (epoch milliseconds).
    pub timestamp_ms: i64,
    /// The log level.
    pub level: LogLevel,
    /// The monitoring category this log belongs to.
    pub category: MonitoringCategory,
    /// A short, human-readable message.
    pub message: String,
    /// Optional structured context (key-value pairs).
    pub context: Vec<KeyValuePair>,
    /// Optional session/workout ID for correlation.
    pub session_id: Option<String>,
    /// Optional user ID (hashed / anonymized in production).
    pub user_id: Option<String>,
}

impl LogEntry {
    /// Creates a new log entry at the given timestamp.
    pub fn new(
        timestamp_ms: i64,
        level: LogLevel,
        category: MonitoringCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp_ms,
            level,
            category,
            message: message.into(),
            context: Vec::new(),
            session_id: None,
            user_id: None,
        }
    }

    /// Adds a context key-value pair and returns self for chaining.
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push(KeyValuePair {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Sets the session ID and returns self for chaining.
    pub fn with_session(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Sets the user ID and returns self for chaining.
    pub fn with_user(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Returns `true` if this log entry should be reported to
    /// Crashlytics.
    pub fn should_report_to_crashlytics(&self) -> bool {
        self.level.reports_to_crashlytics()
    }
}

/// A key-value pair for structured log context.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

// ---------------------------------------------------------------------------
// Monitoring category
// ---------------------------------------------------------------------------

/// The functional category of a monitoring event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitoringCategory {
    /// Workout engine events (start, pause, GPS, distance).
    WorkoutEngine,
    /// GPS quality / accuracy events.
    Gps,
    /// Sync engine events (upload, download, conflict, failure).
    Sync,
    /// AI coaching events (call, cost, error, fallback).
    Ai,
    /// Authentication events (sign-in, sign-out, reauth).
    Auth,
    /// Database events (SQLite, migration, schema).
    Database,
    /// Cloud Storage events (route upload, download, retention).
    Storage,
    /// Cloud Functions events (Cloud Run, function invocations).
    CloudFunction,
    /// Firestore events (reads, writes, quota).
    Firestore,
    /// Wearable events (Health Connect, watch disconnect).
    Wearable,
    /// Music events (playback, network loss, audio focus).
    Music,
    /// Background execution events (foreground service, kill, restart).
    Background,
    /// Notification events (delivery, quiet hours).
    Notification,
    /// Error recovery events (recovery, retry, state transition).
    ErrorRecovery,
    /// Crash events (uncaught exceptions, panics).
    Crash,
    /// Performance events (latency, throughput).
    Performance,
    /// Billing / cost events (budget, spend).
    Billing,
}

impl MonitoringCategory {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            MonitoringCategory::WorkoutEngine => "workout_engine",
            MonitoringCategory::Gps => "gps",
            MonitoringCategory::Sync => "sync",
            MonitoringCategory::Ai => "ai",
            MonitoringCategory::Auth => "auth",
            MonitoringCategory::Database => "database",
            MonitoringCategory::Storage => "storage",
            MonitoringCategory::CloudFunction => "cloud_function",
            MonitoringCategory::Firestore => "firestore",
            MonitoringCategory::Wearable => "wearable",
            MonitoringCategory::Music => "music",
            MonitoringCategory::Background => "background",
            MonitoringCategory::Notification => "notification",
            MonitoringCategory::ErrorRecovery => "error_recovery",
            MonitoringCategory::Crash => "crash",
            MonitoringCategory::Performance => "performance",
            MonitoringCategory::Billing => "billing",
        }
    }
}

// ---------------------------------------------------------------------------
// Crash report
// ---------------------------------------------------------------------------

/// The severity of a crash for Crashlytics reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrashSeverity {
    /// A caught exception that the app recovered from.
    NonFatal,
    /// An uncaught exception that crashed the app.
    Fatal,
}

impl CrashSeverity {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            CrashSeverity::NonFatal => "non_fatal",
            CrashSeverity::Fatal => "fatal",
        }
    }

    /// Returns `true` if this crash is fatal.
    pub fn is_fatal(self) -> bool {
        matches!(self, CrashSeverity::Fatal)
    }
}

/// A crash report for Crashlytics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrashReport {
    /// UTC timestamp (epoch milliseconds).
    pub timestamp_ms: i64,
    /// Crash severity (fatal or non-fatal).
    pub severity: CrashSeverity,
    /// The exception type (e.g., "NullPointerException").
    pub exception_type: String,
    /// The exception message.
    pub message: String,
    /// Stack trace (optional, may be truncated for large traces).
    pub stack_trace: Option<String>,
    /// Breadcrumbs leading up to the crash.
    pub breadcrumbs: Vec<LogEntry>,
    /// App version at the time of the crash.
    pub app_version: String,
    /// Device model (e.g., "Pixel 8").
    pub device_model: String,
    /// OS version (e.g., "Android 14").
    pub os_version: String,
    /// Whether the crash happened during an active workout.
    pub during_workout: bool,
    /// Optional workout session ID.
    pub session_id: Option<String>,
}

impl CrashReport {
    /// Creates a new crash report.
    pub fn new(
        timestamp_ms: i64,
        severity: CrashSeverity,
        exception_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp_ms,
            severity,
            exception_type: exception_type.into(),
            message: message.into(),
            stack_trace: None,
            breadcrumbs: Vec::new(),
            app_version: String::new(),
            device_model: String::new(),
            os_version: String::new(),
            during_workout: false,
            session_id: None,
        }
    }

    /// Sets the stack trace and returns self for chaining.
    pub fn with_stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    /// Adds a breadcrumb and returns self for chaining.
    pub fn add_breadcrumb(&mut self, entry: LogEntry) {
        self.breadcrumbs.push(entry);
    }

    /// Sets the app version and returns self for chaining.
    pub fn with_app_version(mut self, version: impl Into<String>) -> Self {
        self.app_version = version.into();
        self
    }

    /// Sets the device model and returns self for chaining.
    pub fn with_device_model(mut self, model: impl Into<String>) -> Self {
        self.device_model = model.into();
        self
    }

    /// Sets the OS version and returns self for chaining.
    pub fn with_os_version(mut self, version: impl Into<String>) -> Self {
        self.os_version = version.into();
        self
    }

    /// Sets whether the crash happened during a workout.
    pub fn with_during_workout(mut self, during: bool) -> Self {
        self.during_workout = during;
        self
    }

    /// Sets the session ID and returns self for chaining.
    pub fn with_session(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Returns `true` if this crash should be escalated (fatal or
    /// during an active workout).
    pub fn should_escalate(&self) -> bool {
        self.severity.is_fatal() || self.during_workout
    }
}

// ---------------------------------------------------------------------------
// Performance trace
// ---------------------------------------------------------------------------

/// A performance trace for Firebase Performance Monitoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceTrace {
    /// The name of the trace (e.g., "workout_start",
    /// "sync_upload_route").
    pub name: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// UTC start timestamp (epoch milliseconds).
    pub started_at_ms: i64,
    /// Whether the trace was successful.
    pub success: bool,
    /// Optional error message if the trace failed.
    pub error_message: Option<String>,
    /// Custom metrics (key-value numeric pairs).
    pub metrics: Vec<MetricPair>,
}

impl PerformanceTrace {
    /// Creates a new performance trace.
    pub fn new(name: impl Into<String>, started_at_ms: i64, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            duration_ms,
            started_at_ms,
            success: true,
            error_message: None,
            metrics: Vec::new(),
        }
    }

    /// Marks the trace as failed with an error message.
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error_message = Some(error.into());
        self
    }

    /// Adds a custom metric and returns self for chaining.
    pub fn with_metric(mut self, name: impl Into<String>, value: u64) -> Self {
        self.metrics.push(MetricPair {
            name: name.into(),
            value,
        });
        self
    }

    /// Returns `true` if this trace is slow (duration exceeds the
    /// given threshold in milliseconds).
    pub fn is_slow(&self, threshold_ms: u64) -> bool {
        self.duration_ms > threshold_ms
    }
}

/// A named numeric metric for performance traces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricPair {
    pub name: String,
    pub value: u64,
}

// ---------------------------------------------------------------------------
// Sync failure metrics
// ---------------------------------------------------------------------------

/// Aggregated sync failure metrics over a time window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncFailureMetrics {
    /// The time window in minutes these metrics cover.
    pub window_minutes: u32,
    /// Total number of sync attempts.
    pub total_attempts: u32,
    /// Number of successful syncs.
    pub successful: u32,
    /// Number of failed syncs.
    pub failed: u32,
    /// Number of retries triggered.
    pub retries: u32,
    /// Number of conflicts detected.
    pub conflicts: u32,
    /// Number of duplicate-prevention skips.
    pub duplicates_prevented: u32,
    /// Number of tombstone deletions processed.
    pub tombstones: u32,
}

impl SyncFailureMetrics {
    /// Creates a new sync failure metrics snapshot.
    pub fn new(window_minutes: u32) -> Self {
        Self {
            window_minutes,
            total_attempts: 0,
            successful: 0,
            failed: 0,
            retries: 0,
            conflicts: 0,
            duplicates_prevented: 0,
            tombstones: 0,
        }
    }

    /// Records a successful sync.
    pub fn record_success(&mut self) {
        self.total_attempts += 1;
        self.successful += 1;
    }

    /// Records a failed sync.
    pub fn record_failure(&mut self) {
        self.total_attempts += 1;
        self.failed += 1;
    }

    /// Records a retry.
    pub fn record_retry(&mut self) {
        self.retries += 1;
    }

    /// Records a conflict.
    pub fn record_conflict(&mut self) {
        self.conflicts += 1;
    }

    /// Records a duplicate prevention.
    pub fn record_duplicate(&mut self) {
        self.duplicates_prevented += 1;
    }

    /// Records a tombstone.
    pub fn record_tombstone(&mut self) {
        self.tombstones += 1;
    }

    /// Returns the failure rate as a fraction (0.0 to 1.0).
    pub fn failure_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            return 0.0;
        }
        self.failed as f64 / self.total_attempts as f64
    }

    /// Returns the success rate as a fraction (0.0 to 1.0).
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            return 0.0;
        }
        self.successful as f64 / self.total_attempts as f64
    }

    /// Returns `true` if the failure rate exceeds the given
    /// threshold (0.0 to 1.0).
    pub fn is_failing(&self, threshold: f64) -> bool {
        self.failure_rate() > threshold
    }
}

// ---------------------------------------------------------------------------
// AI cost monitoring
// ---------------------------------------------------------------------------

/// AI operation type for cost tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiOperationType {
    /// Generate initial training plan.
    GeneratePlan,
    /// Adjust an existing plan.
    AdjustPlan,
    /// Summarize a workout.
    SummarizeWorkout,
    /// Generate a coaching message.
    CoachingMessage,
    /// Interpret user feedback.
    InterpretFeedback,
    /// Recommend progression.
    RecommendProgression,
    /// Generate encouragement.
    GenerateEncouragement,
    /// Moderate an unsafe request.
    ModerateRequest,
}

impl AiOperationType {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            AiOperationType::GeneratePlan => "generate_plan",
            AiOperationType::AdjustPlan => "adjust_plan",
            AiOperationType::SummarizeWorkout => "summarize_workout",
            AiOperationType::CoachingMessage => "coaching_message",
            AiOperationType::InterpretFeedback => "interpret_feedback",
            AiOperationType::RecommendProgression => "recommend_progression",
            AiOperationType::GenerateEncouragement => "generate_encouragement",
            AiOperationType::ModerateRequest => "moderate_request",
        }
    }
}

/// AI cost metrics for a single operation or aggregated window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiCostMetrics {
    /// The time window in minutes these metrics cover.
    pub window_minutes: u32,
    /// Total number of AI calls.
    pub total_calls: u32,
    /// Number of successful AI calls.
    pub successful: u32,
    /// Number of failed AI calls.
    pub failed: u32,
    /// Number of cache hits (no AI call needed).
    pub cache_hits: u32,
    /// Total cost in cents (USD * 100).
    pub total_cost_cents: u64,
    /// Total input tokens consumed.
    pub total_input_tokens: u64,
    /// Total output tokens consumed.
    pub total_output_tokens: u64,
    /// Number of fallbacks to rule-based plan.
    pub fallbacks: u32,
    /// Per-operation-type call counts.
    pub operation_counts: Vec<AiOperationCount>,
}

impl AiCostMetrics {
    /// Creates a new AI cost metrics snapshot.
    pub fn new(window_minutes: u32) -> Self {
        Self {
            window_minutes,
            total_calls: 0,
            successful: 0,
            failed: 0,
            cache_hits: 0,
            total_cost_cents: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            fallbacks: 0,
            operation_counts: Vec::new(),
        }
    }

    /// Records a successful AI call.
    pub fn record_success(&mut self, cost_cents: u64, input_tokens: u64, output_tokens: u64) {
        self.total_calls += 1;
        self.successful += 1;
        self.total_cost_cents += cost_cents;
        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
    }

    /// Records a failed AI call.
    pub fn record_failure(&mut self) {
        self.total_calls += 1;
        self.failed += 1;
    }

    /// Records a cache hit.
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Records a fallback to rule-based.
    pub fn record_fallback(&mut self) {
        self.fallbacks += 1;
    }

    /// Records an operation of a given type.
    pub fn record_operation(&mut self, op_type: AiOperationType) {
        for count in &mut self.operation_counts {
            if count.operation_type == op_type {
                count.count += 1;
                return;
            }
        }
        self.operation_counts.push(AiOperationCount {
            operation_type: op_type,
            count: 1,
        });
    }

    /// Returns the error rate as a fraction (0.0 to 1.0).
    pub fn error_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        self.failed as f64 / self.total_calls as f64
    }

    /// Returns the cache hit rate as a fraction (0.0 to 1.0).
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.total_calls + self.cache_hits;
        if total == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total as f64
    }

    /// Returns the average cost per call in cents.
    pub fn avg_cost_cents(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        self.total_cost_cents as f64 / self.total_calls as f64
    }

    /// Returns the total cost in USD.
    pub fn total_cost_usd(&self) -> f64 {
        self.total_cost_cents as f64 / 100.0
    }

    /// Returns `true` if the AI error rate exceeds the given
    /// threshold (0.0 to 1.0).
    pub fn is_error_rate_high(&self, threshold: f64) -> bool {
        self.error_rate() > threshold
    }
}

/// A per-operation-type call count for AI metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiOperationCount {
    pub operation_type: AiOperationType,
    pub count: u32,
}

// ---------------------------------------------------------------------------
// Alert
// ---------------------------------------------------------------------------

/// The type of alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    /// Cloud Function error rate exceeded threshold.
    CloudFunctionErrorRate,
    /// Cloud Function latency exceeded threshold.
    CloudFunctionLatency,
    /// Firestore read quota exceeded.
    FirestoreReadQuota,
    /// Firestore write quota exceeded.
    FirestoreWriteQuota,
    /// Firestore delete quota exceeded.
    FirestoreDeleteQuota,
    /// Cloud Storage usage exceeded threshold.
    StorageUsage,
    /// Cloud Storage bandwidth exceeded threshold.
    StorageBandwidth,
    /// Billing budget exceeded threshold.
    BillingBudget,
    /// AI cost exceeded threshold.
    AiCost,
    /// AI error rate exceeded threshold.
    AiErrorRate,
    /// Sync failure rate exceeded threshold.
    SyncFailureRate,
    /// Service uptime dropped below threshold.
    UptimeDrop,
    /// Crash rate exceeded threshold.
    CrashRate,
}

impl AlertType {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            AlertType::CloudFunctionErrorRate => "cloud_function_error_rate",
            AlertType::CloudFunctionLatency => "cloud_function_latency",
            AlertType::FirestoreReadQuota => "firestore_read_quota",
            AlertType::FirestoreWriteQuota => "firestore_write_quota",
            AlertType::FirestoreDeleteQuota => "firestore_delete_quota",
            AlertType::StorageUsage => "storage_usage",
            AlertType::StorageBandwidth => "storage_bandwidth",
            AlertType::BillingBudget => "billing_budget",
            AlertType::AiCost => "ai_cost",
            AlertType::AiErrorRate => "ai_error_rate",
            AlertType::SyncFailureRate => "sync_failure_rate",
            AlertType::UptimeDrop => "uptime_drop",
            AlertType::CrashRate => "crash_rate",
        }
    }
}

/// The severity of an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    /// Informational — no action needed.
    Info,
    /// Warning — investigate soon.
    Warning,
    /// High — action required.
    High,
    /// Critical — immediate action required.
    Critical,
}

impl Default for AlertSeverity {
    fn default() -> Self {
        AlertSeverity::Warning
    }
}

impl AlertSeverity {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::High => "high",
            AlertSeverity::Critical => "critical",
        }
    }

    /// Returns `true` if this alert requires immediate paging.
    pub fn requires_paging(self) -> bool {
        matches!(self, AlertSeverity::Critical)
    }
}

/// An alert triggered by a monitoring threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    /// UTC timestamp (epoch milliseconds) when the alert fired.
    pub timestamp_ms: i64,
    /// The alert type.
    pub alert_type: AlertType,
    /// The alert severity.
    pub severity: AlertSeverity,
    /// A human-readable alert title.
    pub title: String,
    /// A detailed description of the alert.
    pub description: String,
    /// The current value that triggered the alert.
    pub current_value: f64,
    /// The threshold that was exceeded.
    pub threshold: f64,
    /// The unit of the value (e.g., "percent", "ms", "cents").
    pub unit: String,
}

impl Alert {
    /// Creates a new alert.
    pub fn new(
        timestamp_ms: i64,
        alert_type: AlertType,
        severity: AlertSeverity,
        title: impl Into<String>,
        description: impl Into<String>,
        current_value: f64,
        threshold: f64,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            timestamp_ms,
            alert_type,
            severity,
            title: title.into(),
            description: description.into(),
            current_value,
            threshold,
            unit: unit.into(),
        }
    }

    /// Returns `true` if this alert requires paging.
    pub fn requires_paging(&self) -> bool {
        self.severity.requires_paging()
    }
}

// ---------------------------------------------------------------------------
// Alert threshold configuration
// ---------------------------------------------------------------------------

/// Threshold configuration for monitoring alerts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Cloud Function error rate threshold (fraction, 0.0–1.0).
    pub cloud_function_error_rate: f64,
    /// Cloud Function latency threshold (milliseconds).
    pub cloud_function_latency_ms: u64,
    /// Firestore reads per day threshold.
    pub firestore_reads_per_day: u64,
    /// Firestore writes per day threshold.
    pub firestore_writes_per_day: u64,
    /// Firestore deletes per day threshold.
    pub firestore_deletes_per_day: u64,
    /// Cloud Storage usage threshold (bytes).
    pub storage_usage_bytes: u64,
    /// Cloud Storage bandwidth threshold (bytes per day).
    pub storage_bandwidth_bytes: u64,
    /// Billing budget threshold (cents, USD * 100).
    pub billing_budget_cents: u64,
    /// AI cost per day threshold (cents).
    pub ai_cost_per_day_cents: u64,
    /// AI error rate threshold (fraction, 0.0–1.0).
    pub ai_error_rate: f64,
    /// Sync failure rate threshold (fraction, 0.0–1.0).
    pub sync_failure_rate: f64,
    /// Uptime threshold (fraction, 0.0–1.0).
    pub uptime: f64,
    /// Crash rate threshold (crashes per 1000 sessions).
    pub crash_rate_per_1000: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cloud_function_error_rate: 0.05,
            cloud_function_latency_ms: 5_000,
            firestore_reads_per_day: 50_000,
            firestore_writes_per_day: 10_000,
            firestore_deletes_per_day: 5_000,
            storage_usage_bytes: 5_000_000_000,       // 5 GB
            storage_bandwidth_bytes: 10_000_000_000,  // 10 GB/day
            billing_budget_cents: 5_000,              // $50.00
            ai_cost_per_day_cents: 500,              // $5.00/day
            ai_error_rate: 0.10,
            sync_failure_rate: 0.20,
            uptime: 0.99,
            crash_rate_per_1000: 10.0,
        }
    }
}

impl AlertThresholds {
    /// Checks Cloud Function error rate and returns an alert if the
    /// threshold is exceeded.
    pub fn check_cloud_function_error_rate(
        &self,
        timestamp_ms: i64,
        error_rate: f64,
    ) -> Option<Alert> {
        if error_rate > self.cloud_function_error_rate {
            let severity = if error_rate > self.cloud_function_error_rate * 3.0 {
                AlertSeverity::Critical
            } else if error_rate > self.cloud_function_error_rate * 2.0 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::CloudFunctionErrorRate,
                severity,
                "Cloud Function error rate exceeded",
                format!(
                    "Cloud Function error rate is {:.1}%, threshold is {:.1}%",
                    error_rate * 100.0,
                    self.cloud_function_error_rate * 100.0,
                ),
                error_rate,
                self.cloud_function_error_rate,
                "fraction",
            ))
        } else {
            None
        }
    }

    /// Checks Cloud Function latency and returns an alert if the
    /// threshold is exceeded.
    pub fn check_cloud_function_latency(
        &self,
        timestamp_ms: i64,
        latency_ms: u64,
    ) -> Option<Alert> {
        if latency_ms > self.cloud_function_latency_ms {
            let severity = if latency_ms > self.cloud_function_latency_ms * 3 {
                AlertSeverity::Critical
            } else if latency_ms > self.cloud_function_latency_ms * 2 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::CloudFunctionLatency,
                severity,
                "Cloud Function latency exceeded",
                format!(
                    "Cloud Function latency is {} ms, threshold is {} ms",
                    latency_ms, self.cloud_function_latency_ms,
                ),
                latency_ms as f64,
                self.cloud_function_latency_ms as f64,
                "ms",
            ))
        } else {
            None
        }
    }

    /// Checks Firestore reads and returns an alert if the threshold
    /// is exceeded.
    pub fn check_firestore_reads(
        &self,
        timestamp_ms: i64,
        reads_per_day: u64,
    ) -> Option<Alert> {
        if reads_per_day > self.firestore_reads_per_day {
            Some(Alert::new(
                timestamp_ms,
                AlertType::FirestoreReadQuota,
                AlertSeverity::Warning,
                "Firestore read quota exceeded",
                format!(
                    "Firestore reads are {}/day, threshold is {}/day",
                    reads_per_day, self.firestore_reads_per_day,
                ),
                reads_per_day as f64,
                self.firestore_reads_per_day as f64,
                "reads/day",
            ))
        } else {
            None
        }
    }

    /// Checks Firestore writes and returns an alert if the threshold
    /// is exceeded.
    pub fn check_firestore_writes(
        &self,
        timestamp_ms: i64,
        writes_per_day: u64,
    ) -> Option<Alert> {
        if writes_per_day > self.firestore_writes_per_day {
            Some(Alert::new(
                timestamp_ms,
                AlertType::FirestoreWriteQuota,
                AlertSeverity::Warning,
                "Firestore write quota exceeded",
                format!(
                    "Firestore writes are {}/day, threshold is {}/day",
                    writes_per_day, self.firestore_writes_per_day,
                ),
                writes_per_day as f64,
                self.firestore_writes_per_day as f64,
                "writes/day",
            ))
        } else {
            None
        }
    }

    /// Checks Cloud Storage usage and returns an alert if the
    /// threshold is exceeded.
    pub fn check_storage_usage(
        &self,
        timestamp_ms: i64,
        usage_bytes: u64,
    ) -> Option<Alert> {
        if usage_bytes > self.storage_usage_bytes {
            let severity = if usage_bytes > self.storage_usage_bytes * 2 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::StorageUsage,
                severity,
                "Cloud Storage usage exceeded",
                format!(
                    "Storage usage is {} bytes, threshold is {} bytes",
                    usage_bytes, self.storage_usage_bytes,
                ),
                usage_bytes as f64,
                self.storage_usage_bytes as f64,
                "bytes",
            ))
        } else {
            None
        }
    }

    /// Checks AI cost and returns an alert if the threshold is
    /// exceeded.
    pub fn check_ai_cost(
        &self,
        timestamp_ms: i64,
        cost_per_day_cents: u64,
    ) -> Option<Alert> {
        if cost_per_day_cents > self.ai_cost_per_day_cents {
            let severity = if cost_per_day_cents > self.ai_cost_per_day_cents * 2 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::AiCost,
                severity,
                "AI cost exceeded",
                format!(
                    "AI cost is {} cents/day, threshold is {} cents/day",
                    cost_per_day_cents, self.ai_cost_per_day_cents,
                ),
                cost_per_day_cents as f64,
                self.ai_cost_per_day_cents as f64,
                "cents/day",
            ))
        } else {
            None
        }
    }

    /// Checks AI error rate and returns an alert if the threshold is
    /// exceeded.
    pub fn check_ai_error_rate(
        &self,
        timestamp_ms: i64,
        error_rate: f64,
    ) -> Option<Alert> {
        if error_rate > self.ai_error_rate {
            let severity = if error_rate > self.ai_error_rate * 3.0 {
                AlertSeverity::Critical
            } else if error_rate > self.ai_error_rate * 2.0 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::AiErrorRate,
                severity,
                "AI error rate exceeded",
                format!(
                    "AI error rate is {:.1}%, threshold is {:.1}%",
                    error_rate * 100.0,
                    self.ai_error_rate * 100.0,
                ),
                error_rate,
                self.ai_error_rate,
                "fraction",
            ))
        } else {
            None
        }
    }

    /// Checks sync failure rate and returns an alert if the threshold
    /// is exceeded.
    pub fn check_sync_failure_rate(
        &self,
        timestamp_ms: i64,
        failure_rate: f64,
    ) -> Option<Alert> {
        if failure_rate > self.sync_failure_rate {
            let severity = if failure_rate > self.sync_failure_rate * 3.0 {
                AlertSeverity::Critical
            } else if failure_rate > self.sync_failure_rate * 2.0 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::SyncFailureRate,
                severity,
                "Sync failure rate exceeded",
                format!(
                    "Sync failure rate is {:.1}%, threshold is {:.1}%",
                    failure_rate * 100.0,
                    self.sync_failure_rate * 100.0,
                ),
                failure_rate,
                self.sync_failure_rate,
                "fraction",
            ))
        } else {
            None
        }
    }

    /// Checks uptime and returns an alert if it drops below the
    /// threshold.
    pub fn check_uptime(
        &self,
        timestamp_ms: i64,
        uptime: f64,
    ) -> Option<Alert> {
        if uptime < self.uptime {
            let severity = if uptime < self.uptime - 0.05 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::High
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::UptimeDrop,
                severity,
                "Uptime dropped below threshold",
                format!(
                    "Uptime is {:.2}%, threshold is {:.2}%",
                    uptime * 100.0,
                    self.uptime * 100.0,
                ),
                uptime,
                self.uptime,
                "fraction",
            ))
        } else {
            None
        }
    }

    /// Checks billing budget and returns an alert if the threshold is
    /// exceeded.
    pub fn check_billing_budget(
        &self,
        timestamp_ms: i64,
        spend_cents: u64,
    ) -> Option<Alert> {
        if spend_cents > self.billing_budget_cents {
            let severity = if spend_cents > self.billing_budget_cents * 2 {
                AlertSeverity::Critical
            } else if spend_cents > (self.billing_budget_cents as f64 * 1.5) as u64 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::BillingBudget,
                severity,
                "Billing budget exceeded",
                format!(
                    "Spend is ${:.2}, budget is ${:.2}",
                    spend_cents as f64 / 100.0,
                    self.billing_budget_cents as f64 / 100.0,
                ),
                spend_cents as f64,
                self.billing_budget_cents as f64,
                "cents",
            ))
        } else {
            None
        }
    }

    /// Checks crash rate and returns an alert if the threshold is
    /// exceeded.
    pub fn check_crash_rate(
        &self,
        timestamp_ms: i64,
        crash_rate_per_1000: f64,
    ) -> Option<Alert> {
        if crash_rate_per_1000 > self.crash_rate_per_1000 {
            let severity = if crash_rate_per_1000 > self.crash_rate_per_1000 * 3.0 {
                AlertSeverity::Critical
            } else if crash_rate_per_1000 > self.crash_rate_per_1000 * 2.0 {
                AlertSeverity::High
            } else {
                AlertSeverity::Warning
            };
            Some(Alert::new(
                timestamp_ms,
                AlertType::CrashRate,
                severity,
                "Crash rate exceeded",
                format!(
                    "Crash rate is {:.1}/1000 sessions, threshold is {:.1}/1000",
                    crash_rate_per_1000, self.crash_rate_per_1000,
                ),
                crash_rate_per_1000,
                self.crash_rate_per_1000,
                "crashes/1000",
            ))
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Uptime monitoring
// ---------------------------------------------------------------------------

/// The status of a monitored service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    /// The service is up and responding.
    Up,
    /// The service is degraded (slow but responding).
    Degraded,
    /// The service is down (not responding).
    Down,
    /// The service status is unknown (no health check yet).
    Unknown,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        ServiceStatus::Unknown
    }
}

impl ServiceStatus {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            ServiceStatus::Up => "up",
            ServiceStatus::Degraded => "degraded",
            ServiceStatus::Down => "down",
            ServiceStatus::Unknown => "unknown",
        }
    }

    /// Returns `true` if the service is operational.
    pub fn is_operational(self) -> bool {
        matches!(self, ServiceStatus::Up | ServiceStatus::Degraded)
    }

    /// Returns `true` if the service is down.
    pub fn is_down(self) -> bool {
        matches!(self, ServiceStatus::Down)
    }
}

/// A monitored service for uptime tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredService {
    /// The name of the service (e.g., "auth", "firestore",
    /// "cloud_functions", "ai_backend").
    pub name: String,
    /// The current status of the service.
    pub status: ServiceStatus,
    /// The uptime fraction over the last 24 hours (0.0 to 1.0).
    pub uptime_24h: f64,
    /// The average response time in milliseconds.
    pub avg_response_ms: u64,
    /// The last time the service was checked (epoch milliseconds).
    pub last_checked_ms: i64,
}

impl MonitoredService {
    /// Creates a new monitored service with an unknown status.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ServiceStatus::Unknown,
            uptime_24h: 0.0,
            avg_response_ms: 0,
            last_checked_ms: 0,
        }
    }

    /// Updates the service status from a health check.
    pub fn check(&mut self, timestamp_ms: i64, response_ms: u64, is_responding: bool) {
        self.last_checked_ms = timestamp_ms;
        self.avg_response_ms = response_ms;
        if is_responding {
            if response_ms > 3_000 {
                self.status = ServiceStatus::Degraded;
            } else {
                self.status = ServiceStatus::Up;
            }
        } else {
            self.status = ServiceStatus::Down;
        }
    }

    /// Returns `true` if the service is operational.
    pub fn is_operational(&self) -> bool {
        self.status.is_operational()
    }
}

/// Aggregated uptime monitoring for all services.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UptimeMonitor {
    /// All monitored services.
    pub services: Vec<MonitoredService>,
    /// The overall uptime fraction (0.0 to 1.0).
    pub overall_uptime: f64,
    /// The number of services that are up.
    pub services_up: u32,
    /// The number of services that are down.
    pub services_down: u32,
    /// The number of services that are degraded.
    pub services_degraded: u32,
}

impl UptimeMonitor {
    /// Creates a new uptime monitor.
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            overall_uptime: 0.0,
            services_up: 0,
            services_down: 0,
            services_degraded: 0,
        }
    }

    /// Adds a service to the monitor.
    pub fn add_service(&mut self, service: MonitoredService) {
        self.services.push(service);
        self.recompute();
    }

    /// Recomputes the aggregate stats from the current service list.
    pub fn recompute(&mut self) {
        self.services_up = 0;
        self.services_down = 0;
        self.services_degraded = 0;
        let mut total_uptime = 0.0;
        for s in &self.services {
            match s.status {
                ServiceStatus::Up => self.services_up += 1,
                ServiceStatus::Degraded => self.services_degraded += 1,
                ServiceStatus::Down => self.services_down += 1,
                ServiceStatus::Unknown => {}
            }
            total_uptime += s.uptime_24h;
        }
        if self.services.is_empty() {
            self.overall_uptime = 0.0;
        } else {
            self.overall_uptime = total_uptime / self.services.len() as f64;
        }
    }

    /// Returns `true` if all services are up.
    pub fn all_up(&self) -> bool {
        !self.services.is_empty() && self.services_down == 0 && self.services_degraded == 0
    }

    /// Returns `true` if any service is down.
    pub fn has_outage(&self) -> bool {
        self.services_down > 0
    }
}

impl Default for UptimeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds the standard set of monitored services for S.T.R.I.D.E.
pub fn build_monitored_services() -> Vec<MonitoredService> {
    vec![
        MonitoredService::new("auth"),
        MonitoredService::new("firestore"),
        MonitoredService::new("cloud_storage"),
        MonitoredService::new("cloud_functions"),
        MonitoredService::new("ai_backend"),
        MonitoredService::new("push_notifications"),
    ]
}

// ---------------------------------------------------------------------------
// Release health dashboard
// ---------------------------------------------------------------------------

/// The overall health status of a release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseHealthStatus {
    /// All systems healthy, no active alerts.
    Healthy,
    /// Some warnings but no critical issues.
    Stable,
    /// Active alerts that need attention.
    Degraded,
    /// Critical issues, release may need rollback.
    Critical,
}

impl Default for ReleaseHealthStatus {
    fn default() -> Self {
        ReleaseHealthStatus::Healthy
    }
}

impl ReleaseHealthStatus {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            ReleaseHealthStatus::Healthy => "healthy",
            ReleaseHealthStatus::Stable => "stable",
            ReleaseHealthStatus::Degraded => "degraded",
            ReleaseHealthStatus::Critical => "critical",
        }
    }

    /// Returns `true` if this status is healthy or stable.
    pub fn is_good(self) -> bool {
        matches!(self, ReleaseHealthStatus::Healthy | ReleaseHealthStatus::Stable)
    }

    /// Returns `true` if this status requires immediate action.
    pub fn requires_action(self) -> bool {
        matches!(self, ReleaseHealthStatus::Degraded | ReleaseHealthStatus::Critical)
    }
}

/// A release health dashboard aggregating all monitoring data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseHealthDashboard {
    /// The app version this dashboard covers.
    pub app_version: String,
    /// The overall release health status.
    pub status: ReleaseHealthStatus,
    /// The number of active alerts.
    pub active_alerts: u32,
    /// The number of critical alerts.
    pub critical_alerts: u32,
    /// The crash-free session rate (0.0 to 1.0).
    pub crash_free_rate: f64,
    /// The number of active users in the last 24 hours.
    pub active_users_24h: u32,
    /// The number of workouts completed in the last 24 hours.
    pub workouts_24h: u32,
    /// The sync failure rate (0.0 to 1.0).
    pub sync_failure_rate: f64,
    /// The AI error rate (0.0 to 1.0).
    pub ai_error_rate: f64,
    /// The overall uptime (0.0 to 1.0).
    pub overall_uptime: f64,
    /// The total AI cost in cents for the last 24 hours.
    pub ai_cost_24h_cents: u64,
    /// The Firestore reads in the last 24 hours.
    pub firestore_reads_24h: u64,
    /// The Cloud Storage usage in bytes.
    pub storage_usage_bytes: u64,
    /// The list of active alerts.
    pub alerts: Vec<Alert>,
    /// UTC timestamp (epoch milliseconds) when the dashboard was
    /// generated.
    pub generated_at_ms: i64,
}

impl ReleaseHealthDashboard {
    /// Creates a new release health dashboard.
    pub fn new(app_version: impl Into<String>, generated_at_ms: i64) -> Self {
        Self {
            app_version: app_version.into(),
            status: ReleaseHealthStatus::Healthy,
            active_alerts: 0,
            critical_alerts: 0,
            crash_free_rate: 1.0,
            active_users_24h: 0,
            workouts_24h: 0,
            sync_failure_rate: 0.0,
            ai_error_rate: 0.0,
            overall_uptime: 1.0,
            ai_cost_24h_cents: 0,
            firestore_reads_24h: 0,
            storage_usage_bytes: 0,
            alerts: Vec::new(),
            generated_at_ms,
        }
    }

    /// Adds an alert to the dashboard.
    pub fn add_alert(&mut self, alert: Alert) {
        if alert.severity == AlertSeverity::Critical {
            self.critical_alerts += 1;
        }
        self.active_alerts += 1;
        self.alerts.push(alert);
        self.recompute_status();
    }

    /// Recomputes the overall status from the current metrics.
    pub fn recompute_status(&mut self) {
        if self.critical_alerts > 0 || self.overall_uptime < 0.90 {
            self.status = ReleaseHealthStatus::Critical;
        } else if self.active_alerts > 0
            || self.crash_free_rate < 0.95
            || self.sync_failure_rate > 0.20
            || self.ai_error_rate > 0.10
        {
            self.status = ReleaseHealthStatus::Degraded;
        } else if self.crash_free_rate < 0.99 || self.overall_uptime < 0.99 {
            self.status = ReleaseHealthStatus::Stable;
        } else {
            self.status = ReleaseHealthStatus::Healthy;
        }
    }

    /// Returns `true` if the release is healthy.
    pub fn is_healthy(&self) -> bool {
        self.status.is_good()
    }

    /// Returns `true` if the release requires immediate action.
    pub fn requires_action(&self) -> bool {
        self.status.requires_action()
    }

    /// Returns a human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Release v{}: {} — {} active alerts ({} critical), {:.2}% crash-free, {:.2}% uptime, {:.1}% sync failure, {:.1}% AI error, ${:.2} AI cost/day",
            self.app_version,
            self.status.label(),
            self.active_alerts,
            self.critical_alerts,
            self.crash_free_rate * 100.0,
            self.overall_uptime * 100.0,
            self.sync_failure_rate * 100.0,
            self.ai_error_rate * 100.0,
            self.ai_cost_24h_cents as f64 / 100.0,
        )
    }
}

// ---------------------------------------------------------------------------
// Monitoring configuration
// ---------------------------------------------------------------------------

/// Configuration for the monitoring system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether Crashlytics is enabled.
    pub crashlytics_enabled: bool,
    /// Whether Performance Monitoring is enabled.
    pub performance_monitoring_enabled: bool,
    /// Whether structured logging is enabled.
    pub structured_logging_enabled: bool,
    /// The minimum log level to report (logs below this are
    /// discarded).
    pub min_log_level: LogLevel,
    /// The alert thresholds.
    pub thresholds: AlertThresholds,
    /// Whether to send alerts to a paging system.
    pub alert_paging_enabled: bool,
    /// The sampling rate for performance traces (0.0 to 1.0).
    pub trace_sampling_rate: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            crashlytics_enabled: true,
            performance_monitoring_enabled: true,
            structured_logging_enabled: true,
            min_log_level: LogLevel::Info,
            thresholds: AlertThresholds::default(),
            alert_paging_enabled: false,
            trace_sampling_rate: 0.1,
        }
    }
}

impl MonitoringConfig {
    /// Returns `true` if a log entry at the given level should be
    /// reported.
    pub fn should_log(&self, level: LogLevel) -> bool {
        level.priority() >= self.min_log_level.priority()
    }

    /// Returns `true` if a performance trace should be sampled.
    pub fn should_sample_trace(&self) -> bool {
        self.trace_sampling_rate > 0.0
    }
}

// ---------------------------------------------------------------------------
// Log buffer
// ---------------------------------------------------------------------------

/// A bounded buffer of recent log entries for breadcrumbs and
/// diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogBuffer {
    /// The buffered log entries (newest at the end).
    pub entries: Vec<LogEntry>,
    /// The maximum number of entries to retain.
    pub max_entries: usize,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(500)
    }
}

impl LogBuffer {
    /// Creates a new log buffer with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Adds a log entry to the buffer, evicting the oldest entry if
    /// the buffer is full.
    pub fn add(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Returns the last N entries as breadcrumbs.
    pub fn breadcrumbs(&self, n: usize) -> Vec<LogEntry> {
        let start = if self.entries.len() > n {
            self.entries.len() - n
        } else {
            0
        };
        self.entries[start..].to_vec()
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of entries at or above the given level.
    pub fn count_at_level(&self, level: LogLevel) -> usize {
        self.entries
            .iter()
            .filter(|e| e.level.priority() >= level.priority())
            .count()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── LogLevel tests ──────────────────────────────────────────────

    #[test]
    fn log_level_labels_are_non_empty() {
        for level in [
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warning,
            LogLevel::Error,
            LogLevel::Critical,
        ] {
            assert!(!level.label().is_empty());
        }
    }

    #[test]
    fn log_level_priority_is_ordered() {
        assert!(LogLevel::Debug.priority() < LogLevel::Info.priority());
        assert!(LogLevel::Info.priority() < LogLevel::Warning.priority());
        assert!(LogLevel::Warning.priority() < LogLevel::Error.priority());
        assert!(LogLevel::Error.priority() < LogLevel::Critical.priority());
    }

    #[test]
    fn error_and_critical_report_to_crashlytics() {
        assert!(LogLevel::Error.reports_to_crashlytics());
        assert!(LogLevel::Critical.reports_to_crashlytics());
        assert!(!LogLevel::Info.reports_to_crashlytics());
        assert!(!LogLevel::Warning.reports_to_crashlytics());
    }

    #[test]
    fn debug_is_not_production_safe() {
        assert!(!LogLevel::Debug.is_production_safe());
        assert!(LogLevel::Info.is_production_safe());
        assert!(LogLevel::Critical.is_production_safe());
    }

    #[test]
    fn log_level_serializes_round_trip() {
        for level in [
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warning,
            LogLevel::Error,
            LogLevel::Critical,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let back: LogLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, back);
        }
    }

    // ── LogEntry tests ──────────────────────────────────────────────

    #[test]
    fn log_entry_new_has_defaults() {
        let entry = LogEntry::new(1000, LogLevel::Info, MonitoringCategory::Sync, "sync ok");
        assert_eq!(entry.timestamp_ms, 1000);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.category, MonitoringCategory::Sync);
        assert_eq!(entry.message, "sync ok");
        assert!(entry.context.is_empty());
        assert!(entry.session_id.is_none());
        assert!(entry.user_id.is_none());
    }

    #[test]
    fn log_entry_with_context_adds_pairs() {
        let entry = LogEntry::new(0, LogLevel::Warning, MonitoringCategory::Gps, "weak gps")
            .with_context("accuracy", "50m")
            .with_context("satellites", "4");
        assert_eq!(entry.context.len(), 2);
        assert_eq!(entry.context[0].key, "accuracy");
        assert_eq!(entry.context[0].value, "50m");
        assert_eq!(entry.context[1].key, "satellites");
    }

    #[test]
    fn log_entry_with_session_and_user() {
        let entry = LogEntry::new(0, LogLevel::Info, MonitoringCategory::WorkoutEngine, "started")
            .with_session("workout-123")
            .with_user("user-456");
        assert_eq!(entry.session_id, Some("workout-123".to_string()));
        assert_eq!(entry.user_id, Some("user-456".to_string()));
    }

    #[test]
    fn log_entry_should_report_error_to_crashlytics() {
        let entry = LogEntry::new(0, LogLevel::Error, MonitoringCategory::Sync, "failed");
        assert!(entry.should_report_to_crashlytics());
        let info_entry = LogEntry::new(0, LogLevel::Info, MonitoringCategory::Sync, "ok");
        assert!(!info_entry.should_report_to_crashlytics());
    }

    #[test]
    fn log_entry_serializes_round_trip() {
        let entry = LogEntry::new(123, LogLevel::Warning, MonitoringCategory::Ai, "AI slow")
            .with_context("model", "gemini")
            .with_session("s1");
        let json = serde_json::to_string(&entry).unwrap();
        let back: LogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, back);
    }

    // ── MonitoringCategory tests ────────────────────────────────────

    #[test]
    fn monitoring_category_labels_are_non_empty() {
        for cat in [
            MonitoringCategory::WorkoutEngine,
            MonitoringCategory::Gps,
            MonitoringCategory::Sync,
            MonitoringCategory::Ai,
            MonitoringCategory::Auth,
            MonitoringCategory::Database,
            MonitoringCategory::Storage,
            MonitoringCategory::CloudFunction,
            MonitoringCategory::Firestore,
            MonitoringCategory::Wearable,
            MonitoringCategory::Music,
            MonitoringCategory::Background,
            MonitoringCategory::Notification,
            MonitoringCategory::ErrorRecovery,
            MonitoringCategory::Crash,
            MonitoringCategory::Performance,
            MonitoringCategory::Billing,
        ] {
            assert!(!cat.label().is_empty());
        }
    }

    // ── CrashReport tests ───────────────────────────────────────────

    #[test]
    fn crash_report_new_has_defaults() {
        let report = CrashReport::new(
            1000,
            CrashSeverity::Fatal,
            "NullPointerException",
            "null ref",
        );
        assert_eq!(report.severity, CrashSeverity::Fatal);
        assert_eq!(report.exception_type, "NullPointerException");
        assert!(report.stack_trace.is_none());
        assert!(report.breadcrumbs.is_empty());
        assert_eq!(report.app_version, "");
        assert!(!report.during_workout);
        assert!(report.session_id.is_none());
    }

    #[test]
    fn crash_report_with_builder_methods() {
        let report = CrashReport::new(0, CrashSeverity::NonFatal, "Exception", "err")
            .with_stack_trace("at foo.rs:42")
            .with_app_version("1.0.0")
            .with_device_model("Pixel 8")
            .with_os_version("Android 14")
            .with_during_workout(true)
            .with_session("w-1");
        assert_eq!(report.stack_trace, Some("at foo.rs:42".to_string()));
        assert_eq!(report.app_version, "1.0.0");
        assert!(report.during_workout);
        assert_eq!(report.session_id, Some("w-1".to_string()));
    }

    #[test]
    fn crash_report_add_breadcrumb() {
        let mut report = CrashReport::new(0, CrashSeverity::Fatal, "Ex", "msg");
        report.add_breadcrumb(LogEntry::new(100, LogLevel::Info, MonitoringCategory::Gps, "gps fix"));
        report.add_breadcrumb(LogEntry::new(200, LogLevel::Warning, MonitoringCategory::Sync, "slow"));
        assert_eq!(report.breadcrumbs.len(), 2);
    }

    #[test]
    fn crash_report_should_escalate_for_fatal() {
        let report = CrashReport::new(0, CrashSeverity::Fatal, "Ex", "msg");
        assert!(report.should_escalate());
    }

    #[test]
    fn crash_report_should_escalate_for_during_workout() {
        let report =
            CrashReport::new(0, CrashSeverity::NonFatal, "Ex", "msg").with_during_workout(true);
        assert!(report.should_escalate());
    }

    #[test]
    fn crash_report_non_fatal_not_during_workout_not_escalate() {
        let report = CrashReport::new(0, CrashSeverity::NonFatal, "Ex", "msg");
        assert!(!report.should_escalate());
    }

    #[test]
    fn crash_severity_is_fatal() {
        assert!(CrashSeverity::Fatal.is_fatal());
        assert!(!CrashSeverity::NonFatal.is_fatal());
    }

    #[test]
    fn crash_report_serializes_round_trip() {
        let report = CrashReport::new(999, CrashSeverity::Fatal, "Ex", "msg")
            .with_app_version("2.0")
            .with_during_workout(true);
        let json = serde_json::to_string(&report).unwrap();
        let back: CrashReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, back);
    }

    // ── PerformanceTrace tests ──────────────────────────────────────

    #[test]
    fn perf_trace_new_defaults() {
        let trace = PerformanceTrace::new("workout_start", 1000, 500);
        assert_eq!(trace.name, "workout_start");
        assert_eq!(trace.duration_ms, 500);
        assert!(trace.success);
        assert!(trace.error_message.is_none());
        assert!(trace.metrics.is_empty());
    }

    #[test]
    fn perf_trace_failed_marks_error() {
        let trace = PerformanceTrace::new("sync", 0, 1000).failed("timeout");
        assert!(!trace.success);
        assert_eq!(trace.error_message, Some("timeout".to_string()));
    }

    #[test]
    fn perf_trace_with_metric_adds() {
        let trace = PerformanceTrace::new("test", 0, 100)
            .with_metric("bytes_sent", 1024)
            .with_metric("retry_count", 3);
        assert_eq!(trace.metrics.len(), 2);
        assert_eq!(trace.metrics[0].name, "bytes_sent");
        assert_eq!(trace.metrics[0].value, 1024);
    }

    #[test]
    fn perf_trace_is_slow() {
        let trace = PerformanceTrace::new("test", 0, 6000);
        assert!(trace.is_slow(5000));
        assert!(!trace.is_slow(10000));
    }

    #[test]
    fn perf_trace_serializes_round_trip() {
        let trace = PerformanceTrace::new("t", 100, 200)
            .with_metric("m", 10)
            .failed("err");
        let json = serde_json::to_string(&trace).unwrap();
        let back: PerformanceTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(trace, back);
    }

    // ── SyncFailureMetrics tests ───────────────────────────────────

    #[test]
    fn sync_metrics_new_defaults() {
        let m = SyncFailureMetrics::new(60);
        assert_eq!(m.window_minutes, 60);
        assert_eq!(m.total_attempts, 0);
        assert_eq!(m.failed, 0);
    }

    #[test]
    fn sync_metrics_record_success_and_failure() {
        let mut m = SyncFailureMetrics::new(60);
        m.record_success();
        m.record_success();
        m.record_failure();
        assert_eq!(m.total_attempts, 3);
        assert_eq!(m.successful, 2);
        assert_eq!(m.failed, 1);
    }

    #[test]
    fn sync_metrics_failure_rate() {
        let mut m = SyncFailureMetrics::new(60);
        m.record_success();
        m.record_failure();
        m.record_failure();
        assert!((m.failure_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn sync_metrics_zero_attempts_zero_rate() {
        let m = SyncFailureMetrics::new(60);
        assert_eq!(m.failure_rate(), 0.0);
        assert_eq!(m.success_rate(), 0.0);
    }

    #[test]
    fn sync_metrics_is_failing() {
        let mut m = SyncFailureMetrics::new(60);
        for _ in 0..8 {
            m.record_failure();
        }
        for _ in 0..2 {
            m.record_success();
        }
        assert!(m.is_failing(0.5));
        assert!(!m.is_failing(0.9));
    }

    #[test]
    fn sync_metrics_record_all_types() {
        let mut m = SyncFailureMetrics::new(60);
        m.record_retry();
        m.record_conflict();
        m.record_duplicate();
        m.record_tombstone();
        assert_eq!(m.retries, 1);
        assert_eq!(m.conflicts, 1);
        assert_eq!(m.duplicates_prevented, 1);
        assert_eq!(m.tombstones, 1);
    }

    #[test]
    fn sync_metrics_serializes_round_trip() {
        let mut m = SyncFailureMetrics::new(30);
        m.record_success();
        m.record_failure();
        m.record_retry();
        let json = serde_json::to_string(&m).unwrap();
        let back: SyncFailureMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    // ── AiCostMetrics tests ────────────────────────────────────────

    #[test]
    fn ai_cost_new_defaults() {
        let m = AiCostMetrics::new(1440);
        assert_eq!(m.window_minutes, 1440);
        assert_eq!(m.total_calls, 0);
        assert_eq!(m.total_cost_cents, 0);
    }

    #[test]
    fn ai_cost_record_success() {
        let mut m = AiCostMetrics::new(1440);
        m.record_success(10, 500, 200);
        m.record_success(20, 600, 300);
        assert_eq!(m.total_calls, 2);
        assert_eq!(m.successful, 2);
        assert_eq!(m.total_cost_cents, 30);
        assert_eq!(m.total_input_tokens, 1100);
        assert_eq!(m.total_output_tokens, 500);
    }

    #[test]
    fn ai_cost_record_failure() {
        let mut m = AiCostMetrics::new(60);
        m.record_success(10, 100, 50);
        m.record_failure();
        assert_eq!(m.total_calls, 2);
        assert_eq!(m.successful, 1);
        assert_eq!(m.failed, 1);
        assert_eq!(m.total_cost_cents, 10);
    }

    #[test]
    fn ai_cost_error_rate() {
        let mut m = AiCostMetrics::new(60);
        m.record_success(10, 100, 50);
        m.record_failure();
        m.record_failure();
        assert!((m.error_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn ai_cost_cache_hit_rate() {
        let mut m = AiCostMetrics::new(60);
        m.record_cache_hit();
        m.record_cache_hit();
        m.record_success(10, 100, 50);
        assert!((m.cache_hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn ai_cost_avg_cost() {
        let mut m = AiCostMetrics::new(60);
        m.record_success(10, 0, 0);
        m.record_success(20, 0, 0);
        assert!((m.avg_cost_cents() - 15.0).abs() < 0.01);
    }

    #[test]
    fn ai_cost_total_cost_usd() {
        let mut m = AiCostMetrics::new(1440);
        m.record_success(500, 0, 0);
        assert!((m.total_cost_usd() - 5.0).abs() < 0.01);
    }

    #[test]
    fn ai_cost_is_error_rate_high() {
        let mut m = AiCostMetrics::new(60);
        for _ in 0..5 {
            m.record_failure();
        }
        for _ in 0..5 {
            m.record_success(1, 0, 0);
        }
        assert!(m.is_error_rate_high(0.4));
    }

    #[test]
    fn ai_cost_record_operation() {
        let mut m = AiCostMetrics::new(60);
        m.record_operation(AiOperationType::GeneratePlan);
        m.record_operation(AiOperationType::GeneratePlan);
        m.record_operation(AiOperationType::SummarizeWorkout);
        assert_eq!(m.operation_counts.len(), 2);
        assert_eq!(m.operation_counts[0].count, 2);
        assert_eq!(m.operation_counts[1].count, 1);
    }

    #[test]
    fn ai_cost_record_fallback_and_cache() {
        let mut m = AiCostMetrics::new(60);
        m.record_fallback();
        m.record_cache_hit();
        assert_eq!(m.fallbacks, 1);
        assert_eq!(m.cache_hits, 1);
    }

    #[test]
    fn ai_cost_serializes_round_trip() {
        let mut m = AiCostMetrics::new(1440);
        m.record_success(10, 100, 50);
        m.record_failure();
        m.record_operation(AiOperationType::GeneratePlan);
        let json = serde_json::to_string(&m).unwrap();
        let back: AiCostMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn ai_operation_type_labels_non_empty() {
        for op in [
            AiOperationType::GeneratePlan,
            AiOperationType::AdjustPlan,
            AiOperationType::SummarizeWorkout,
            AiOperationType::CoachingMessage,
            AiOperationType::InterpretFeedback,
            AiOperationType::RecommendProgression,
            AiOperationType::GenerateEncouragement,
            AiOperationType::ModerateRequest,
        ] {
            assert!(!op.label().is_empty());
        }
    }

    // ── Alert tests ─────────────────────────────────────────────────

    #[test]
    fn alert_new() {
        let alert = Alert::new(
            1000,
            AlertType::SyncFailureRate,
            AlertSeverity::High,
            "Sync failing",
            "Too many failures",
            0.5,
            0.2,
            "fraction",
        );
        assert_eq!(alert.alert_type, AlertType::SyncFailureRate);
        assert_eq!(alert.severity, AlertSeverity::High);
        assert!((alert.current_value - 0.5).abs() < 0.001);
    }

    #[test]
    fn alert_requires_paging_only_critical() {
        let critical = Alert::new(0, AlertType::CrashRate, AlertSeverity::Critical, "t", "d", 0.0, 0.0, "u");
        let high = Alert::new(0, AlertType::CrashRate, AlertSeverity::High, "t", "d", 0.0, 0.0, "u");
        assert!(critical.requires_paging());
        assert!(!high.requires_paging());
    }

    #[test]
    fn alert_serializes_round_trip() {
        let alert = Alert::new(
            123,
            AlertType::AiCost,
            AlertSeverity::Warning,
            "title",
            "desc",
            100.0,
            50.0,
            "cents",
        );
        let json = serde_json::to_string(&alert).unwrap();
        let back: Alert = serde_json::from_str(&json).unwrap();
        assert_eq!(alert, back);
    }

    #[test]
    fn alert_type_labels_non_empty() {
        for at in [
            AlertType::CloudFunctionErrorRate,
            AlertType::CloudFunctionLatency,
            AlertType::FirestoreReadQuota,
            AlertType::FirestoreWriteQuota,
            AlertType::FirestoreDeleteQuota,
            AlertType::StorageUsage,
            AlertType::StorageBandwidth,
            AlertType::BillingBudget,
            AlertType::AiCost,
            AlertType::AiErrorRate,
            AlertType::SyncFailureRate,
            AlertType::UptimeDrop,
            AlertType::CrashRate,
        ] {
            assert!(!at.label().is_empty());
        }
    }

    // ── AlertThresholds tests ──────────────────────────────────────

    #[test]
    fn alert_thresholds_defaults() {
        let t = AlertThresholds::default();
        assert!(t.cloud_function_error_rate > 0.0);
        assert!(t.cloud_function_latency_ms > 0);
        assert!(t.firestore_reads_per_day > 0);
        assert!(t.storage_usage_bytes > 0);
        assert!(t.billing_budget_cents > 0);
    }

    #[test]
    fn alert_thresholds_check_cf_error_rate_exceeded() {
        let t = AlertThresholds::default();
        let alert = t.check_cloud_function_error_rate(0, 0.10);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(alert.alert_type, AlertType::CloudFunctionErrorRate);
    }

    #[test]
    fn alert_thresholds_check_cf_error_rate_not_exceeded() {
        let t = AlertThresholds::default();
        let alert = t.check_cloud_function_error_rate(0, 0.01);
        assert!(alert.is_none());
    }

    #[test]
    fn alert_thresholds_check_cf_error_rate_escalates_severity() {
        let t = AlertThresholds::default();
        let warning = t.check_cloud_function_error_rate(0, 0.06).unwrap();
        assert_eq!(warning.severity, AlertSeverity::Warning);
        let high = t.check_cloud_function_error_rate(0, 0.11).unwrap();
        assert_eq!(high.severity, AlertSeverity::High);
        let critical = t.check_cloud_function_error_rate(0, 0.16).unwrap();
        assert_eq!(critical.severity, AlertSeverity::Critical);
    }

    #[test]
    fn alert_thresholds_check_cf_latency_exceeded() {
        let t = AlertThresholds::default();
        let alert = t.check_cloud_function_latency(0, 6000);
        assert!(alert.is_some());
        let alert = t.check_cloud_function_latency(0, 3000);
        assert!(alert.is_none());
    }

    #[test]
    fn alert_thresholds_check_firestore_reads() {
        let t = AlertThresholds::default();
        let alert = t.check_firestore_reads(0, 100_000);
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().alert_type, AlertType::FirestoreReadQuota);
        assert!(t.check_firestore_reads(0, 1000).is_none());
    }

    #[test]
    fn alert_thresholds_check_firestore_writes() {
        let t = AlertThresholds::default();
        let alert = t.check_firestore_writes(0, 20_000);
        assert!(alert.is_some());
        assert!(t.check_firestore_writes(0, 1000).is_none());
    }

    #[test]
    fn alert_thresholds_check_storage_usage() {
        let t = AlertThresholds::default();
        let alert = t.check_storage_usage(0, 10_000_000_000);
        assert!(alert.is_some());
        assert!(t.check_storage_usage(0, 1_000_000_000).is_none());
    }

    #[test]
    fn alert_thresholds_check_ai_cost() {
        let t = AlertThresholds::default();
        let alert = t.check_ai_cost(0, 1000);
        assert!(alert.is_some());
        assert!(t.check_ai_cost(0, 100).is_none());
    }

    #[test]
    fn alert_thresholds_check_ai_error_rate() {
        let t = AlertThresholds::default();
        let alert = t.check_ai_error_rate(0, 0.15);
        assert!(alert.is_some());
        assert!(t.check_ai_error_rate(0, 0.05).is_none());
    }

    #[test]
    fn alert_thresholds_check_sync_failure_rate() {
        let t = AlertThresholds::default();
        let alert = t.check_sync_failure_rate(0, 0.30);
        assert!(alert.is_some());
        assert!(t.check_sync_failure_rate(0, 0.10).is_none());
    }

    #[test]
    fn alert_thresholds_check_uptime_drop() {
        let t = AlertThresholds::default();
        let alert = t.check_uptime(0, 0.95);
        assert!(alert.is_some());
        let alert = alert.unwrap();
        assert_eq!(alert.alert_type, AlertType::UptimeDrop);
        assert!(t.check_uptime(0, 0.999).is_none());
    }

    #[test]
    fn alert_thresholds_check_billing_budget() {
        let t = AlertThresholds::default();
        let alert = t.check_billing_budget(0, 10_000);
        assert!(alert.is_some());
        assert!(t.check_billing_budget(0, 1000).is_none());
    }

    #[test]
    fn alert_thresholds_check_crash_rate() {
        let t = AlertThresholds::default();
        let alert = t.check_crash_rate(0, 20.0);
        assert!(alert.is_some());
        assert!(t.check_crash_rate(0, 5.0).is_none());
    }

    // ── UptimeMonitor tests ────────────────────────────────────────

    #[test]
    fn uptime_monitor_new_empty() {
        let m = UptimeMonitor::new();
        assert!(m.services.is_empty());
        assert_eq!(m.services_up, 0);
        assert!((m.overall_uptime - 0.0).abs() < 0.001);
    }

    #[test]
    fn uptime_monitor_add_service_recomputes() {
        let mut m = UptimeMonitor::new();
        let mut s1 = MonitoredService::new("auth");
        s1.status = ServiceStatus::Up;
        s1.uptime_24h = 0.99;
        m.add_service(s1);
        assert_eq!(m.services_up, 1);
        assert!((m.overall_uptime - 0.99).abs() < 0.001);
    }

    #[test]
    fn uptime_monitor_all_up() {
        let mut m = UptimeMonitor::new();
        let mut s1 = MonitoredService::new("auth");
        s1.status = ServiceStatus::Up;
        let mut s2 = MonitoredService::new("firestore");
        s2.status = ServiceStatus::Up;
        m.add_service(s1);
        m.add_service(s2);
        assert!(m.all_up());
    }

    #[test]
    fn uptime_monitor_has_outage() {
        let mut m = UptimeMonitor::new();
        let mut s1 = MonitoredService::new("auth");
        s1.status = ServiceStatus::Up;
        let mut s2 = MonitoredService::new("firestore");
        s2.status = ServiceStatus::Down;
        m.add_service(s1);
        m.add_service(s2);
        assert!(m.has_outage());
        assert!(!m.all_up());
    }

    #[test]
    fn monitored_service_check_up() {
        let mut s = MonitoredService::new("auth");
        s.check(1000, 200, true);
        assert_eq!(s.status, ServiceStatus::Up);
        assert_eq!(s.avg_response_ms, 200);
        assert_eq!(s.last_checked_ms, 1000);
    }

    #[test]
    fn monitored_service_check_degraded() {
        let mut s = MonitoredService::new("auth");
        s.check(1000, 5000, true);
        assert_eq!(s.status, ServiceStatus::Degraded);
    }

    #[test]
    fn monitored_service_check_down() {
        let mut s = MonitoredService::new("auth");
        s.check(1000, 0, false);
        assert_eq!(s.status, ServiceStatus::Down);
    }

    #[test]
    fn build_monitored_services_has_six() {
        let services = build_monitored_services();
        assert_eq!(services.len(), 6);
        assert!(services.iter().any(|s| s.name == "auth"));
        assert!(services.iter().any(|s| s.name == "firestore"));
        assert!(services.iter().any(|s| s.name == "cloud_storage"));
        assert!(services.iter().any(|s| s.name == "cloud_functions"));
        assert!(services.iter().any(|s| s.name == "ai_backend"));
        assert!(services.iter().any(|s| s.name == "push_notifications"));
    }

    #[test]
    fn service_status_is_operational() {
        assert!(ServiceStatus::Up.is_operational());
        assert!(ServiceStatus::Degraded.is_operational());
        assert!(!ServiceStatus::Down.is_operational());
        assert!(!ServiceStatus::Unknown.is_operational());
    }

    #[test]
    fn uptime_monitor_serializes_round_trip() {
        let mut m = UptimeMonitor::new();
        let mut s = MonitoredService::new("auth");
        s.status = ServiceStatus::Up;
        s.uptime_24h = 0.99;
        m.add_service(s);
        let json = serde_json::to_string(&m).unwrap();
        let back: UptimeMonitor = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    // ── ReleaseHealthDashboard tests ───────────────────────────────

    #[test]
    fn dashboard_new_defaults_healthy() {
        let d = ReleaseHealthDashboard::new("1.0.0", 1000);
        assert_eq!(d.app_version, "1.0.0");
        assert_eq!(d.status, ReleaseHealthStatus::Healthy);
        assert_eq!(d.active_alerts, 0);
        assert!((d.crash_free_rate - 1.0).abs() < 0.001);
    }

    #[test]
    fn dashboard_add_alert_increments() {
        let mut d = ReleaseHealthDashboard::new("1.0.0", 0);
        d.add_alert(Alert::new(
            0,
            AlertType::SyncFailureRate,
            AlertSeverity::Warning,
            "t",
            "d",
            0.5,
            0.2,
            "fraction",
        ));
        assert_eq!(d.active_alerts, 1);
        assert_eq!(d.critical_alerts, 0);
        assert_eq!(d.status, ReleaseHealthStatus::Degraded);
    }

    #[test]
    fn dashboard_add_critical_alert_makes_critical() {
        let mut d = ReleaseHealthDashboard::new("1.0.0", 0);
        d.add_alert(Alert::new(
            0,
            AlertType::CrashRate,
            AlertSeverity::Critical,
            "t",
            "d",
            100.0,
            10.0,
            "crashes/1000",
        ));
        assert_eq!(d.critical_alerts, 1);
        assert_eq!(d.status, ReleaseHealthStatus::Critical);
        assert!(d.requires_action());
    }

    #[test]
    fn dashboard_low_uptime_makes_critical() {
        let mut d = ReleaseHealthDashboard::new("1.0.0", 0);
        d.overall_uptime = 0.85;
        d.recompute_status();
        assert_eq!(d.status, ReleaseHealthStatus::Critical);
    }

    #[test]
    fn dashboard_stable_when_slightly_degraded() {
        let mut d = ReleaseHealthDashboard::new("1.0.0", 0);
        d.crash_free_rate = 0.96;
        d.recompute_status();
        assert_eq!(d.status, ReleaseHealthStatus::Stable);
    }

    #[test]
    fn dashboard_healthy_when_all_good() {
        let mut d = ReleaseHealthDashboard::new("1.0.0", 0);
        d.recompute_status();
        assert_eq!(d.status, ReleaseHealthStatus::Healthy);
        assert!(d.is_healthy());
    }

    #[test]
    fn dashboard_summary_contains_version() {
        let d = ReleaseHealthDashboard::new("2.5.3", 0);
        let s = d.summary();
        assert!(s.contains("2.5.3"));
        assert!(s.contains("healthy"));
    }

    #[test]
    fn dashboard_serializes_round_trip() {
        let mut d = ReleaseHealthDashboard::new("1.0.0", 12345);
        d.add_alert(Alert::new(
            0,
            AlertType::AiCost,
            AlertSeverity::Warning,
            "t",
            "d",
            100.0,
            50.0,
            "cents",
        ));
        let json = serde_json::to_string(&d).unwrap();
        let back: ReleaseHealthDashboard = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn release_health_status_is_good() {
        assert!(ReleaseHealthStatus::Healthy.is_good());
        assert!(ReleaseHealthStatus::Stable.is_good());
        assert!(!ReleaseHealthStatus::Degraded.is_good());
        assert!(!ReleaseHealthStatus::Critical.is_good());
    }

    #[test]
    fn release_health_status_requires_action() {
        assert!(ReleaseHealthStatus::Degraded.requires_action());
        assert!(ReleaseHealthStatus::Critical.requires_action());
        assert!(!ReleaseHealthStatus::Healthy.requires_action());
        assert!(!ReleaseHealthStatus::Stable.requires_action());
    }

    // ── MonitoringConfig tests ─────────────────────────────────────

    #[test]
    fn monitoring_config_defaults() {
        let c = MonitoringConfig::default();
        assert!(c.crashlytics_enabled);
        assert!(c.performance_monitoring_enabled);
        assert!(c.structured_logging_enabled);
        assert_eq!(c.min_log_level, LogLevel::Info);
        assert!(!c.alert_paging_enabled);
        assert!(c.trace_sampling_rate > 0.0);
    }

    #[test]
    fn monitoring_config_should_log() {
        let c = MonitoringConfig::default();
        assert!(!c.should_log(LogLevel::Debug));
        assert!(c.should_log(LogLevel::Info));
        assert!(c.should_log(LogLevel::Error));
    }

    #[test]
    fn monitoring_config_should_sample_trace() {
        let c = MonitoringConfig::default();
        assert!(c.should_sample_trace());
    }

    #[test]
    fn monitoring_config_serializes_round_trip() {
        let c = MonitoringConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: MonitoringConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    // ── LogBuffer tests ────────────────────────────────────────────

    #[test]
    fn log_buffer_new_empty() {
        let buf = LogBuffer::new(100);
        assert!(buf.entries.is_empty());
        assert_eq!(buf.max_entries, 100);
    }

    #[test]
    fn log_buffer_add_evicts_oldest() {
        let mut buf = LogBuffer::new(3);
        for i in 0..5 {
            buf.add(LogEntry::new(i, LogLevel::Info, MonitoringCategory::Sync, format!("msg {i}")));
        }
        assert_eq!(buf.entries.len(), 3);
        assert_eq!(buf.entries[0].timestamp_ms, 2);
        assert_eq!(buf.entries[2].timestamp_ms, 4);
    }

    #[test]
    fn log_buffer_breadcrumbs_returns_last_n() {
        let mut buf = LogBuffer::new(100);
        for i in 0..10 {
            buf.add(LogEntry::new(i, LogLevel::Info, MonitoringCategory::Sync, format!("m{i}")));
        }
        let crumbs = buf.breadcrumbs(3);
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs[2].timestamp_ms, 9);
    }

    #[test]
    fn log_buffer_clear() {
        let mut buf = LogBuffer::new(100);
        buf.add(LogEntry::new(0, LogLevel::Info, MonitoringCategory::Sync, "m"));
        buf.clear();
        assert!(buf.entries.is_empty());
    }

    #[test]
    fn log_buffer_count_at_level() {
        let mut buf = LogBuffer::new(100);
        buf.add(LogEntry::new(0, LogLevel::Info, MonitoringCategory::Sync, "m"));
        buf.add(LogEntry::new(0, LogLevel::Error, MonitoringCategory::Sync, "m"));
        buf.add(LogEntry::new(0, LogLevel::Critical, MonitoringCategory::Sync, "m"));
        assert_eq!(buf.count_at_level(LogLevel::Error), 2);
        assert_eq!(buf.count_at_level(LogLevel::Critical), 1);
        assert_eq!(buf.count_at_level(LogLevel::Info), 3);
    }

    #[test]
    fn log_buffer_default_is_500() {
        let buf = LogBuffer::default();
        assert_eq!(buf.max_entries, 500);
    }

    #[test]
    fn log_buffer_serializes_round_trip() {
        let mut buf = LogBuffer::new(10);
        buf.add(LogEntry::new(1, LogLevel::Warning, MonitoringCategory::Gps, "weak"));
        let json = serde_json::to_string(&buf).unwrap();
        let back: LogBuffer = serde_json::from_str(&json).unwrap();
        assert_eq!(buf, back);
    }
}
