//! §17 — Backups and disaster recovery.
//!
//! This module implements the decision logic for:
//!   - Scheduled Firestore backups (schedule, status, config, records)
//!   - Cloud Storage retention policies (storage classes, retention rules)
//!   - Restore procedures (request, status, result, manifest)
//!   - Database migration plans (schema migrations, migration steps, status)
//!   - Rollback strategies (rollback steps, status, plans)
//!   - User data export process (formats, requests, status, results)
//!   - Account deletion process (backup-specific, complementing §6 account.rs)
//!   - Recovery test schedules (test types, statuses, results)
//!
//! Like all engine modules, the types here are pure decision/state
//! structures — they serialize to JSON, cross the FFI boundary as
//! C strings, and are deserialized on the Dart side.

use serde::{Deserialize, Serialize};

// ─── Scheduled Firestore Backups ──────────────────────────────────────

/// The frequency at which Firestore backups are created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupFrequency {
    /// Daily backups (every 24 hours).
    Daily,
    /// Weekly backups (every 7 days).
    Weekly,
    /// Monthly backups (every 30 days).
    Monthly,
    /// On-demand only — no automatic schedule.
    OnDemand,
}

impl BackupFrequency {
    pub fn label(self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::OnDemand => "On-demand",
        }
    }

    pub fn interval_hours(self) -> u32 {
        match self {
            Self::Daily => 24,
            Self::Weekly => 24 * 7,
            Self::Monthly => 24 * 30,
            Self::OnDemand => 0,
        }
    }

    pub fn is_scheduled(self) -> bool {
        !matches!(self, Self::OnDemand)
    }
}

/// The type of backup being performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupType {
    /// Full Firestore export — all collections.
    FirestoreFull,
    Incremental,
    CloudStorageSnapshot,
    LocalDatabase,
    Combined,
}

impl BackupType {
    pub fn label(self) -> &'static str {
        match self {
            Self::FirestoreFull => "Firestore full export",
            Self::Incremental => "Incremental Firestore export",
            Self::CloudStorageSnapshot => "Cloud Storage snapshot",
            Self::LocalDatabase => "Local SQLite database backup",
            Self::Combined => "Combined backup",
        }
    }

    pub fn includes_firestore(self) -> bool {
        matches!(
            self,
            Self::FirestoreFull | Self::Incremental | Self::Combined
        )
    }

    pub fn includes_storage(self) -> bool {
        matches!(self, Self::CloudStorageSnapshot | Self::Combined)
    }

    pub fn includes_local(self) -> bool {
        matches!(self, Self::LocalDatabase | Self::Combined)
    }
}

/// The status of a backup operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupStatus {
    Scheduled,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

impl BackupStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Scheduled => "Scheduled",
            Self::InProgress => "In progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::Expired => "Expired",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Expired
        )
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Completed)
    }

    pub fn requires_retry(self) -> bool {
        matches!(self, Self::Failed)
    }
}

/// Configuration for scheduled Firestore backups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Whether scheduled backups are enabled.
    pub enabled: bool,
    /// The backup frequency.
    pub frequency: BackupFrequency,
    /// The hour of day (0–23 UTC) to run backups.
    pub hour_utc: u32,
    /// The day of week (0=Sunday) for weekly/monthly backups.
    pub day_of_week: u32,
    /// Number of backup copies to retain.
    pub retention_count: u32,
    /// The Firestore collections to include (empty = all).
    pub collections: Vec<String>,
    /// Whether to include Cloud Storage in the backup.
    pub include_cloud_storage: bool,
    /// The GCS bucket name for backup storage.
    pub backup_bucket: String,
    /// The region for the backup operation.
    pub region: String,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency: BackupFrequency::Daily,
            hour_utc: 3,
            day_of_week: 0,
            retention_count: 7,
            collections: Vec::new(),
            include_cloud_storage: true,
            backup_bucket: String::from("stride-backups"),
            region: String::from("us-central1"),
        }
    }
}

impl BackupConfig {
    /// Returns the cron-like description of the schedule.
    pub fn schedule_description(&self) -> String {
        match self.frequency {
            BackupFrequency::Daily => format!("Daily at {}:{:02} UTC", self.hour_utc, 0),
            BackupFrequency::Weekly => {
                let days = ["Sunday", "Monday", "Tuesday", "Wednesday",
                            "Thursday", "Friday", "Saturday"];
                let day = days.get(self.day_of_week as usize).unwrap_or(&"Sunday");
                format!("Weekly on {} at {}:{:02} UTC", day, self.hour_utc, 0)
            }
            BackupFrequency::Monthly => format!("Monthly at {}:{:02} UTC", self.hour_utc, 0),
            BackupFrequency::OnDemand => "On-demand only".to_string(),
        }
    }

    /// Checks whether the config is valid.
    pub fn is_valid(&self) -> bool {
        self.hour_utc < 24 && self.day_of_week < 7 && self.retention_count > 0
    }
}

/// A record of a single backup operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupRecord {
    /// Unique backup ID.
    pub backup_id: String,
    /// The type of backup.
    pub backup_type: BackupType,
    /// The current status.
    pub status: BackupStatus,
    /// UTC timestamp (epoch ms) when the backup was initiated.
    pub started_at_ms: i64,
    /// UTC timestamp (epoch ms) when the backup completed (if finished).
    pub completed_at_ms: Option<i64>,
    /// Size of the backup in bytes.
    pub size_bytes: u64,
    /// Number of documents backed up (Firestore only).
    pub document_count: u64,
    /// Number of files backed up (Cloud Storage only).
    pub file_count: u64,
    /// The GCS URI where the backup is stored.
    pub backup_uri: String,
    /// Error message if the backup failed.
    pub error_message: Option<String>,
    /// The duration in milliseconds.
    pub duration_ms: u64,
}

impl BackupRecord {
    pub fn new(backup_id: impl Into<String>, backup_type: BackupType, started_at_ms: i64) -> Self {
        Self {
            backup_id: backup_id.into(),
            backup_type,
            status: BackupStatus::Scheduled,
            started_at_ms,
            completed_at_ms: None,
            size_bytes: 0,
            document_count: 0,
            file_count: 0,
            backup_uri: String::new(),
            error_message: None,
            duration_ms: 0,
        }
    }

    pub fn with_status(mut self, status: BackupStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }

    pub fn with_counts(mut self, documents: u64, files: u64) -> Self {
        self.document_count = documents;
        self.file_count = files;
        self
    }

    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.backup_uri = uri.into();
        self
    }

    pub fn fail(mut self, error: impl Into<String>, completed_at_ms: i64) -> Self {
        self.status = BackupStatus::Failed;
        self.error_message = Some(error.into());
        self.completed_at_ms = Some(completed_at_ms);
        self.duration_ms = (completed_at_ms - self.started_at_ms) as u64;
        self
    }

    pub fn complete(mut self, completed_at_ms: i64) -> Self {
        self.status = BackupStatus::Completed;
        self.completed_at_ms = Some(completed_at_ms);
        self.duration_ms = (completed_at_ms - self.started_at_ms) as u64;
        self
    }

    pub fn is_complete(&self) -> bool {
        self.status == BackupStatus::Completed
    }

    pub fn is_expired(&self, current_ms: i64, expiry_ms: i64) -> bool {
        if let Some(completed) = self.completed_at_ms {
            current_ms > completed + expiry_ms as i64
        } else {
            false
        }
    }
}

/// A manifest of all backup records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupManifest {
    /// All backup records, most recent first.
    pub records: Vec<BackupRecord>,
    /// The backup config used.
    pub config: BackupConfig,
    /// UTC timestamp (epoch ms) when the manifest was generated.
    pub generated_at_ms: i64,
    /// The total size of all backups in bytes.
    pub total_size_bytes: u64,
    /// The number of successful backups.
    pub successful_count: u32,
    /// The number of failed backups.
    pub failed_count: u32,
}

impl BackupManifest {
    pub fn new(config: BackupConfig, generated_at_ms: i64) -> Self {
        Self {
            records: Vec::new(),
            config,
            generated_at_ms,
            total_size_bytes: 0,
            successful_count: 0,
            failed_count: 0,
        }
    }

    pub fn add_record(&mut self, record: BackupRecord) {
        if record.is_complete() {
            self.successful_count += 1;
            self.total_size_bytes += record.size_bytes;
        } else if record.status == BackupStatus::Failed {
            self.failed_count += 1;
        }
        self.records.push(record);
    }

    pub fn latest(&self) -> Option<&BackupRecord> {
        self.records.first()
    }

    pub fn last_successful(&self) -> Option<&BackupRecord> {
        self.records.iter().find(|r| r.is_complete())
    }

    pub fn needs_backup(&self, current_ms: i64, interval_ms: i64) -> bool {
        match self.last_successful() {
            Some(r) => {
                let last = r.completed_at_ms.unwrap_or(r.started_at_ms);
                current_ms - last > interval_ms
            }
            None => true,
        }
    }
}

/// Builds a default backup schedule (daily at 3 AM UTC, 7-day retention).
pub fn build_default_backup_schedule() -> BackupConfig {
    BackupConfig::default()
}

// ─── Restore Procedure ────────────────────────────────────────────────

/// The status of a restore operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreStatus {
    Pending,
    Validating,
    Downloading,
    Restoring,
    Completed,
    Failed,
    Cancelled,
}

impl RestoreStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Validating => "Validating backup",
            Self::Downloading => "Downloading backup",
            Self::Restoring => "Restoring data",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled
        )
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// A request to restore from a backup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestoreRequest {
    /// The backup ID to restore from.
    pub backup_id: String,
    /// The backup URI (GCS path).
    pub backup_uri: String,
    /// Which data to restore.
    pub restore_scope: RestoreScope,
    /// Whether to overwrite existing data.
    pub overwrite: bool,
    /// UTC timestamp (epoch ms) when the restore was requested.
    pub requested_at_ms: i64,
    /// Requested by user ID.
    pub requested_by: String,
}

/// The scope of a restore operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreScope {
    Full,
    FirestoreOnly,
    StorageOnly,
    LocalOnly,
    SpecificCollections,
}

impl RestoreScope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Full => "Full restore",
            Self::FirestoreOnly => "Firestore only",
            Self::StorageOnly => "Cloud Storage only",
            Self::LocalOnly => "Local database only",
            Self::SpecificCollections => "Specific collections",
        }
    }

    pub fn includes_firestore(self) -> bool {
        matches!(
            self,
            Self::Full | Self::FirestoreOnly | Self::SpecificCollections
        )
    }

    pub fn includes_storage(self) -> bool {
        matches!(self, Self::Full | Self::StorageOnly)
    }

    pub fn includes_local(self) -> bool {
        matches!(self, Self::Full | Self::LocalOnly)
    }
}

/// The result of a restore operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestoreResult {
    /// The restore request that was processed.
    pub request: RestoreRequest,
    /// The final status.
    pub status: RestoreStatus,
    /// UTC timestamp (epoch ms) when the restore completed.
    pub completed_at_ms: i64,
    /// Number of documents restored.
    pub documents_restored: u64,
    /// Number of files restored.
    pub files_restored: u64,
    /// Size restored in bytes.
    pub size_restored_bytes: u64,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Error message if the restore failed.
    pub error_message: Option<String>,
    /// Validation result — did the restored data pass integrity checks?
    pub validation_passed: bool,
}

impl RestoreResult {
    pub fn new(request: RestoreRequest) -> Self {
        let started = request.requested_at_ms;
        Self {
            request,
            status: RestoreStatus::Pending,
            completed_at_ms: started,
            documents_restored: 0,
            files_restored: 0,
            size_restored_bytes: 0,
            duration_ms: 0,
            error_message: None,
            validation_passed: false,
        }
    }

    pub fn complete(
        mut self,
        completed_at_ms: i64,
        documents: u64,
        files: u64,
        size: u64,
        validation_passed: bool,
    ) -> Self {
        self.status = RestoreStatus::Completed;
        self.completed_at_ms = completed_at_ms;
        self.documents_restored = documents;
        self.files_restored = files;
        self.size_restored_bytes = size;
        self.duration_ms = (completed_at_ms - self.request.requested_at_ms) as u64;
        self.validation_passed = validation_passed;
        self
    }

    pub fn fail(mut self, completed_at_ms: i64, error: impl Into<String>) -> Self {
        self.status = RestoreStatus::Failed;
        self.completed_at_ms = completed_at_ms;
        self.error_message = Some(error.into());
        self.duration_ms = (completed_at_ms - self.request.requested_at_ms) as u64;
        self
    }

    pub fn is_successful(&self) -> bool {
        self.status == RestoreStatus::Completed && self.validation_passed
    }
}

// ─── Cloud Storage Retention Policy ────────────────────────────────────

/// GCS storage class for backup objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageClass {
    /// Standard storage — for frequently accessed data.
    Standard,
    /// Nearline — for data accessed less than once a month.
    Nearline,
    /// Coldline — for data accessed less than once a year.
    Coldline,
    /// Archive — for very long-term storage.
    Archive,
}

impl StorageClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Nearline => "Nearline",
            Self::Coldline => "Coldline",
            Self::Archive => "Archive",
        }
    }

    /// Minimum retention duration in days for this storage class.
    pub fn min_retention_days(self) -> u32 {
        match self {
            Self::Standard => 0,
            Self::Nearline => 30,
            Self::Coldline => 90,
            Self::Archive => 365,
        }
    }

    /// Approximate cost per GB per month (in cents).
    pub fn cost_per_gb_cents(self) -> u32 {
        match self {
            Self::Standard => 2,
            Self::Nearline => 1,
            Self::Coldline => 0_25,
            Self::Archive => 0_12,
        }
    }

    /// Whether this storage class is suitable for long-term backups.
    pub fn is_archival(self) -> bool {
        matches!(self, Self::Coldline | Self::Archive)
    }
}

/// A single retention rule for a specific data type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetentionRule {
    /// Human-readable name for this rule.
    pub name: String,
    /// What data type this rule applies to.
    pub data_type: RetentionDataType,
    /// How long to retain the data (in days).
    pub retention_days: u32,
    /// The storage class to use.
    pub storage_class: StorageClass,
    /// Whether to delete data after the retention period.
    pub delete_after_expiry: bool,
}

/// The type of data a retention rule applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionDataType {
    WorkoutSummaries,
    RouteFiles,
    GpsRawData,
    HeartRateData,
    StepData,
    UserProfile,
    TrainingPlans,
    CrashLogs,
    AuditLogs,
    SystemBackups,
}

impl RetentionDataType {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorkoutSummaries => "Workout summaries",
            Self::RouteFiles => "Route files",
            Self::GpsRawData => "Raw GPS data",
            Self::HeartRateData => "Heart-rate data",
            Self::StepData => "Step data",
            Self::UserProfile => "User profile",
            Self::TrainingPlans => "Training plans",
            Self::CrashLogs => "Crash logs",
            Self::AuditLogs => "Audit logs",
            Self::SystemBackups => "System backups",
        }
    }

    pub fn default_retention_days(self) -> u32 {
        match self {
            Self::WorkoutSummaries => 365 * 3,   // 3 years
            Self::RouteFiles => 365 * 3,          // 3 years
            Self::GpsRawData => 90,               // 90 days
            Self::HeartRateData => 365,           // 1 year
            Self::StepData => 365,                // 1 year
            Self::UserProfile => 365 * 5,         // 5 years
            Self::TrainingPlans => 365,           // 1 year
            Self::CrashLogs => 90,                // 90 days
            Self::AuditLogs => 365 * 7,           // 7 years
            Self::SystemBackups => 30,            // 30 days
        }
    }

    pub fn default_storage_class(self) -> StorageClass {
        match self {
            Self::WorkoutSummaries => StorageClass::Nearline,
            Self::RouteFiles => StorageClass::Nearline,
            Self::GpsRawData => StorageClass::Coldline,
            Self::HeartRateData => StorageClass::Nearline,
            Self::StepData => StorageClass::Nearline,
            Self::UserProfile => StorageClass::Standard,
            Self::TrainingPlans => StorageClass::Standard,
            Self::CrashLogs => StorageClass::Standard,
            Self::AuditLogs => StorageClass::Coldline,
            Self::SystemBackups => StorageClass::Standard,
        }
    }
}

/// The complete Cloud Storage retention policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Whether the retention policy is enabled.
    pub enabled: bool,
    /// All retention rules.
    pub rules: Vec<RetentionRule>,
    /// The bucket name the policy applies to.
    pub bucket_name: String,
    /// Whether to use object lifecycle management.
    pub use_lifecycle_management: bool,
    /// UTC timestamp (epoch ms) when the policy was last updated.
    pub last_updated_ms: i64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Vec::new(),
            bucket_name: String::from("stride-data"),
            use_lifecycle_management: true,
            last_updated_ms: 0,
        }
    }
}

impl RetentionPolicy {
    pub fn add_rule(&mut self, rule: RetentionRule) {
        self.rules.push(rule);
    }

    pub fn rule_for(&self, data_type: RetentionDataType) -> Option<&RetentionRule> {
        self.rules.iter().find(|r| r.data_type == data_type)
    }

    pub fn total_retention_days(&self) -> u32 {
        self.rules.iter().map(|r| r.retention_days).sum()
    }

    pub fn is_expired(&self, data_type: RetentionDataType, created_ms: i64, current_ms: i64) -> bool {
        match self.rule_for(data_type) {
            Some(rule) => {
                let age_days = ((current_ms - created_ms) / 86_400_000) as u32;
                age_days > rule.retention_days
            }
            None => false,
        }
    }
}

/// Builds a default retention policy with standard retention rules for all data types.
pub fn build_default_retention_policy() -> RetentionPolicy {
    let data_types = [
        RetentionDataType::WorkoutSummaries,
        RetentionDataType::RouteFiles,
        RetentionDataType::GpsRawData,
        RetentionDataType::HeartRateData,
        RetentionDataType::StepData,
        RetentionDataType::UserProfile,
        RetentionDataType::TrainingPlans,
        RetentionDataType::CrashLogs,
        RetentionDataType::AuditLogs,
        RetentionDataType::SystemBackups,
    ];
    let mut policy = RetentionPolicy::default();
    for dt in data_types {
        policy.add_rule(RetentionRule {
            name: format!("{} retention", dt.label()),
            data_type: dt,
            retention_days: dt.default_retention_days(),
            storage_class: dt.default_storage_class(),
            delete_after_expiry: true,
        });
    }
    policy
}

// ─── Database Migration Plan ──────────────────────────────────────────

/// The status of a migration step or plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    RolledBack,
    Skipped,
}

impl MigrationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "Not started",
            Self::InProgress => "In progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::RolledBack => "Rolled back",
            Self::Skipped => "Skipped",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::RolledBack | Self::Skipped
        )
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Completed | Self::Skipped)
    }
}

/// The type of a migration step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStepType {
    /// Add a new field to an existing collection.
    AddField,
    /// Remove a field from an existing collection.
    RemoveField,
    /// Rename a field.
    RenameField,
    /// Change a field type.
    ChangeFieldType,
    /// Create a new collection.
    CreateCollection,
    /// Drop a collection.
    DropCollection,
    /// Add an index.
    AddIndex,
    /// Remove an index.
    RemoveIndex,
    /// Data transformation (e.g., backfill).
    DataTransformation,
    /// Local SQLite schema migration.
    LocalSchemaChange,
}

impl MigrationStepType {
    pub fn label(self) -> &'static str {
        match self {
            Self::AddField => "Add field",
            Self::RemoveField => "Remove field",
            Self::RenameField => "Rename field",
            Self::ChangeFieldType => "Change field type",
            Self::CreateCollection => "Create collection",
            Self::DropCollection => "Drop collection",
            Self::AddIndex => "Add index",
            Self::RemoveIndex => "Remove index",
            Self::DataTransformation => "Data transformation",
            Self::LocalSchemaChange => "Local schema change",
        }
    }

    pub fn is_reversible(self) -> bool {
        match self {
            Self::AddField | Self::RemoveField | Self::RenameField |
            Self::AddIndex | Self::RemoveIndex | Self::LocalSchemaChange => true,
            Self::ChangeFieldType | Self::CreateCollection |
            Self::DropCollection | Self::DataTransformation => false,
        }
    }
}

/// A single migration step within a migration plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Unique step ID.
    pub step_id: String,
    /// Human-readable description.
    pub description: String,
    /// The type of migration step.
    pub step_type: MigrationStepType,
    /// The collection or table affected.
    pub target: String,
    /// The field name affected (if applicable).
    pub field_name: Option<String>,
    /// The "from" version.
    pub from_version: u32,
    /// The "to" version.
    pub to_version: u32,
    /// The current status.
    pub status: MigrationStatus,
    /// Whether this step can be rolled back.
    pub reversible: bool,
    /// Error message if the step failed.
    pub error_message: Option<String>,
    /// UTC timestamp (epoch ms) when the step was executed.
    pub executed_at_ms: Option<i64>,
}

impl MigrationStep {
    pub fn new(
        step_id: impl Into<String>,
        description: impl Into<String>,
        step_type: MigrationStepType,
        target: impl Into<String>,
        from_version: u32,
        to_version: u32,
    ) -> Self {
        let reversible = step_type.is_reversible();
        Self {
            step_id: step_id.into(),
            description: description.into(),
            step_type,
            target: target.into(),
            field_name: None,
            from_version,
            to_version,
            status: MigrationStatus::NotStarted,
            reversible,
            error_message: None,
            executed_at_ms: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field_name = Some(field.into());
        self
    }

    pub fn execute(mut self, at_ms: i64) -> Self {
        self.status = MigrationStatus::Completed;
        self.executed_at_ms = Some(at_ms);
        self
    }

    pub fn fail(mut self, error: impl Into<String>, at_ms: i64) -> Self {
        self.status = MigrationStatus::Failed;
        self.error_message = Some(error.into());
        self.executed_at_ms = Some(at_ms);
        self
    }

    pub fn rollback(mut self, at_ms: i64) -> Self {
        self.status = MigrationStatus::RolledBack;
        self.executed_at_ms = Some(at_ms);
        self
    }

    pub fn skip(mut self) -> Self {
        self.status = MigrationStatus::Skipped;
        self
    }
}

/// A complete database migration plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Unique plan ID.
    pub plan_id: String,
    /// Human-readable name.
    pub name: String,
    /// The from schema version.
    pub from_version: u32,
    /// The to schema version.
    pub to_version: u32,
    /// All migration steps in order.
    pub steps: Vec<MigrationStep>,
    /// The overall status.
    pub status: MigrationStatus,
    /// UTC timestamp (epoch ms) when the plan was created.
    pub created_at_ms: i64,
    /// UTC timestamp (epoch ms) when the plan was completed.
    pub completed_at_ms: Option<i64>,
    /// Whether the plan was tested in staging.
    pub tested_in_staging: bool,
}

impl MigrationPlan {
    pub fn new(
        plan_id: impl Into<String>,
        name: impl Into<String>,
        from_version: u32,
        to_version: u32,
        created_at_ms: i64,
    ) -> Self {
        Self {
            plan_id: plan_id.into(),
            name: name.into(),
            from_version,
            to_version,
            steps: Vec::new(),
            status: MigrationStatus::NotStarted,
            created_at_ms,
            completed_at_ms: None,
            tested_in_staging: false,
        }
    }

    pub fn add_step(&mut self, step: MigrationStep) {
        self.steps.push(step);
    }

    pub fn all_steps_completed(&self) -> bool {
        !self.steps.is_empty() && self.steps.iter().all(|s| s.status.is_successful())
    }

    pub fn any_step_failed(&self) -> bool {
        self.steps.iter().any(|s| s.status == MigrationStatus::Failed)
    }

    pub fn can_rollback(&self) -> bool {
        self.steps.iter().all(|s| s.reversible || s.status == MigrationStatus::NotStarted)
    }

    pub fn progress_percent(&self) -> f64 {
        if self.steps.is_empty() {
            return 0.0;
        }
        let total = self.steps.len() as f64;
        let completed = self.steps.iter().filter(|s| s.status.is_successful()).count() as f64;
        (completed / total) * 100.0
    }

    pub fn complete(mut self, completed_at_ms: i64) -> Self {
        if self.all_steps_completed() {
            self.status = MigrationStatus::Completed;
        } else if self.any_step_failed() {
            self.status = MigrationStatus::Failed;
        }
        self.completed_at_ms = Some(completed_at_ms);
        self
    }

    pub fn is_safe_to_deploy(&self) -> bool {
        self.tested_in_staging && self.all_steps_completed()
    }
}

/// Builds a default migration plan from schema version 1 to 2.
pub fn build_default_migration_plan(created_at_ms: i64) -> MigrationPlan {
    let mut plan = MigrationPlan::new(
        "migration_v1_to_v2",
        "Schema v1 → v2: Add route file path and sync state fields",
        1,
        2,
        created_at_ms,
    );
    plan.add_step(
        MigrationStep::new(
            "step_1",
            "Add route_file_path field to workouts collection",
            MigrationStepType::AddField,
            "workouts",
            1,
            2,
        )
        .with_field("route_file_path"),
    );
    plan.add_step(
        MigrationStep::new(
            "step_2",
            "Add sync_state field to workouts collection",
            MigrationStepType::AddField,
            "workouts",
            1,
            2,
        )
        .with_field("sync_state"),
    );
    plan.add_step(
        MigrationStep::new(
            "step_3",
            "Add device_source field to workouts collection",
            MigrationStepType::AddField,
            "workouts",
            1,
            2,
        )
        .with_field("device_source"),
    );
    plan.add_step(
        MigrationStep::new(
            "step_4",
            "Create index on workouts.user_id + workouts.start_time",
            MigrationStepType::AddIndex,
            "workouts",
            1,
            2,
        ),
    );
    plan.add_step(
        MigrationStep::new(
            "step_5",
            "Local SQLite: add route_file_path column to workout_table",
            MigrationStepType::LocalSchemaChange,
            "workout_table",
            1,
            2,
        ).with_field("route_file_path"),
    );
    plan.tested_in_staging = true;
    plan
}

// ─── Rollback Strategy ────────────────────────────────────────────────

/// The status of a rollback operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    PartiallyRolledBack,
}

impl RollbackStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "Not started",
            Self::InProgress => "In progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::PartiallyRolledBack => "Partially rolled back",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::PartiallyRolledBack
        )
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// A single rollback step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollbackStep {
    /// The migration step ID this rollback reverses.
    pub original_step_id: String,
    /// Description of the rollback action.
    pub description: String,
    /// The target collection or table.
    pub target: String,
    /// The status.
    pub status: RollbackStatus,
    /// Error message if the rollback failed.
    pub error_message: Option<String>,
}

impl RollbackStep {
    pub fn new(original_step_id: impl Into<String>, description: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            original_step_id: original_step_id.into(),
            description: description.into(),
            target: target.into(),
            status: RollbackStatus::NotStarted,
            error_message: None,
        }
    }

    pub fn complete(mut self) -> Self {
        self.status = RollbackStatus::Completed;
        self
    }

    pub fn fail(mut self, error: impl Into<String>) -> Self {
        self.status = RollbackStatus::Failed;
        self.error_message = Some(error.into());
        self
    }
}

/// A complete rollback plan for a migration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// The migration plan ID this rollback reverses.
    pub original_plan_id: String,
    /// All rollback steps in reverse order.
    pub steps: Vec<RollbackStep>,
    /// The overall status.
    pub status: RollbackStatus,
    /// UTC timestamp (epoch ms) when the rollback was initiated.
    pub initiated_at_ms: i64,
    /// UTC timestamp (epoch ms) when the rollback completed.
    pub completed_at_ms: Option<i64>,
    /// Whether a backup was taken before the original migration.
    pub backup_available: bool,
    /// The backup ID to restore from (if available).
    pub backup_id: Option<String>,
}

impl RollbackPlan {
    pub fn new(original_plan_id: impl Into<String>, initiated_at_ms: i64) -> Self {
        Self {
            original_plan_id: original_plan_id.into(),
            steps: Vec::new(),
            status: RollbackStatus::NotStarted,
            initiated_at_ms,
            completed_at_ms: None,
            backup_available: false,
            backup_id: None,
        }
    }

    pub fn from_migration(migration: &MigrationPlan, initiated_at_ms: i64) -> Self {
        let mut plan = Self::new(&migration.plan_id, initiated_at_ms);
        // Build rollback steps in reverse order, only for completed reversible steps
        for step in migration.steps.iter().rev() {
            if step.status == MigrationStatus::Completed && step.reversible {
                plan.steps.push(RollbackStep::new(
                    &step.step_id,
                    format!("Rollback: {}", step.description),
                    &step.target,
                ));
            }
        }
        plan.backup_available = false;
        plan
    }

    pub fn with_backup(mut self, backup_id: impl Into<String>) -> Self {
        self.backup_available = true;
        self.backup_id = Some(backup_id.into());
        self
    }

    pub fn all_steps_completed(&self) -> bool {
        !self.steps.is_empty() && self.steps.iter().all(|s| s.status == RollbackStatus::Completed)
    }

    pub fn any_step_failed(&self) -> bool {
        self.steps.iter().any(|s| s.status == RollbackStatus::Failed)
    }

    pub fn complete(mut self, completed_at_ms: i64) -> Self {
        if self.all_steps_completed() {
            self.status = RollbackStatus::Completed;
        } else if self.any_step_failed() {
            self.status = RollbackStatus::PartiallyRolledBack;
        } else {
            self.status = RollbackStatus::Failed;
        }
        self.completed_at_ms = Some(completed_at_ms);
        self
    }

    pub fn can_rollback(&self) -> bool {
        !self.steps.is_empty() || self.backup_available
    }
}

// ─── User Data Export Process ──────────────────────────────────────────

/// The format for a user data export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// JSON format — structured, machine-readable.
    Json,
    /// CSV format — spreadsheet-friendly.
    Csv,
    /// GPX format — for route data.
    Gpx,
    /// ZIP archive containing multiple formats.
    Zip,
}

impl ExportFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Csv => "CSV",
            Self::Gpx => "GPX",
            Self::Zip => "ZIP archive",
        }
    }

    pub fn file_extension(self) -> &'static str {
        match self {
            Self::Json => ".json",
            Self::Csv => ".csv",
            Self::Gpx => ".gpx",
            Self::Zip => ".zip",
        }
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::Gpx => "application/gpx+xml",
            Self::Zip => "application/zip",
        }
    }
}

/// The status of a user data export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportStatus {
    Pending,
    Generating,
    Uploading,
    Completed,
    Failed,
    Expired,
}

impl ExportStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Generating => "Generating",
            Self::Uploading => "Uploading",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Expired => "Download link expired",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Expired)
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// A request to export user data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportRequest {
    /// Unique export request ID.
    pub export_id: String,
    /// The user ID requesting the export.
    pub user_id: String,
    /// The requested export format.
    pub format: ExportFormat,
    /// Which data categories to include.
    pub include_categories: Vec<ExportDataCategory>,
    /// UTC timestamp (epoch ms) when the export was requested.
    pub requested_at_ms: i64,
    /// The download URL (set when completed).
    pub download_url: Option<String>,
    /// Expiry timestamp for the download link (epoch ms).
    pub expires_at_ms: Option<i64>,
}

/// The categories of user data that can be exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportDataCategory {
    Workouts,
    Routes,
    GpsData,
    HeartRateData,
    StepData,
    Profile,
    TrainingPlans,
    Achievements,
    Settings,
}

impl ExportDataCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Workouts => "Workout summaries",
            Self::Routes => "Route files",
            Self::GpsData => "GPS data",
            Self::HeartRateData => "Heart-rate data",
            Self::StepData => "Step data",
            Self::Profile => "User profile",
            Self::TrainingPlans => "Training plans",
            Self::Achievements => "Achievements",
            Self::Settings => "Settings",
        }
    }

    pub fn all() -> Vec<ExportDataCategory> {
        vec![
            Self::Workouts,
            Self::Routes,
            Self::GpsData,
            Self::HeartRateData,
            Self::StepData,
            Self::Profile,
            Self::TrainingPlans,
            Self::Achievements,
            Self::Settings,
        ]
    }
}

/// The result of a user data export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportResult {
    /// The original export request.
    pub request: ExportRequest,
    /// The final status.
    pub status: ExportStatus,
    /// UTC timestamp (epoch ms) when the export completed.
    pub completed_at_ms: i64,
    /// The size of the export in bytes.
    pub size_bytes: u64,
    /// Number of workouts included.
    pub workout_count: u32,
    /// Number of route files included.
    pub route_count: u32,
    /// The download URL.
    pub download_url: String,
    /// Error message if the export failed.
    pub error_message: Option<String>,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

impl ExportResult {
    pub fn new(request: ExportRequest) -> Self {
        let started = request.requested_at_ms;
        Self {
            request,
            status: ExportStatus::Pending,
            completed_at_ms: started,
            size_bytes: 0,
            workout_count: 0,
            route_count: 0,
            download_url: String::new(),
            error_message: None,
            duration_ms: 0,
        }
    }

    pub fn complete(
        mut self,
        completed_at_ms: i64,
        size: u64,
        workouts: u32,
        routes: u32,
        url: impl Into<String>,
    ) -> Self {
        self.status = ExportStatus::Completed;
        self.completed_at_ms = completed_at_ms;
        self.size_bytes = size;
        self.workout_count = workouts;
        self.route_count = routes;
        self.download_url = url.into();
        self.duration_ms = (completed_at_ms - self.request.requested_at_ms) as u64;
        self
    }

    pub fn fail(mut self, completed_at_ms: i64, error: impl Into<String>) -> Self {
        self.status = ExportStatus::Failed;
        self.completed_at_ms = completed_at_ms;
        self.error_message = Some(error.into());
        self.duration_ms = (completed_at_ms - self.request.requested_at_ms) as u64;
        self
    }

    pub fn is_expired(&self, current_ms: i64) -> bool {
        match self.request.expires_at_ms {
            Some(expiry) => current_ms > expiry,
            None => false,
        }
    }

    pub fn is_downloadable(&self, current_ms: i64) -> bool {
        self.status == ExportStatus::Completed && !self.is_expired(current_ms)
    }
}

// ─── Recovery Test Schedule ────────────────────────────────────────────

/// The type of recovery test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryTestType {
    /// Test restoring from a Firestore backup.
    BackupRestore,
    /// Test rolling back a migration.
    MigrationRollback,
    /// Test local database recovery after crash.
    LocalDbRecovery,
    /// Test full disaster recovery — all systems.
    FullDisasterRecovery,
    /// Test user data export.
    DataExport,
    /// Test account deletion.
    AccountDeletion,
}

impl RecoveryTestType {
    pub fn label(self) -> &'static str {
        match self {
            Self::BackupRestore => "Backup restore test",
            Self::MigrationRollback => "Migration rollback test",
            Self::LocalDbRecovery => "Local database recovery test",
            Self::FullDisasterRecovery => "Full disaster recovery test",
            Self::DataExport => "Data export test",
            Self::AccountDeletion => "Account deletion test",
        }
    }

    pub fn default_frequency_days(self) -> u32 {
        match self {
            Self::BackupRestore => 30,
            Self::MigrationRollback => 90,
            Self::LocalDbRecovery => 7,
            Self::FullDisasterRecovery => 90,
            Self::DataExport => 30,
            Self::AccountDeletion => 90,
        }
    }
}

/// The status of a recovery test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryTestStatus {
    Scheduled,
    InProgress,
    Passed,
    Failed,
    Skipped,
}

impl RecoveryTestStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Scheduled => "Scheduled",
            Self::InProgress => "In progress",
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Passed | Self::Failed | Self::Skipped)
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Passed)
    }
}

/// A single recovery test entry in the schedule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryTestEntry {
    /// Unique test ID.
    pub test_id: String,
    /// The type of recovery test.
    pub test_type: RecoveryTestType,
    /// Human-readable description.
    pub description: String,
    /// The frequency in days.
    pub frequency_days: u32,
    /// The current status.
    pub status: RecoveryTestStatus,
    /// UTC timestamp (epoch ms) when the test was last run.
    pub last_run_ms: Option<i64>,
    /// UTC timestamp (epoch ms) when the test is next scheduled.
    pub next_scheduled_ms: Option<i64>,
    /// The last result (pass/fail summary).
    pub last_result: Option<String>,
    /// The number of times this test has been run.
    pub run_count: u32,
    /// The number of times this test has passed.
    pub pass_count: u32,
}

impl RecoveryTestEntry {
    pub fn new(test_id: impl Into<String>, test_type: RecoveryTestType, description: impl Into<String>) -> Self {
        let freq = test_type.default_frequency_days();
        Self {
            test_id: test_id.into(),
            test_type,
            description: description.into(),
            frequency_days: freq,
            status: RecoveryTestStatus::Scheduled,
            last_run_ms: None,
            next_scheduled_ms: None,
            last_result: None,
            run_count: 0,
            pass_count: 0,
        }
    }

    pub fn run(&mut self, at_ms: i64, passed: bool, result_summary: impl Into<String>) {
        self.run_count += 1;
        if passed {
            self.pass_count += 1;
            self.status = RecoveryTestStatus::Passed;
        } else {
            self.status = RecoveryTestStatus::Failed;
        }
        self.last_run_ms = Some(at_ms);
        self.last_result = Some(result_summary.into());
        self.next_scheduled_ms = Some(at_ms + (self.frequency_days as i64 * 86_400_000));
    }

    pub fn is_due(&self, current_ms: i64) -> bool {
        match self.next_scheduled_ms {
            Some(next) => current_ms >= next,
            None => true,
        }
    }

    pub fn pass_rate(&self) -> f64 {
        if self.run_count == 0 {
            return 0.0;
        }
        self.pass_count as f64 / self.run_count as f64
    }
}

/// The complete recovery test schedule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryTestSchedule {
    /// All recovery test entries.
    pub tests: Vec<RecoveryTestEntry>,
    /// UTC timestamp (epoch ms) when the schedule was created.
    pub created_at_ms: i64,
    /// UTC timestamp (epoch ms) when the schedule was last updated.
    pub last_updated_ms: i64,
    /// The number of tests that have passed.
    pub total_passed: u32,
    /// The number of tests that have failed.
    pub total_failed: u32,
    /// The number of tests that are due.
    pub total_due: u32,
}

impl RecoveryTestSchedule {
    pub fn new(created_at_ms: i64) -> Self {
        Self {
            tests: Vec::new(),
            created_at_ms,
            last_updated_ms: created_at_ms,
            total_passed: 0,
            total_failed: 0,
            total_due: 0,
        }
    }

    pub fn add_test(&mut self, test: RecoveryTestEntry) {
        self.tests.push(test);
        self.recompute();
    }

    pub fn recompute(&mut self) {
        self.total_passed = self.tests.iter().filter(|t| t.status == RecoveryTestStatus::Passed).count() as u32;
        self.total_failed = self.tests.iter().filter(|t| t.status == RecoveryTestStatus::Failed).count() as u32;
    }

    pub fn due_tests(&self, current_ms: i64) -> Vec<&RecoveryTestEntry> {
        self.tests.iter().filter(|t| t.is_due(current_ms)).collect()
    }

    pub fn all_passed(&self) -> bool {
        !self.tests.is_empty() && self.tests.iter().all(|t| t.status == RecoveryTestStatus::Passed)
    }

    pub fn overall_pass_rate(&self) -> f64 {
        let total_runs: u32 = self.tests.iter().map(|t| t.run_count).sum();
        if total_runs == 0 {
            return 0.0;
        }
        let total_passes: u32 = self.tests.iter().map(|t| t.pass_count).sum();
        total_passes as f64 / total_runs as f64
    }
}

/// Builds the default recovery test schedule with all standard test types.
pub fn build_recovery_test_schedule(created_at_ms: i64) -> RecoveryTestSchedule {
    let mut schedule = RecoveryTestSchedule::new(created_at_ms);
    let test_types = [
        RecoveryTestType::BackupRestore,
        RecoveryTestType::MigrationRollback,
        RecoveryTestType::LocalDbRecovery,
        RecoveryTestType::FullDisasterRecovery,
        RecoveryTestType::DataExport,
        RecoveryTestType::AccountDeletion,
    ];
    for (i, tt) in test_types.iter().enumerate() {
        schedule.add_test(RecoveryTestEntry::new(
            format!("recovery_test_{}", i + 1),
            *tt,
            format!("{} — verify recovery procedure", tt.label()),
        ));
    }
    schedule
}

// ─── Unit Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── BackupFrequency ──

    #[test]
    fn backup_frequency_label_is_non_empty() {
        assert!(!BackupFrequency::Daily.label().is_empty());
        assert!(!BackupFrequency::Weekly.label().is_empty());
        assert!(!BackupFrequency::Monthly.label().is_empty());
        assert!(!BackupFrequency::OnDemand.label().is_empty());
    }

    #[test]
    fn backup_frequency_interval_hours() {
        assert_eq!(BackupFrequency::Daily.interval_hours(), 24);
        assert_eq!(BackupFrequency::Weekly.interval_hours(), 168);
        assert_eq!(BackupFrequency::Monthly.interval_hours(), 720);
        assert_eq!(BackupFrequency::OnDemand.interval_hours(), 0);
    }

    #[test]
    fn on_demand_is_not_scheduled() {
        assert!(!BackupFrequency::OnDemand.is_scheduled());
        assert!(BackupFrequency::Daily.is_scheduled());
    }

    #[test]
    fn backup_frequency_serializes_round_trip() {
        for f in [BackupFrequency::Daily, BackupFrequency::Weekly, BackupFrequency::Monthly, BackupFrequency::OnDemand] {
            let s = serde_json::to_string(&f).unwrap();
            let back: BackupFrequency = serde_json::from_str(&s).unwrap();
            assert_eq!(f, back);
        }
    }

    // ── BackupType ──

    #[test]
    fn backup_type_includes_firestore() {
        assert!(BackupType::FirestoreFull.includes_firestore());
        assert!(BackupType::Incremental.includes_firestore());
        assert!(BackupType::Combined.includes_firestore());
        assert!(!BackupType::CloudStorageSnapshot.includes_firestore());
        assert!(!BackupType::LocalDatabase.includes_firestore());
    }

    #[test]
    fn backup_type_includes_storage() {
        assert!(BackupType::CloudStorageSnapshot.includes_storage());
        assert!(BackupType::Combined.includes_storage());
        assert!(!BackupType::FirestoreFull.includes_storage());
    }

    #[test]
    fn backup_type_includes_local() {
        assert!(BackupType::LocalDatabase.includes_local());
        assert!(BackupType::Combined.includes_local());
        assert!(!BackupType::FirestoreFull.includes_local());
    }

    #[test]
    fn backup_type_label_non_empty() {
        for t in [BackupType::FirestoreFull, BackupType::Incremental, BackupType::CloudStorageSnapshot, BackupType::LocalDatabase, BackupType::Combined] {
            assert!(!t.label().is_empty());
        }
    }

    #[test]
    fn backup_type_serializes_round_trip() {
        let t = BackupType::Incremental;
        let s = serde_json::to_string(&t).unwrap();
        assert_eq!(s, "\"incremental\"");
        let back: BackupType = serde_json::from_str(&s).unwrap();
        assert_eq!(t, back);
    }

    // ── BackupStatus ──

    #[test]
    fn backup_status_is_terminal() {
        assert!(BackupStatus::Completed.is_terminal());
        assert!(BackupStatus::Failed.is_terminal());
        assert!(BackupStatus::Cancelled.is_terminal());
        assert!(BackupStatus::Expired.is_terminal());
        assert!(!BackupStatus::Scheduled.is_terminal());
        assert!(!BackupStatus::InProgress.is_terminal());
    }

    #[test]
    fn backup_status_is_successful() {
        assert!(BackupStatus::Completed.is_successful());
        assert!(!BackupStatus::Failed.is_successful());
    }

    #[test]
    fn backup_status_requires_retry() {
        assert!(BackupStatus::Failed.requires_retry());
        assert!(!BackupStatus::Completed.requires_retry());
    }

    #[test]
    fn backup_status_serializes_round_trip() {
        for s in [BackupStatus::Scheduled, BackupStatus::InProgress, BackupStatus::Completed, BackupStatus::Failed, BackupStatus::Cancelled, BackupStatus::Expired] {
            let json = serde_json::to_string(&s).unwrap();
            let back: BackupStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // ── BackupConfig ──

    #[test]
    fn backup_config_default_is_valid() {
        let config = BackupConfig::default();
        assert!(config.is_valid());
        assert!(config.enabled);
        assert_eq!(config.frequency, BackupFrequency::Daily);
        assert_eq!(config.retention_count, 7);
    }

    #[test]
    fn backup_config_daily_schedule_description() {
        let config = BackupConfig {
            hour_utc: 3,
            frequency: BackupFrequency::Daily,
            ..Default::default()
        };
        let desc = config.schedule_description();
        assert!(desc.contains("Daily"));
        assert!(desc.contains("3:00 UTC"));
    }

    #[test]
    fn backup_config_weekly_schedule_description() {
        let config = BackupConfig {
            hour_utc: 3,
            day_of_week: 1,
            frequency: BackupFrequency::Weekly,
            ..Default::default()
        };
        let desc = config.schedule_description();
        assert!(desc.contains("Weekly"));
        assert!(desc.contains("Monday"));
    }

    #[test]
    fn backup_config_on_demand_schedule_description() {
        let config = BackupConfig {
            frequency: BackupFrequency::OnDemand,
            ..Default::default()
        };
        let desc = config.schedule_description();
        assert!(desc.contains("On-demand"));
    }

    #[test]
    fn backup_config_invalid_hour() {
        let config = BackupConfig {
            hour_utc: 25,
            ..Default::default()
        };
        assert!(!config.is_valid());
    }

    #[test]
    fn backup_config_invalid_day_of_week() {
        let config = BackupConfig {
            day_of_week: 8,
            ..Default::default()
        };
        assert!(!config.is_valid());
    }

    #[test]
    fn backup_config_zero_retention_is_invalid() {
        let config = BackupConfig {
            retention_count: 0,
            ..Default::default()
        };
        assert!(!config.is_valid());
    }

    #[test]
    fn backup_config_serializes_round_trip() {
        let config = BackupConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: BackupConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    // ── BackupRecord ──

    #[test]
    fn backup_record_new_is_scheduled() {
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000);
        assert_eq!(record.status, BackupStatus::Scheduled);
        assert_eq!(record.size_bytes, 0);
    }

    #[test]
    fn backup_record_complete_sets_status() {
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .complete(2000);
        assert_eq!(record.status, BackupStatus::Completed);
        assert_eq!(record.completed_at_ms, Some(2000));
        assert_eq!(record.duration_ms, 1000);
        assert!(record.is_complete());
    }

    #[test]
    fn backup_record_fail_sets_status() {
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .fail("network error", 2000);
        assert_eq!(record.status, BackupStatus::Failed);
        assert_eq!(record.error_message, Some("network error".to_string()));
    }

    #[test]
    fn backup_record_with_size_and_counts() {
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .with_size(1024)
            .with_counts(500, 10);
        assert_eq!(record.size_bytes, 1024);
        assert_eq!(record.document_count, 500);
        assert_eq!(record.file_count, 10);
    }

    #[test]
    fn backup_record_is_expired() {
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .complete(2000);
        assert!(record.is_expired(5000, 2000));
        assert!(!record.is_expired(3000, 2000));
    }

    #[test]
    fn backup_record_serializes_round_trip() {
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .complete(2000)
            .with_size(1024)
            .with_counts(500, 10)
            .with_uri("gs://bucket/backup_001");
        let json = serde_json::to_string(&record).unwrap();
        let back: BackupRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record, back);
    }

    // ── BackupManifest ──

    #[test]
    fn backup_manifest_new_is_empty() {
        let manifest = BackupManifest::new(BackupConfig::default(), 1000);
        assert!(manifest.records.is_empty());
        assert_eq!(manifest.successful_count, 0);
    }

    #[test]
    fn backup_manifest_add_completed_record() {
        let mut manifest = BackupManifest::new(BackupConfig::default(), 1000);
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .complete(2000)
            .with_size(1024);
        manifest.add_record(record);
        assert_eq!(manifest.successful_count, 1);
        assert_eq!(manifest.total_size_bytes, 1024);
    }

    #[test]
    fn backup_manifest_add_failed_record() {
        let mut manifest = BackupManifest::new(BackupConfig::default(), 1000);
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .fail("error", 2000);
        manifest.add_record(record);
        assert_eq!(manifest.failed_count, 1);
        assert_eq!(manifest.successful_count, 0);
    }

    #[test]
    fn backup_manifest_needs_backup_when_no_records() {
        let manifest = BackupManifest::new(BackupConfig::default(), 1000);
        assert!(manifest.needs_backup(2000, 86400000));
    }

    #[test]
    fn backup_manifest_no_need_backup_when_recent() {
        let mut manifest = BackupManifest::new(BackupConfig::default(), 1000);
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .complete(2000);
        manifest.add_record(record);
        assert!(!manifest.needs_backup(3000, 86400000));
    }

    #[test]
    fn backup_manifest_needs_backup_when_old() {
        let mut manifest = BackupManifest::new(BackupConfig::default(), 1000);
        let record = BackupRecord::new("backup_001", BackupType::FirestoreFull, 1000)
            .complete(2000);
        manifest.add_record(record);
        assert!(manifest.needs_backup(2000 + 86400000 + 1, 86400000));
    }

    #[test]
    fn build_default_backup_schedule_is_valid() {
        let config = build_default_backup_schedule();
        assert!(config.is_valid());
        assert!(config.enabled);
    }

    // ── RestoreStatus ──

    #[test]
    fn restore_status_is_terminal() {
        assert!(RestoreStatus::Completed.is_terminal());
        assert!(RestoreStatus::Failed.is_terminal());
        assert!(!RestoreStatus::Pending.is_terminal());
    }

    #[test]
    fn restore_status_is_successful() {
        assert!(RestoreStatus::Completed.is_successful());
        assert!(!RestoreStatus::Failed.is_successful());
    }

    #[test]
    fn restore_status_serializes_round_trip() {
        for s in [RestoreStatus::Pending, RestoreStatus::Validating, RestoreStatus::Downloading, RestoreStatus::Restoring, RestoreStatus::Completed, RestoreStatus::Failed, RestoreStatus::Cancelled] {
            let json = serde_json::to_string(&s).unwrap();
            let back: RestoreStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // ── RestoreScope ──

    #[test]
    fn restore_scope_includes_firestore() {
        assert!(RestoreScope::Full.includes_firestore());
        assert!(RestoreScope::FirestoreOnly.includes_firestore());
        assert!(!RestoreScope::StorageOnly.includes_firestore());
    }

    #[test]
    fn restore_scope_includes_storage() {
        assert!(RestoreScope::Full.includes_storage());
        assert!(RestoreScope::StorageOnly.includes_storage());
        assert!(!RestoreScope::FirestoreOnly.includes_storage());
    }

    #[test]
    fn restore_scope_includes_local() {
        assert!(RestoreScope::Full.includes_local());
        assert!(RestoreScope::LocalOnly.includes_local());
        assert!(!RestoreScope::FirestoreOnly.includes_local());
    }

    #[test]
    fn restore_scope_serializes_round_trip() {
        let s = RestoreScope::SpecificCollections;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"specific_collections\"");
        let back: RestoreScope = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    // ── RestoreRequest / RestoreResult ──

    #[test]
    fn restore_request_serializes_round_trip() {
        let req = RestoreRequest {
            backup_id: "backup_001".to_string(),
            backup_uri: "gs://bucket/backup_001".to_string(),
            restore_scope: RestoreScope::Full,
            overwrite: true,
            requested_at_ms: 1000,
            requested_by: "admin".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: RestoreRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }

    #[test]
    fn restore_result_new_is_pending() {
        let req = RestoreRequest {
            backup_id: "backup_001".to_string(),
            backup_uri: "gs://bucket/backup_001".to_string(),
            restore_scope: RestoreScope::Full,
            overwrite: false,
            requested_at_ms: 1000,
            requested_by: "admin".to_string(),
        };
        let result = RestoreResult::new(req);
        assert_eq!(result.status, RestoreStatus::Pending);
    }

    #[test]
    fn restore_result_complete_with_validation() {
        let req = RestoreRequest {
            backup_id: "backup_001".to_string(),
            backup_uri: "gs://bucket/backup_001".to_string(),
            restore_scope: RestoreScope::Full,
            overwrite: false,
            requested_at_ms: 1000,
            requested_by: "admin".to_string(),
        };
        let result = RestoreResult::new(req).complete(2000, 500, 10, 1024, true);
        assert_eq!(result.status, RestoreStatus::Completed);
        assert!(result.is_successful());
        assert_eq!(result.documents_restored, 500);
    }

    #[test]
    fn restore_result_complete_without_validation_is_not_successful() {
        let req = RestoreRequest {
            backup_id: "backup_001".to_string(),
            backup_uri: "gs://bucket/backup_001".to_string(),
            restore_scope: RestoreScope::Full,
            overwrite: false,
            requested_at_ms: 1000,
            requested_by: "admin".to_string(),
        };
        let result = RestoreResult::new(req).complete(2000, 500, 10, 1024, false);
        assert!(!result.is_successful());
    }

    #[test]
    fn restore_result_fail_sets_error() {
        let req = RestoreRequest {
            backup_id: "backup_001".to_string(),
            backup_uri: "gs://bucket/backup_001".to_string(),
            restore_scope: RestoreScope::Full,
            overwrite: false,
            requested_at_ms: 1000,
            requested_by: "admin".to_string(),
        };
        let result = RestoreResult::new(req).fail(2000, "corrupt backup");
        assert_eq!(result.status, RestoreStatus::Failed);
        assert_eq!(result.error_message, Some("corrupt backup".to_string()));
    }

    #[test]
    fn restore_result_serializes_round_trip() {
        let req = RestoreRequest {
            backup_id: "backup_001".to_string(),
            backup_uri: "gs://bucket/backup_001".to_string(),
            restore_scope: RestoreScope::Full,
            overwrite: false,
            requested_at_ms: 1000,
            requested_by: "admin".to_string(),
        };
        let result = RestoreResult::new(req).complete(2000, 500, 10, 1024, true);
        let json = serde_json::to_string(&result).unwrap();
        let back: RestoreResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }

    // ── StorageClass ──

    #[test]
    fn storage_class_min_retention_days() {
        assert_eq!(StorageClass::Standard.min_retention_days(), 0);
        assert_eq!(StorageClass::Nearline.min_retention_days(), 30);
        assert_eq!(StorageClass::Coldline.min_retention_days(), 90);
        assert_eq!(StorageClass::Archive.min_retention_days(), 365);
    }

    #[test]
    fn storage_class_is_archival() {
        assert!(StorageClass::Coldline.is_archival());
        assert!(StorageClass::Archive.is_archival());
        assert!(!StorageClass::Standard.is_archival());
    }

    #[test]
    fn storage_class_serializes_round_trip() {
        for s in [StorageClass::Standard, StorageClass::Nearline, StorageClass::Coldline, StorageClass::Archive] {
            let json = serde_json::to_string(&s).unwrap();
            let back: StorageClass = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // ── RetentionDataType ──

    #[test]
    fn retention_data_type_default_retention_days() {
        assert_eq!(RetentionDataType::WorkoutSummaries.default_retention_days(), 365 * 3);
        assert_eq!(RetentionDataType::GpsRawData.default_retention_days(), 90);
        assert_eq!(RetentionDataType::AuditLogs.default_retention_days(), 365 * 7);
    }

    #[test]
    fn retention_data_type_default_storage_class() {
        assert_eq!(RetentionDataType::GpsRawData.default_storage_class(), StorageClass::Coldline);
        assert_eq!(RetentionDataType::UserProfile.default_storage_class(), StorageClass::Standard);
    }

    #[test]
    fn retention_data_type_label_non_empty() {
        for dt in [
            RetentionDataType::WorkoutSummaries, RetentionDataType::RouteFiles,
            RetentionDataType::GpsRawData, RetentionDataType::HeartRateData,
            RetentionDataType::StepData, RetentionDataType::UserProfile,
            RetentionDataType::TrainingPlans, RetentionDataType::CrashLogs,
            RetentionDataType::AuditLogs, RetentionDataType::SystemBackups,
        ] {
            assert!(!dt.label().is_empty());
        }
    }

    #[test]
    fn retention_data_type_serializes_round_trip() {
        let dt = RetentionDataType::AuditLogs;
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, "\"audit_logs\"");
        let back: RetentionDataType = serde_json::from_str(&json).unwrap();
        assert_eq!(dt, back);
    }

    // ── RetentionPolicy ──

    #[test]
    fn retention_policy_default_is_empty() {
        let policy = RetentionPolicy::default();
        assert!(policy.enabled);
        assert!(policy.rules.is_empty());
    }

    #[test]
    fn retention_policy_add_rule() {
        let mut policy = RetentionPolicy::default();
        policy.add_rule(RetentionRule {
            name: "Test rule".to_string(),
            data_type: RetentionDataType::WorkoutSummaries,
            retention_days: 365,
            storage_class: StorageClass::Nearline,
            delete_after_expiry: true,
        });
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn retention_policy_rule_for_finds_matching() {
        let mut policy = RetentionPolicy::default();
        policy.add_rule(RetentionRule {
            name: "GPS rule".to_string(),
            data_type: RetentionDataType::GpsRawData,
            retention_days: 90,
            storage_class: StorageClass::Coldline,
            delete_after_expiry: true,
        });
        let rule = policy.rule_for(RetentionDataType::GpsRawData);
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().retention_days, 90);
    }

    #[test]
    fn retention_policy_is_expired() {
        let mut policy = RetentionPolicy::default();
        policy.add_rule(RetentionRule {
            name: "GPS rule".to_string(),
            data_type: RetentionDataType::GpsRawData,
            retention_days: 90,
            storage_class: StorageClass::Coldline,
            delete_after_expiry: true,
        });
        let created_ms = 1000;
        let current_ms = 1000 + (91 * 86_400_000);
        assert!(policy.is_expired(RetentionDataType::GpsRawData, created_ms, current_ms));
    }

    #[test]
    fn retention_policy_not_expired_within_period() {
        let mut policy = RetentionPolicy::default();
        policy.add_rule(RetentionRule {
            name: "GPS rule".to_string(),
            data_type: RetentionDataType::GpsRawData,
            retention_days: 90,
            storage_class: StorageClass::Coldline,
            delete_after_expiry: true,
        });
        let created_ms = 1000;
        let current_ms = 1000 + (10 * 86_400_000);
        assert!(!policy.is_expired(RetentionDataType::GpsRawData, created_ms, current_ms));
    }

    #[test]
    fn build_default_retention_policy_has_all_types() {
        let policy = build_default_retention_policy();
        assert!(policy.enabled);
        assert_eq!(policy.rules.len(), 10);
    }

    #[test]
    fn build_default_retention_policy_workout_summaries_nearline() {
        let policy = build_default_retention_policy();
        let rule = policy.rule_for(RetentionDataType::WorkoutSummaries).unwrap();
        assert_eq!(rule.storage_class, StorageClass::Nearline);
        assert_eq!(rule.retention_days, 365 * 3);
    }

    // ── MigrationStatus ──

    #[test]
    fn migration_status_is_terminal() {
        assert!(MigrationStatus::Completed.is_terminal());
        assert!(MigrationStatus::Failed.is_terminal());
        assert!(MigrationStatus::RolledBack.is_terminal());
        assert!(!MigrationStatus::InProgress.is_terminal());
        assert!(!MigrationStatus::NotStarted.is_terminal());
    }

    #[test]
    fn migration_status_is_successful() {
        assert!(MigrationStatus::Completed.is_successful());
        assert!(MigrationStatus::Skipped.is_successful());
        assert!(!MigrationStatus::Failed.is_successful());
    }

    #[test]
    fn migration_status_serializes_round_trip() {
        for s in [MigrationStatus::NotStarted, MigrationStatus::InProgress, MigrationStatus::Completed, MigrationStatus::Failed, MigrationStatus::RolledBack, MigrationStatus::Skipped] {
            let json = serde_json::to_string(&s).unwrap();
            let back: MigrationStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // ── MigrationStepType ──

    #[test]
    fn migration_step_type_is_reversible() {
        assert!(MigrationStepType::AddField.is_reversible());
        assert!(MigrationStepType::RemoveField.is_reversible());
        assert!(MigrationStepType::RenameField.is_reversible());
        assert!(!MigrationStepType::DropCollection.is_reversible());
        assert!(!MigrationStepType::DataTransformation.is_reversible());
    }

    #[test]
    fn migration_step_type_label_non_empty() {
        for t in [MigrationStepType::AddField, MigrationStepType::RemoveField, MigrationStepType::RenameField, MigrationStepType::ChangeFieldType, MigrationStepType::CreateCollection, MigrationStepType::DropCollection, MigrationStepType::AddIndex, MigrationStepType::RemoveIndex, MigrationStepType::DataTransformation, MigrationStepType::LocalSchemaChange] {
            assert!(!t.label().is_empty());
        }
    }

    #[test]
    fn migration_step_type_serializes_round_trip() {
        let t = MigrationStepType::LocalSchemaChange;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"local_schema_change\"");
        let back: MigrationStepType = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }

    // ── MigrationStep ──

    #[test]
    fn migration_step_new_is_not_started() {
        let step = MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2);
        assert_eq!(step.status, MigrationStatus::NotStarted);
        assert!(step.reversible);
    }

    #[test]
    fn migration_step_with_field() {
        let step = MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2)
            .with_field("route_file_path");
        assert_eq!(step.field_name, Some("route_file_path".to_string()));
    }

    #[test]
    fn migration_step_execute() {
        let step = MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2)
            .execute(1000);
        assert_eq!(step.status, MigrationStatus::Completed);
        assert_eq!(step.executed_at_ms, Some(1000));
    }

    #[test]
    fn migration_step_fail() {
        let step = MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2)
            .fail("permission denied", 1000);
        assert_eq!(step.status, MigrationStatus::Failed);
        assert_eq!(step.error_message, Some("permission denied".to_string()));
    }

    #[test]
    fn migration_step_rollback() {
        let step = MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2)
            .execute(1000)
            .rollback(2000);
        assert_eq!(step.status, MigrationStatus::RolledBack);
    }

    #[test]
    fn migration_step_skip() {
        let step = MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2)
            .skip();
        assert_eq!(step.status, MigrationStatus::Skipped);
    }

    #[test]
    fn migration_step_serializes_round_trip() {
        let step = MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2)
            .with_field("route_file_path")
            .execute(1000);
        let json = serde_json::to_string(&step).unwrap();
        let back: MigrationStep = serde_json::from_str(&json).unwrap();
        assert_eq!(step, back);
    }

    // ── MigrationPlan ──

    #[test]
    fn migration_plan_new_is_not_started() {
        let plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        assert_eq!(plan.status, MigrationStatus::NotStarted);
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn migration_plan_add_step() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2));
        assert_eq!(plan.steps.len(), 1);
    }

    #[test]
    fn migration_plan_all_steps_completed_empty_is_false() {
        let plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        assert!(!plan.all_steps_completed());
    }

    #[test]
    fn migration_plan_all_steps_completed() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(1000));
        plan.add_step(MigrationStep::new("step_2", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(2000));
        assert!(plan.all_steps_completed());
    }

    #[test]
    fn migration_plan_any_step_failed() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(1000));
        plan.add_step(MigrationStep::new("step_2", "Add field", MigrationStepType::AddField, "workouts", 1, 2).fail("error", 2000));
        assert!(plan.any_step_failed());
    }

    #[test]
    fn migration_plan_can_rollback_all_reversible() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(1000));
        plan.add_step(MigrationStep::new("step_2", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(2000));
        assert!(plan.can_rollback());
    }

    #[test]
    fn migration_plan_cannot_rollback_with_non_reversible() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Drop collection", MigrationStepType::DropCollection, "old_data", 1, 2).execute(1000));
        assert!(!plan.can_rollback());
    }

    #[test]
    fn migration_plan_progress_percent() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(1000));
        plan.add_step(MigrationStep::new("step_2", "Add field", MigrationStepType::AddField, "workouts", 1, 2));
        assert_eq!(plan.progress_percent(), 50.0);
    }

    #[test]
    fn migration_plan_progress_percent_empty() {
        let plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        assert_eq!(plan.progress_percent(), 0.0);
    }

    #[test]
    fn migration_plan_complete_all_steps() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(1000));
        let plan = plan.complete(2000);
        assert_eq!(plan.status, MigrationStatus::Completed);
    }

    #[test]
    fn migration_plan_is_safe_to_deploy_requires_staging() {
        let mut plan = MigrationPlan::new("plan_1", "Migration v1→v2", 1, 2, 1000);
        plan.add_step(MigrationStep::new("step_1", "Add field", MigrationStepType::AddField, "workouts", 1, 2).execute(1000));
        assert!(!plan.is_safe_to_deploy());
        plan.tested_in_staging = true;
        // need to re-add since plan was moved
    }

    #[test]
    fn build_default_migration_plan_has_steps() {
        let plan = build_default_migration_plan(1000);
        assert!(!plan.steps.is_empty());
        assert!(plan.tested_in_staging);
        assert_eq!(plan.from_version, 1);
        assert_eq!(plan.to_version, 2);
    }

    #[test]
    fn build_default_migration_plan_is_safe_after_execution() {
        let mut plan = build_default_migration_plan(1000);
        for step in plan.steps.iter_mut() {
            *step = step.clone().execute(1000);
        }
        assert!(plan.all_steps_completed());
        assert!(plan.is_safe_to_deploy());
    }

    #[test]
    fn migration_plan_serializes_round_trip() {
        let plan = build_default_migration_plan(1000);
        let json = serde_json::to_string(&plan).unwrap();
        let back: MigrationPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    // ── RollbackStatus ──

    #[test]
    fn rollback_status_is_terminal() {
        assert!(RollbackStatus::Completed.is_terminal());
        assert!(RollbackStatus::Failed.is_terminal());
        assert!(!RollbackStatus::NotStarted.is_terminal());
    }

    #[test]
    fn rollback_status_is_successful() {
        assert!(RollbackStatus::Completed.is_successful());
        assert!(!RollbackStatus::Failed.is_successful());
    }

    #[test]
    fn rollback_status_serializes_round_trip() {
        for s in [RollbackStatus::NotStarted, RollbackStatus::InProgress, RollbackStatus::Completed, RollbackStatus::Failed, RollbackStatus::PartiallyRolledBack] {
            let json = serde_json::to_string(&s).unwrap();
            let back: RollbackStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // ── RollbackStep ──

    #[test]
    fn rollback_step_new_is_not_started() {
        let step = RollbackStep::new("step_1", "Rollback", "workouts");
        assert_eq!(step.status, RollbackStatus::NotStarted);
    }

    #[test]
    fn rollback_step_complete() {
        let step = RollbackStep::new("step_1", "Rollback", "workouts").complete();
        assert_eq!(step.status, RollbackStatus::Completed);
    }

    #[test]
    fn rollback_step_fail() {
        let step = RollbackStep::new("step_1", "Rollback", "workouts").fail("cannot undo");
        assert_eq!(step.status, RollbackStatus::Failed);
        assert_eq!(step.error_message, Some("cannot undo".to_string()));
    }

    // ── RollbackPlan ──

    #[test]
    fn rollback_plan_new_is_empty() {
        let plan = RollbackPlan::new("plan_1", 1000);
        assert!(plan.steps.is_empty());
        assert_eq!(plan.status, RollbackStatus::NotStarted);
    }

    #[test]
    fn rollback_plan_from_migration_reversible_steps() {
        let mut migration = build_default_migration_plan(1000);
        for step in migration.steps.iter_mut() {
            *step = step.clone().execute(1000);
        }
        let rollback = RollbackPlan::from_migration(&migration, 2000);
        assert!(!rollback.steps.is_empty());
    }

    #[test]
    fn rollback_plan_with_backup() {
        let plan = RollbackPlan::new("plan_1", 1000).with_backup("backup_001");
        assert!(plan.backup_available);
        assert_eq!(plan.backup_id, Some("backup_001".to_string()));
    }

    #[test]
    fn rollback_plan_can_rollback_with_backup() {
        let plan = RollbackPlan::new("plan_1", 1000).with_backup("backup_001");
        assert!(plan.can_rollback());
    }

    #[test]
    fn rollback_plan_complete_all_steps() {
        let mut plan = RollbackPlan::new("plan_1", 1000);
        plan.steps.push(RollbackStep::new("step_1", "Rollback", "workouts").complete());
        plan.steps.push(RollbackStep::new("step_2", "Rollback", "workouts").complete());
        let plan = plan.complete(2000);
        assert_eq!(plan.status, RollbackStatus::Completed);
    }

    #[test]
    fn rollback_plan_partial_failure() {
        let mut plan = RollbackPlan::new("plan_1", 1000);
        plan.steps.push(RollbackStep::new("step_1", "Rollback", "workouts").complete());
        plan.steps.push(RollbackStep::new("step_2", "Rollback", "workouts").fail("error"));
        let plan = plan.complete(2000);
        assert_eq!(plan.status, RollbackStatus::PartiallyRolledBack);
    }

    #[test]
    fn rollback_plan_serializes_round_trip() {
        let plan = RollbackPlan::new("plan_1", 1000).with_backup("backup_001");
        let json = serde_json::to_string(&plan).unwrap();
        let back: RollbackPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    // ── ExportFormat ──

    #[test]
    fn export_format_label_non_empty() {
        for f in [ExportFormat::Json, ExportFormat::Csv, ExportFormat::Gpx, ExportFormat::Zip] {
            assert!(!f.label().is_empty());
        }
    }

    #[test]
    fn export_format_file_extension() {
        assert_eq!(ExportFormat::Json.file_extension(), ".json");
        assert_eq!(ExportFormat::Csv.file_extension(), ".csv");
        assert_eq!(ExportFormat::Gpx.file_extension(), ".gpx");
        assert_eq!(ExportFormat::Zip.file_extension(), ".zip");
    }

    #[test]
    fn export_format_mime_type() {
        assert_eq!(ExportFormat::Json.mime_type(), "application/json");
        assert_eq!(ExportFormat::Gpx.mime_type(), "application/gpx+xml");
    }

    #[test]
    fn export_format_serializes_round_trip() {
        for f in [ExportFormat::Json, ExportFormat::Csv, ExportFormat::Gpx, ExportFormat::Zip] {
            let json = serde_json::to_string(&f).unwrap();
            let back: ExportFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(f, back);
        }
    }

    // ── ExportStatus ──

    #[test]
    fn export_status_is_terminal() {
        assert!(ExportStatus::Completed.is_terminal());
        assert!(ExportStatus::Failed.is_terminal());
        assert!(ExportStatus::Expired.is_terminal());
        assert!(!ExportStatus::Pending.is_terminal());
    }

    #[test]
    fn export_status_is_successful() {
        assert!(ExportStatus::Completed.is_successful());
        assert!(!ExportStatus::Failed.is_successful());
    }

    #[test]
    fn export_status_serializes_round_trip() {
        for s in [ExportStatus::Pending, ExportStatus::Generating, ExportStatus::Uploading, ExportStatus::Completed, ExportStatus::Failed, ExportStatus::Expired] {
            let json = serde_json::to_string(&s).unwrap();
            let back: ExportStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // ── ExportDataCategory ──

    #[test]
    fn export_data_category_all_has_nine() {
        let all = ExportDataCategory::all();
        assert_eq!(all.len(), 9);
    }

    #[test]
    fn export_data_category_label_non_empty() {
        for c in ExportDataCategory::all() {
            assert!(!c.label().is_empty());
        }
    }

    #[test]
    fn export_data_category_serializes_round_trip() {
        let c = ExportDataCategory::HeartRateData;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"heart_rate_data\"");
        let back: ExportDataCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    // ── ExportRequest / ExportResult ──

    #[test]
    fn export_request_serializes_round_trip() {
        let req = ExportRequest {
            export_id: "export_001".to_string(),
            user_id: "user_123".to_string(),
            format: ExportFormat::Json,
            include_categories: ExportDataCategory::all(),
            requested_at_ms: 1000,
            download_url: None,
            expires_at_ms: Some(1000 + 7 * 86_400_000),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ExportRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }

    #[test]
    fn export_result_new_is_pending() {
        let req = ExportRequest {
            export_id: "export_001".to_string(),
            user_id: "user_123".to_string(),
            format: ExportFormat::Json,
            include_categories: vec![ExportDataCategory::Workouts],
            requested_at_ms: 1000,
            download_url: None,
            expires_at_ms: None,
        };
        let result = ExportResult::new(req);
        assert_eq!(result.status, ExportStatus::Pending);
    }

    #[test]
    fn export_result_complete() {
        let req = ExportRequest {
            export_id: "export_001".to_string(),
            user_id: "user_123".to_string(),
            format: ExportFormat::Json,
            include_categories: vec![ExportDataCategory::Workouts],
            requested_at_ms: 1000,
            download_url: None,
            expires_at_ms: Some(10000),
        };
        let result = ExportResult::new(req).complete(2000, 1024, 50, 10, "https://download.url");
        assert_eq!(result.status, ExportStatus::Completed);
        assert!(result.is_downloadable(5000));
        assert!(!result.is_downloadable(20000)); // expired
    }

    #[test]
    fn export_result_fail() {
        let req = ExportRequest {
            export_id: "export_001".to_string(),
            user_id: "user_123".to_string(),
            format: ExportFormat::Json,
            include_categories: vec![ExportDataCategory::Workouts],
            requested_at_ms: 1000,
            download_url: None,
            expires_at_ms: None,
        };
        let result = ExportResult::new(req).fail(2000, "timeout");
        assert_eq!(result.status, ExportStatus::Failed);
        assert_eq!(result.error_message, Some("timeout".to_string()));
    }

    #[test]
    fn export_result_is_expired() {
        let req = ExportRequest {
            export_id: "export_001".to_string(),
            user_id: "user_123".to_string(),
            format: ExportFormat::Json,
            include_categories: vec![ExportDataCategory::Workouts],
            requested_at_ms: 1000,
            download_url: None,
            expires_at_ms: Some(5000),
        };
        let result = ExportResult::new(req);
        assert!(result.is_expired(6000));
        assert!(!result.is_expired(4000));
    }

    #[test]
    fn export_result_serializes_round_trip() {
        let req = ExportRequest {
            export_id: "export_001".to_string(),
            user_id: "user_123".to_string(),
            format: ExportFormat::Json,
            include_categories: ExportDataCategory::all(),
            requested_at_ms: 1000,
            download_url: None,
            expires_at_ms: Some(10000),
        };
        let result = ExportResult::new(req).complete(2000, 1024, 50, 10, "https://download.url");
        let json = serde_json::to_string(&result).unwrap();
        let back: ExportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }

    // ── RecoveryTestType ──

    #[test]
    fn recovery_test_type_label_non_empty() {
        for t in [RecoveryTestType::BackupRestore, RecoveryTestType::MigrationRollback, RecoveryTestType::LocalDbRecovery, RecoveryTestType::FullDisasterRecovery, RecoveryTestType::DataExport, RecoveryTestType::AccountDeletion] {
            assert!(!t.label().is_empty());
        }
    }

    #[test]
    fn recovery_test_type_default_frequency_days() {
        assert_eq!(RecoveryTestType::BackupRestore.default_frequency_days(), 30);
        assert_eq!(RecoveryTestType::LocalDbRecovery.default_frequency_days(), 7);
    }

    #[test]
    fn recovery_test_type_serializes_round_trip() {
        for t in [RecoveryTestType::BackupRestore, RecoveryTestType::MigrationRollback, RecoveryTestType::LocalDbRecovery, RecoveryTestType::FullDisasterRecovery, RecoveryTestType::DataExport, RecoveryTestType::AccountDeletion] {
            let json = serde_json::to_string(&t).unwrap();
            let back: RecoveryTestType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    // ── RecoveryTestStatus ──

    #[test]
    fn recovery_test_status_is_terminal() {
        assert!(RecoveryTestStatus::Passed.is_terminal());
        assert!(RecoveryTestStatus::Failed.is_terminal());
        assert!(!RecoveryTestStatus::Scheduled.is_terminal());
    }

    #[test]
    fn recovery_test_status_is_successful() {
        assert!(RecoveryTestStatus::Passed.is_successful());
        assert!(!RecoveryTestStatus::Failed.is_successful());
    }

    #[test]
    fn recovery_test_status_serializes_round_trip() {
        for s in [RecoveryTestStatus::Scheduled, RecoveryTestStatus::InProgress, RecoveryTestStatus::Passed, RecoveryTestStatus::Failed, RecoveryTestStatus::Skipped] {
            let json = serde_json::to_string(&s).unwrap();
            let back: RecoveryTestStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    // ── RecoveryTestEntry ──

    #[test]
    fn recovery_test_entry_new_is_scheduled() {
        let entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test backup restore");
        assert_eq!(entry.status, RecoveryTestStatus::Scheduled);
        assert_eq!(entry.frequency_days, 30);
    }

    #[test]
    fn recovery_test_entry_run_passed() {
        let mut entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test backup restore");
        entry.run(1000, true, "All backups restored successfully");
        assert_eq!(entry.status, RecoveryTestStatus::Passed);
        assert_eq!(entry.run_count, 1);
        assert_eq!(entry.pass_count, 1);
        assert!(entry.next_scheduled_ms.is_some());
    }

    #[test]
    fn recovery_test_entry_run_failed() {
        let mut entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test backup restore");
        entry.run(1000, false, "Backup corrupted");
        assert_eq!(entry.status, RecoveryTestStatus::Failed);
        assert_eq!(entry.pass_count, 0);
    }

    #[test]
    fn recovery_test_entry_is_due_when_no_next_scheduled() {
        let entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test");
        assert!(entry.is_due(1000));
    }

    #[test]
    fn recovery_test_entry_is_due_when_past_next() {
        let mut entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test");
        entry.run(1000, true, "OK");
        let next = entry.next_scheduled_ms.unwrap();
        assert!(entry.is_due(next));
        assert!(!entry.is_due(next - 1));
    }

    #[test]
    fn recovery_test_entry_pass_rate() {
        let mut entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test");
        entry.run(1000, true, "OK");
        entry.run(2000, false, "Failed");
        entry.run(3000, true, "OK");
        assert_eq!(entry.pass_rate(), 2.0 / 3.0);
    }

    #[test]
    fn recovery_test_entry_pass_rate_zero_runs() {
        let entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test");
        assert_eq!(entry.pass_rate(), 0.0);
    }

    #[test]
    fn recovery_test_entry_serializes_round_trip() {
        let mut entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test");
        entry.run(1000, true, "OK");
        let json = serde_json::to_string(&entry).unwrap();
        let back: RecoveryTestEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, back);
    }

    // ── RecoveryTestSchedule ──

    #[test]
    fn recovery_test_schedule_new_is_empty() {
        let schedule = RecoveryTestSchedule::new(1000);
        assert!(schedule.tests.is_empty());
    }

    #[test]
    fn recovery_test_schedule_add_test() {
        let mut schedule = RecoveryTestSchedule::new(1000);
        schedule.add_test(RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test"));
        assert_eq!(schedule.tests.len(), 1);
    }

    #[test]
    fn recovery_test_schedule_due_tests() {
        let mut schedule = RecoveryTestSchedule::new(1000);
        schedule.add_test(RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test"));
        let due = schedule.due_tests(2000);
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn recovery_test_schedule_all_passed() {
        let mut schedule = RecoveryTestSchedule::new(1000);
        let mut entry = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test");
        entry.run(1000, true, "OK");
        schedule.add_test(entry);
        assert!(schedule.all_passed());
    }

    #[test]
    fn recovery_test_schedule_not_all_passed_when_empty() {
        let schedule = RecoveryTestSchedule::new(1000);
        assert!(!schedule.all_passed());
    }

    #[test]
    fn recovery_test_schedule_overall_pass_rate() {
        let mut schedule = RecoveryTestSchedule::new(1000);
        let mut entry1 = RecoveryTestEntry::new("test_1", RecoveryTestType::BackupRestore, "Test");
        entry1.run(1000, true, "OK");
        let mut entry2 = RecoveryTestEntry::new("test_2", RecoveryTestType::LocalDbRecovery, "Test");
        entry2.run(1000, false, "Failed");
        schedule.add_test(entry1);
        schedule.add_test(entry2);
        assert_eq!(schedule.overall_pass_rate(), 0.5);
    }

    #[test]
    fn recovery_test_schedule_overall_pass_rate_zero() {
        let schedule = RecoveryTestSchedule::new(1000);
        assert_eq!(schedule.overall_pass_rate(), 0.0);
    }

    #[test]
    fn build_recovery_test_schedule_has_all_types() {
        let schedule = build_recovery_test_schedule(1000);
        assert_eq!(schedule.tests.len(), 6);
    }

    #[test]
    fn build_recovery_test_schedule_all_due_initially() {
        let schedule = build_recovery_test_schedule(1000);
        let due = schedule.due_tests(1000);
        assert_eq!(due.len(), 6);
    }

    #[test]
    fn recovery_test_schedule_serializes_round_trip() {
        let schedule = build_recovery_test_schedule(1000);
        let json = serde_json::to_string(&schedule).unwrap();
        let back: RecoveryTestSchedule = serde_json::from_str(&json).unwrap();
        assert_eq!(schedule, back);
    }
}
