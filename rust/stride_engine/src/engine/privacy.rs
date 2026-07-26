//! §18 — Privacy, legal, and safety.
//!
//! This module implements the decision logic for the privacy, legal, and
//! safety requirements that must be satisfied before publication:
//!
//!   - Privacy policy content (sections, versioning, last-updated tracking)
//!   - Terms of service (acceptance tracking, version pinning)
//!   - Health and fitness disclaimer (acknowledgement tracking)
//!   - Location-data disclosure (what location data is collected and why)
//!   - Wearable-data disclosure (what wearable/Health Connect data is used)
//!   - AI disclosure (what AI features process user data)
//!   - Music-service disclosure (third-party music service integration)
//!   - Data-retention policy (how long each data type is kept)
//!   - Account-deletion policy (what happens when a user deletes their
//!     account — complements §6 account.rs)
//!   - Support contact information (where users can get help)
//!   - Consent records (per-feature consent tracking with timestamps)
//!   - Data export process (complements §17 backups export)
//!   - Data deletion process (complements §6 account deletion)
//!   - Copyright and attribution compliance (license info, attribution
//!     requirements)
//!   - OpenStreetMap attribution (required OSM attribution)
//!   - Third-party SDK disclosures (SDK inventory + privacy implications)
//!   - Google Play Data Safety form alignment (data type × purpose × sharing
//!     matrix)
//!
//! Like all engine modules, the types here are pure decision/state
//! structures — they serialize to JSON, cross the FFI boundary as
//! C strings, and are deserialized on the Dart side.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Privacy Policy
// ─────────────────────────────────────────────────────────────────────────────

/// The version of the privacy policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyPolicyVersion {
    /// Initial privacy policy (app launch).
    V1,
    /// Updated after GDPR review.
    V2,
    /// Updated after AI disclosure addition.
    V3,
    /// Current version with full Data Safety alignment.
    V4,
}

impl PrivacyPolicyVersion {
    pub fn label(self) -> &'static str {
        match self {
            Self::V1 => "Privacy Policy v1.0",
            Self::V2 => "Privacy Policy v2.0",
            Self::V3 => "Privacy Policy v3.0",
            Self::V4 => "Privacy Policy v4.0",
        }
    }

    pub fn version_string(self) -> &'static str {
        match self {
            Self::V1 => "1.0",
            Self::V2 => "2.0",
            Self::V3 => "3.0",
            Self::V4 => "4.0",
        }
    }

    pub fn is_current(self) -> bool {
        matches!(self, Self::V4)
    }
}

/// A section of the privacy policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyPolicySection {
    /// Introduction and scope.
    Introduction,
    /// What data we collect.
    DataCollection,
    /// How we use your data.
    DataUsage,
    /// Data sharing and third parties.
    DataSharing,
    /// Data retention and deletion.
    DataRetention,
    /// User rights and choices.
    UserRights,
    /// Security measures.
    Security,
    /// Children's privacy.
    ChildrensPrivacy,
    /// Changes to this policy.
    PolicyChanges,
    /// Contact information.
    Contact,
}

impl PrivacyPolicySection {
    pub fn label(self) -> &'static str {
        match self {
            Self::Introduction => "Introduction",
            Self::DataCollection => "Data We Collect",
            Self::DataUsage => "How We Use Your Data",
            Self::DataSharing => "Data Sharing and Third Parties",
            Self::DataRetention => "Data Retention and Deletion",
            Self::UserRights => "Your Rights and Choices",
            Self::Security => "Security",
            Self::ChildrensPrivacy => "Children's Privacy",
            Self::PolicyChanges => "Changes to This Policy",
            Self::Contact => "Contact Us",
        }
    }

    pub fn order(&self) -> u32 {
        match self {
            Self::Introduction => 1,
            Self::DataCollection => 2,
            Self::DataUsage => 3,
            Self::DataSharing => 4,
            Self::DataRetention => 5,
            Self::UserRights => 6,
            Self::Security => 7,
            Self::ChildrensPrivacy => 8,
            Self::PolicyChanges => 9,
            Self::Contact => 10,
        }
    }
}

/// The complete privacy policy metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyPolicy {
    /// The policy version.
    pub version: PrivacyPolicyVersion,
    /// The URL where the policy is hosted.
    pub url: String,
    /// UTC timestamp (epoch ms) when the policy was last updated.
    pub last_updated_ms: i64,
    /// Whether the policy is currently effective.
    pub is_effective: bool,
    /// All sections that the policy covers.
    pub sections: Vec<PrivacyPolicySection>,
}

impl PrivacyPolicy {
    pub fn new(version: PrivacyPolicyVersion, last_updated_ms: i64) -> Self {
        Self {
            version,
            url: String::new(),
            last_updated_ms,
            is_effective: true,
            sections: vec![
                PrivacyPolicySection::Introduction,
                PrivacyPolicySection::DataCollection,
                PrivacyPolicySection::DataUsage,
                PrivacyPolicySection::DataSharing,
                PrivacyPolicySection::DataRetention,
                PrivacyPolicySection::UserRights,
                PrivacyPolicySection::Security,
                PrivacyPolicySection::ChildrensPrivacy,
                PrivacyPolicySection::PolicyChanges,
                PrivacyPolicySection::Contact,
            ],
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn is_complete(&self) -> bool {
        self.sections.len() >= 10
    }

    pub fn has_section(&self, section: PrivacyPolicySection) -> bool {
        self.sections.contains(&section)
    }
}

/// Builds the default (current) privacy policy.
pub fn build_default_privacy_policy(last_updated_ms: i64) -> PrivacyPolicy {
    PrivacyPolicy::new(PrivacyPolicyVersion::V4, last_updated_ms)
        .with_url("https://stride.app/privacy-policy")
}

// ─────────────────────────────────────────────────────────────────────────────
// Terms of Service
// ─────────────────────────────────────────────────────────────────────────────

/// The version of the terms of service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TermsOfServiceVersion {
    /// Initial terms of service (app launch).
    V1,
    /// Updated after legal review.
    V2,
    /// Current version.
    V3,
}

impl TermsOfServiceVersion {
    pub fn label(self) -> &'static str {
        match self {
            Self::V1 => "Terms of Service v1.0",
            Self::V2 => "Terms of Service v2.0",
            Self::V3 => "Terms of Service v3.0",
        }
    }

    pub fn version_string(self) -> &'static str {
        match self {
            Self::V1 => "1.0",
            Self::V2 => "2.0",
            Self::V3 => "3.0",
        }
    }

    pub fn is_current(self) -> bool {
        matches!(self, Self::V3)
    }
}

/// The terms of service metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermsOfService {
    /// The ToS version.
    pub version: TermsOfServiceVersion,
    /// The URL where the ToS is hosted.
    pub url: String,
    /// UTC timestamp (epoch ms) when the ToS was last updated.
    pub last_updated_ms: i64,
    /// Whether the ToS is currently effective.
    pub is_effective: bool,
}

impl TermsOfService {
    pub fn new(version: TermsOfServiceVersion, last_updated_ms: i64) -> Self {
        Self {
            version,
            url: String::new(),
            last_updated_ms,
            is_effective: true,
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }
}

/// Builds the default (current) terms of service.
pub fn build_default_terms_of_service(last_updated_ms: i64) -> TermsOfService {
    TermsOfService::new(TermsOfServiceVersion::V3, last_updated_ms)
        .with_url("https://stride.app/terms-of-service")
}

// ─────────────────────────────────────────────────────────────────────────────
// Health and Fitness Disclaimer
// ─────────────────────────────────────────────────────────────────────────────

/// Whether the user has acknowledged the health and fitness disclaimer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclaimerStatus {
    /// The disclaimer has not been shown yet.
    NotShown,
    /// The disclaimer was shown but not acknowledged.
    Shown,
    /// The disclaimer was acknowledged by the user.
    Acknowledged,
    /// The user declined the disclaimer.
    Declined,
}

impl DisclaimerStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotShown => "Not shown",
            Self::Shown => "Shown",
            Self::Acknowledged => "Acknowledged",
            Self::Declined => "Declined",
        }
    }

    pub fn is_acknowledged(self) -> bool {
        matches!(self, Self::Acknowledged)
    }

    pub fn requires_display(self) -> bool {
        matches!(self, Self::NotShown | Self::Shown)
    }
}

/// The health and fitness disclaimer record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthDisclaimer {
    /// The current status.
    pub status: DisclaimerStatus,
    /// UTC timestamp (epoch ms) when the disclaimer was acknowledged.
    pub acknowledged_at_ms: Option<i64>,
    /// The disclaimer text.
    pub text: String,
}

impl HealthDisclaimer {
    pub fn new() -> Self {
        Self {
            status: DisclaimerStatus::NotShown,
            acknowledged_at_ms: None,
            text: String::from(
                "S.T.R.I.D.E. is a fitness tracking application that provides \
                 general information about your walking and running activity. \
                 It is not a medical device, does not provide medical advice, \
                 and should not be used as a substitute for professional \
                 medical advice, diagnosis, or treatment. Always consult a \
                 qualified healthcare provider before beginning any exercise \
                 program. If you experience pain, dizziness, or discomfort \
                 during exercise, stop immediately and seek medical attention."
            ),
        }
    }

    pub fn acknowledge(mut self, at_ms: i64) -> Self {
        self.status = DisclaimerStatus::Acknowledged;
        self.acknowledged_at_ms = Some(at_ms);
        self
    }

    pub fn decline(mut self) -> Self {
        self.status = DisclaimerStatus::Declined;
        self
    }

    pub fn show(mut self) -> Self {
        if self.status == DisclaimerStatus::NotShown {
            self.status = DisclaimerStatus::Shown;
        }
        self
    }

    pub fn is_acknowledged(&self) -> bool {
        self.status.is_acknowledged()
    }
}

impl Default for HealthDisclaimer {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Data Disclosures
// ─────────────────────────────────────────────────────────────────────────────

/// The type of data disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureType {
    /// Location data (GPS, routes).
    LocationData,
    /// Wearable/Health Connect data (heart rate, steps).
    WearableData,
    /// AI coaching data (processed workout summaries, AI prompts).
    AiData,
    /// Music service integration data (playback state).
    MusicServiceData,
    /// Account and profile data.
    AccountData,
    /// Device and crash data.
    DeviceData,
}

impl DisclosureType {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocationData => "Location Data Disclosure",
            Self::WearableData => "Wearable Data Disclosure",
            Self::AiData => "AI Disclosure",
            Self::MusicServiceData => "Music Service Disclosure",
            Self::AccountData => "Account Data Disclosure",
            Self::DeviceData => "Device and Crash Data Disclosure",
        }
    }

    pub fn is_required_for_publication(self) -> bool {
        matches!(
            self,
            Self::LocationData
                | Self::WearableData
                | Self::AiData
                | Self::MusicServiceData
                | Self::AccountData
                | Self::DeviceData
        )
    }

    pub fn all() -> Vec<DisclosureType> {
        vec![
            Self::LocationData,
            Self::WearableData,
            Self::AiData,
            Self::MusicServiceData,
            Self::AccountData,
            Self::DeviceData,
        ]
    }
}

/// A data disclosure entry — describes what data is collected, why, and
/// how it is used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataDisclosure {
    /// The disclosure type.
    pub disclosure_type: DisclosureType,
    /// What data is collected.
    pub data_collected: String,
    /// Why the data is collected.
    pub purpose: String,
    /// Whether the data is shared with third parties.
    pub shared_with_third_parties: bool,
    /// Whether the data collection is optional (user can opt out).
    pub is_optional: bool,
    /// Whether the user has acknowledged this disclosure.
    pub is_acknowledged: bool,
    /// UTC timestamp (epoch ms) when the disclosure was acknowledged.
    pub acknowledged_at_ms: Option<i64>,
}

impl DataDisclosure {
    pub fn new(disclosure_type: DisclosureType) -> Self {
        let (data_collected, purpose, is_optional) = match disclosure_type {
            DisclosureType::LocationData => (
                "GPS coordinates, route paths, distance, pace, and speed data collected during active workouts. Location is only collected while a workout is in progress and is not tracked in the background unless the user explicitly starts a workout.",
                "To track your walking and running routes, calculate distance and pace, and display your route on a map.",
                false,
            ),
            DisclosureType::WearableData => (
                "Heart rate data, step count, and other fitness metrics from connected wearables or Health Connect. This data is read with your explicit permission and only when a wearable is connected.",
                "To provide accurate heart rate monitoring, step counting, and fitness analysis during your workouts.",
                true,
            ),
            DisclosureType::AiData => (
                "Workout summaries, coaching plans, and progress data sent to the AI coaching service for analysis. No raw location data or personal health records are sent to the AI service.",
                "To generate personalized coaching advice, training plans, and form analysis for your workouts.",
                true,
            ),
            DisclosureType::MusicServiceData => (
                "Music playback state (which track is playing, playback position) is used to coordinate coaching audio with music. No music content or account credentials are stored by S.T.R.I.D.E.",
                "To automatically pause music when coaching advice is delivered and resume it afterward.",
                true,
            ),
            DisclosureType::AccountData => (
                "Email address, display name, and account settings. This data is required for authentication and is stored securely in Firebase.",
                "To authenticate your account, store your profile, and sync your workout data across devices.",
                false,
            ),
            DisclosureType::DeviceData => (
                "Device model, OS version, app version, and crash logs. This data is collected automatically to help diagnose and fix crashes and performance issues.",
                "To monitor app stability, diagnose crashes, and improve the app through Crashlytics and performance monitoring.",
                false,
            ),
        };

        Self {
            disclosure_type,
            data_collected: String::from(data_collected),
            purpose: String::from(purpose),
            shared_with_third_parties: matches!(
                disclosure_type,
                DisclosureType::AiData | DisclosureType::DeviceData
            ),
            is_optional,
            is_acknowledged: false,
            acknowledged_at_ms: None,
        }
    }

    pub fn acknowledge(mut self, at_ms: i64) -> Self {
        self.is_acknowledged = true;
        self.acknowledged_at_ms = Some(at_ms);
        self
    }

    pub fn is_required(&self) -> bool {
        !self.is_optional
    }
}

/// Builds all standard data disclosures.
pub fn build_all_disclosures() -> Vec<DataDisclosure> {
    DisclosureType::all()
        .into_iter()
        .map(DataDisclosure::new)
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Data Retention Policy (Privacy)
// ─────────────────────────────────────────────────────────────────────────────

/// The type of data for retention purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyDataType {
    /// Workout summaries.
    Workouts,
    /// Route files (GPX).
    Routes,
    /// Raw GPS data.
    GpsRaw,
    /// Heart rate data.
    HeartRate,
    /// Step data.
    Steps,
    /// User profile.
    Profile,
    /// AI coaching data.
    AiCoaching,
    /// Crash logs.
    CrashLogs,
    /// Audit logs.
    AuditLogs,
}

impl PrivacyDataType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Workouts => "Workout summaries",
            Self::Routes => "Route files",
            Self::GpsRaw => "Raw GPS data",
            Self::HeartRate => "Heart rate data",
            Self::Steps => "Step data",
            Self::Profile => "User profile",
            Self::AiCoaching => "AI coaching data",
            Self::CrashLogs => "Crash logs",
            Self::AuditLogs => "Audit logs",
        }
    }

    pub fn default_retention_days(self) -> u32 {
        match self {
            Self::Workouts => 365 * 3,     // 3 years
            Self::Routes => 365 * 3,      // 3 years
            Self::GpsRaw => 90,           // 90 days
            Self::HeartRate => 365,       // 1 year
            Self::Steps => 365,          // 1 year
            Self::Profile => 365 * 10,   // until account deleted
            Self::AiCoaching => 90,      // 90 days
            Self::CrashLogs => 90,       // 90 days
            Self::AuditLogs => 365 * 7,  // 7 years (compliance)
        }
    }

    pub fn is_deletable_on_request(self) -> bool {
        matches!(self, Self::Workouts | Self::Routes | Self::GpsRaw | Self::HeartRate | Self::Steps | Self::Profile | Self::AiCoaching)
    }
}

/// A privacy retention rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyRetentionRule {
    /// The data type.
    pub data_type: PrivacyDataType,
    /// How long to retain (in days).
    pub retention_days: u32,
    /// Whether the data is deleted after retention.
    pub delete_after_expiry: bool,
    /// Whether the user can request earlier deletion.
    pub user_can_delete: bool,
}

impl PrivacyRetentionRule {
    pub fn new(data_type: PrivacyDataType) -> Self {
        Self {
            data_type,
            retention_days: data_type.default_retention_days(),
            delete_after_expiry: true,
            user_can_delete: data_type.is_deletable_on_request(),
        }
    }
}

/// The privacy data retention policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyRetentionPolicy {
    /// Whether the policy is enabled.
    pub enabled: bool,
    /// All retention rules.
    pub rules: Vec<PrivacyRetentionRule>,
    /// UTC timestamp (epoch ms) when the policy was last updated.
    pub last_updated_ms: i64,
}

impl PrivacyRetentionPolicy {
    pub fn new(last_updated_ms: i64) -> Self {
        Self {
            enabled: true,
            rules: vec![
                PrivacyRetentionRule::new(PrivacyDataType::Workouts),
                PrivacyRetentionRule::new(PrivacyDataType::Routes),
                PrivacyRetentionRule::new(PrivacyDataType::GpsRaw),
                PrivacyRetentionRule::new(PrivacyDataType::HeartRate),
                PrivacyRetentionRule::new(PrivacyDataType::Steps),
                PrivacyRetentionRule::new(PrivacyDataType::Profile),
                PrivacyRetentionRule::new(PrivacyDataType::AiCoaching),
                PrivacyRetentionRule::new(PrivacyDataType::CrashLogs),
                PrivacyRetentionRule::new(PrivacyDataType::AuditLogs),
            ],
            last_updated_ms,
        }
    }

    pub fn rule_for(&self, data_type: PrivacyDataType) -> Option<&PrivacyRetentionRule> {
        self.rules.iter().find(|r| r.data_type == data_type)
    }

    pub fn is_complete(&self) -> bool {
        self.rules.len() >= 9
    }
}

/// Builds the default privacy retention policy.
pub fn build_default_privacy_retention_policy(last_updated_ms: i64) -> PrivacyRetentionPolicy {
    PrivacyRetentionPolicy::new(last_updated_ms)
}

// ─────────────────────────────────────────────────────────────────────────────
// Account Deletion Policy
// ─────────────────────────────────────────────────────────────────────────────

/// The account deletion policy — what happens when a user requests
/// account deletion. Complements §6 account.rs DeletionScope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountDeletionPolicy {
    /// Whether account deletion is supported.
    pub is_supported: bool,
    /// The URL for the account deletion request page.
    pub deletion_url: String,
    /// How long the deletion process takes (in hours).
    pub processing_time_hours: u32,
    /// Whether the user can cancel a deletion request before it completes.
    pub can_cancel: bool,
    /// The grace period before deletion is irreversible (in hours).
    pub grace_period_hours: u32,
    /// What data is permanently deleted.
    pub permanently_deleted: Vec<PrivacyDataType>,
    /// What data is retained for legal/compliance reasons.
    pub retained_for_compliance: Vec<PrivacyDataType>,
}

impl AccountDeletionPolicy {
    pub fn new() -> Self {
        Self {
            is_supported: true,
            deletion_url: String::from("https://stride.app/account/delete"),
            processing_time_hours: 72,
            can_cancel: true,
            grace_period_hours: 168, // 7 days
            permanently_deleted: vec![
                PrivacyDataType::Workouts,
                PrivacyDataType::Routes,
                PrivacyDataType::GpsRaw,
                PrivacyDataType::HeartRate,
                PrivacyDataType::Steps,
                PrivacyDataType::Profile,
                PrivacyDataType::AiCoaching,
            ],
            retained_for_compliance: vec![
                PrivacyDataType::AuditLogs,
            ],
        }
    }

    pub fn is_data_permanently_deleted(&self, data_type: PrivacyDataType) -> bool {
        self.permanently_deleted.contains(&data_type)
    }

    pub fn is_data_retained_for_compliance(&self, data_type: PrivacyDataType) -> bool {
        self.retained_for_compliance.contains(&data_type)
    }
}

impl Default for AccountDeletionPolicy {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Support Contact
// ─────────────────────────────────────────────────────────────────────────────

/// The support contact information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportContact {
    /// The support email address.
    pub email: String,
    /// The support website URL.
    pub website: String,
    /// The support phone number (optional).
    pub phone: Option<String>,
    /// The support hours description.
    pub hours: String,
}

impl SupportContact {
    pub fn new() -> Self {
        Self {
            email: String::from("support@stride.app"),
            website: String::from("https://stride.app/support"),
            phone: None,
            hours: String::from("Monday–Friday, 9:00–17:00 UTC"),
        }
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }
}

impl Default for SupportContact {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Consent Records
// ─────────────────────────────────────────────────────────────────────────────

/// The type of consent that can be given by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentType {
    /// Consent to collect location data during workouts.
    LocationCollection,
    /// Consent to collect wearable/Health Connect data.
    WearableDataCollection,
    /// Consent to send workout data to the AI coaching service.
    AiDataProcessing,
    /// Consent to control music playback during workouts.
    MusicControl,
    /// Consent to collect crash and analytics data.
    CrashAnalyticsCollection,
    /// Consent to the privacy policy.
    PrivacyPolicy,
    /// Consent to the terms of service.
    TermsOfService,
    /// Consent to the health and fitness disclaimer.
    HealthDisclaimer,
    /// Consent to receive push notifications.
    PushNotifications,
    /// Consent to sync data across devices.
    DataSync,
}

impl ConsentType {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocationCollection => "Location Data Collection",
            Self::WearableDataCollection => "Wearable Data Collection",
            Self::AiDataProcessing => "AI Data Processing",
            Self::MusicControl => "Music Playback Control",
            Self::CrashAnalyticsCollection => "Crash and Analytics Collection",
            Self::PrivacyPolicy => "Privacy Policy Agreement",
            Self::TermsOfService => "Terms of Service Agreement",
            Self::HealthDisclaimer => "Health and Fitness Disclaimer",
            Self::PushNotifications => "Push Notifications",
            Self::DataSync => "Data Synchronization",
        }
    }

    pub fn is_required(self) -> bool {
        matches!(
            self,
            Self::LocationCollection
                | Self::PrivacyPolicy
                | Self::TermsOfService
                | Self::HealthDisclaimer
        )
    }

    pub fn is_optional(self) -> bool {
        !self.is_required()
    }

    pub fn all() -> Vec<ConsentType> {
        vec![
            Self::LocationCollection,
            Self::WearableDataCollection,
            Self::AiDataProcessing,
            Self::MusicControl,
            Self::CrashAnalyticsCollection,
            Self::PrivacyPolicy,
            Self::TermsOfService,
            Self::HealthDisclaimer,
            Self::PushNotifications,
            Self::DataSync,
        ]
    }
}

/// The status of a consent record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    /// Consent has not been requested yet.
    NotRequested,
    /// Consent was granted by the user.
    Granted,
    /// Consent was denied by the user.
    Denied,
    /// Consent was previously granted but has been revoked.
    Revoked,
    /// Consent expired and needs to be re-requested.
    Expired,
}

impl ConsentStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "Not requested",
            Self::Granted => "Granted",
            Self::Denied => "Denied",
            Self::Revoked => "Revoked",
            Self::Expired => "Expired",
        }
    }

    pub fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Granted)
    }

    pub fn can_revoke(self) -> bool {
        matches!(self, Self::Granted)
    }

    pub fn can_grant(self) -> bool {
        matches!(self, Self::NotRequested | Self::Denied | Self::Revoked | Self::Expired)
    }
}

/// A consent record — tracks the user's consent for a specific data type or feature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// The consent type.
    pub consent_type: ConsentType,
    /// The current status.
    pub status: ConsentStatus,
    /// UTC timestamp (epoch ms) when the consent was granted.
    pub granted_at_ms: Option<i64>,
    /// UTC timestamp (epoch ms) when the consent was revoked (if applicable).
    pub revoked_at_ms: Option<i64>,
    /// The consent version (which version of the policy/ToS the user agreed to).
    pub policy_version: String,
}

impl ConsentRecord {
    pub fn new(consent_type: ConsentType) -> Self {
        Self {
            consent_type,
            status: ConsentStatus::NotRequested,
            granted_at_ms: None,
            revoked_at_ms: None,
            policy_version: String::new(),
        }
    }

    pub fn grant(mut self, at_ms: i64, policy_version: impl Into<String>) -> Self {
        self.status = ConsentStatus::Granted;
        self.granted_at_ms = Some(at_ms);
        self.revoked_at_ms = None;
        self.policy_version = policy_version.into();
        self
    }

    pub fn deny(mut self) -> Self {
        self.status = ConsentStatus::Denied;
        self
    }

    pub fn revoke(mut self, at_ms: i64) -> Self {
        self.status = ConsentStatus::Revoked;
        self.revoked_at_ms = Some(at_ms);
        self
    }

    pub fn expire(mut self) -> Self {
        self.status = ConsentStatus::Expired;
        self
    }

    pub fn is_granted(&self) -> bool {
        self.status.is_granted()
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }
}

/// The full consent registry — all consent records for a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentRegistry {
    /// All consent records.
    pub records: Vec<ConsentRecord>,
    /// UTC timestamp (epoch ms) when the registry was last updated.
    pub last_updated_ms: i64,
}

impl ConsentRegistry {
    pub fn new(last_updated_ms: i64) -> Self {
        Self {
            records: ConsentType::all()
                .into_iter()
                .map(ConsentRecord::new)
                .collect(),
            last_updated_ms,
        }
    }

    pub fn record_for(&self, consent_type: ConsentType) -> Option<&ConsentRecord> {
        self.records.iter().find(|r| r.consent_type == consent_type)
    }

    pub fn record_for_mut(&mut self, consent_type: ConsentType) -> Option<&mut ConsentRecord> {
        self.records.iter_mut().find(|r| r.consent_type == consent_type)
    }

    pub fn all_required_granted(&self) -> bool {
        self.records
            .iter()
            .filter(|r| r.consent_type.is_required())
            .all(|r| r.is_granted())
    }

    pub fn all_optional_responded(&self) -> bool {
        self.records
            .iter()
            .filter(|r| r.consent_type.is_optional())
            .all(|r| !matches!(r.status, ConsentStatus::NotRequested))
    }

    pub fn is_complete(&self) -> bool {
        self.all_required_granted() && self.all_optional_responded()
    }

    pub fn granted_count(&self) -> usize {
        self.records.iter().filter(|r| r.is_granted()).count()
    }

    pub fn denied_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| matches!(r.status, ConsentStatus::Denied))
            .count()
    }
}

/// Builds a default consent registry with all consent types.
pub fn build_default_consent_registry(last_updated_ms: i64) -> ConsentRegistry {
    ConsentRegistry::new(last_updated_ms)
}

// ─────────────────────────────────────────────────────────────────────────────
// Data Export and Deletion Process
// ─────────────────────────────────────────────────────────────────────────────

/// The status of a data export request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyExportStatus {
    /// The export has been requested but not started.
    Pending,
    /// The export is being prepared.
    Preparing,
    /// The export is ready for download.
    Ready,
    /// The export has been downloaded.
    Downloaded,
    /// The export failed.
    Failed,
    /// The export link expired.
    Expired,
}

impl PrivacyExportStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Preparing => "Preparing",
            Self::Ready => "Ready for download",
            Self::Downloaded => "Downloaded",
            Self::Failed => "Failed",
            Self::Expired => "Expired",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Downloaded | Self::Failed | Self::Expired)
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Ready | Self::Downloaded)
    }
}

/// A data export request (GDPR/CCPA right to data portability).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyExportRequest {
    /// Unique request ID.
    pub request_id: String,
    /// The user ID requesting the export.
    pub user_id: String,
    /// UTC timestamp (epoch ms) when the request was made.
    pub requested_at_ms: i64,
    /// The current status.
    pub status: PrivacyExportStatus,
    /// The download URL (set when ready).
    pub download_url: Option<String>,
    /// Expiry timestamp for the download link (epoch ms).
    pub expires_at_ms: Option<i64>,
}

impl PrivacyExportRequest {
    pub fn new(request_id: impl Into<String>, user_id: impl Into<String>, requested_at_ms: i64) -> Self {
        Self {
            request_id: request_id.into(),
            user_id: user_id.into(),
            requested_at_ms,
            status: PrivacyExportStatus::Pending,
            download_url: None,
            expires_at_ms: None,
        }
    }

    pub fn ready(mut self, download_url: impl Into<String>, expires_at_ms: i64) -> Self {
        self.status = PrivacyExportStatus::Ready;
        self.download_url = Some(download_url.into());
        self.expires_at_ms = Some(expires_at_ms);
        self
    }

    pub fn fail(mut self) -> Self {
        self.status = PrivacyExportStatus::Failed;
        self
    }

    pub fn is_ready(&self) -> bool {
        self.status.is_successful()
    }

    pub fn is_expired(&self, current_ms: i64) -> bool {
        match self.expires_at_ms {
            Some(expiry) => current_ms > expiry,
            None => false,
        }
    }
}

/// The status of a data deletion request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyDeletionStatus {
    /// The deletion has been requested but not started.
    Pending,
    /// The deletion is in progress.
    InProgress,
    /// The deletion completed successfully.
    Completed,
    /// The deletion failed.
    Failed,
    /// The deletion was cancelled by the user.
    Cancelled,
}

impl PrivacyDeletionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_successful(self) -> bool {
        matches!(self, Self::Completed)
    }

    pub fn can_cancel(self) -> bool {
        matches!(self, Self::Pending | Self::InProgress)
    }
}

/// A data deletion request (GDPR/CCPA right to erasure).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyDeletionRequest {
    /// Unique request ID.
    pub request_id: String,
    /// The user ID requesting deletion.
    pub user_id: String,
    /// UTC timestamp (epoch ms) when the request was made.
    pub requested_at_ms: i64,
    /// The current status.
    pub status: PrivacyDeletionStatus,
    /// UTC timestamp (epoch ms) when the deletion completed.
    pub completed_at_ms: Option<i64>,
    /// The number of items deleted.
    pub items_deleted: u32,
}

impl PrivacyDeletionRequest {
    pub fn new(request_id: impl Into<String>, user_id: impl Into<String>, requested_at_ms: i64) -> Self {
        Self {
            request_id: request_id.into(),
            user_id: user_id.into(),
            requested_at_ms,
            status: PrivacyDeletionStatus::Pending,
            completed_at_ms: None,
            items_deleted: 0,
        }
    }

    pub fn complete(mut self, completed_at_ms: i64, items_deleted: u32) -> Self {
        self.status = PrivacyDeletionStatus::Completed;
        self.completed_at_ms = Some(completed_at_ms);
        self.items_deleted = items_deleted;
        self
    }

    pub fn fail(mut self) -> Self {
        self.status = PrivacyDeletionStatus::Failed;
        self
    }

    pub fn cancel(mut self) -> Self {
        self.status = PrivacyDeletionStatus::Cancelled;
        self
    }

    pub fn is_successful(&self) -> bool {
        self.status.is_successful()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Copyright and Attribution
// ─────────────────────────────────────────────────────────────────────────────

/// The type of attribution required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionType {
    /// OpenStreetMap attribution.
    OpenStreetMap,
    /// Google Maps attribution.
    GoogleMaps,
    /// Flutter framework attribution.
    FlutterFramework,
    /// Rust engine attribution.
    RustEngine,
    /// Third-party SDK attribution.
    ThirdPartySdk,
    /// Music service attribution.
    MusicService,
}

impl AttributionType {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpenStreetMap => "OpenStreetMap",
            Self::GoogleMaps => "Google Maps",
            Self::FlutterFramework => "Flutter Framework",
            Self::RustEngine => "Rust Engine",
            Self::ThirdPartySdk => "Third-party SDK",
            Self::MusicService => "Music Service",
        }
    }

    pub fn is_required(self) -> bool {
        matches!(self, Self::OpenStreetMap | Self::FlutterFramework | Self::ThirdPartySdk)
    }

    pub fn attribution_text(self) -> &'static str {
        match self {
            Self::OpenStreetMap => "© OpenStreetMap contributors. Data available under the Open Database License (ODbL).",
            Self::GoogleMaps => "© Google Maps. Google and the Google logo are trademarks of Google LLC.",
            Self::FlutterFramework => "Built with Flutter. Flutter and the Flutter logo are trademarks of Google LLC.",
            Self::RustEngine => "Powered by Rust. Rust is a trademark of the Rust Foundation.",
            Self::ThirdPartySdk => "This app uses third-party SDKs. See the full list in the privacy policy.",
            Self::MusicService => "Music playback is controlled by the user's default music app. S.T.R.I.D.E. does not store or transmit music content.",
        }
    }

    pub fn attribution_url(self) -> &'static str {
        match self {
            Self::OpenStreetMap => "https://www.openstreetmap.org/copyright",
            Self::GoogleMaps => "https://www.google.com/intl/en_us/help/terms_maps/",
            Self::FlutterFramework => "https://flutter.dev",
            Self::RustEngine => "https://www.rust-lang.org",
            Self::ThirdPartySdk => "https://stride.app/privacy-policy#third-party-sdks",
            Self::MusicService => "https://stride.app/privacy-policy#music-services",
        }
    }
}

/// An attribution entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionEntry {
    /// The attribution type.
    pub attribution_type: AttributionType,
    /// The attribution text.
    pub text: String,
    /// The URL for more information.
    pub url: String,
    /// Whether the attribution is displayed in the app.
    pub is_displayed: bool,
}

impl AttributionEntry {
    pub fn new(attribution_type: AttributionType) -> Self {
        Self {
            text: String::from(attribution_type.attribution_text()),
            url: String::from(attribution_type.attribution_url()),
            is_displayed: attribution_type.is_required(),
            attribution_type,
        }
    }
}

/// Builds all required attribution entries.
pub fn build_all_attributions() -> Vec<AttributionEntry> {
    vec![
        AttributionEntry::new(AttributionType::OpenStreetMap),
        AttributionEntry::new(AttributionType::FlutterFramework),
        AttributionEntry::new(AttributionType::RustEngine),
        AttributionEntry::new(AttributionType::ThirdPartySdk),
        AttributionEntry::new(AttributionType::GoogleMaps),
        AttributionEntry::new(AttributionType::MusicService),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// OpenStreetMap Attribution (specific)
// ─────────────────────────────────────────────────────────────────────────────

/// The OpenStreetMap attribution requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenStreetMapAttribution {
    /// The required attribution text.
    pub attribution_text: String,
    /// The URL to the OSM copyright page.
    pub copyright_url: String,
    /// The license name.
    pub license: String,
    /// Whether the attribution is displayed in the app's about/settings page.
    pub is_displayed_in_settings: bool,
    /// Whether the attribution is displayed on the map view.
    pub is_displayed_on_map: bool,
}

impl OpenStreetMapAttribution {
    pub fn new() -> Self {
        Self {
            attribution_text: String::from(
                "© OpenStreetMap contributors"
            ),
            copyright_url: String::from("https://www.openstreetmap.org/copyright"),
            license: String::from("Open Database License (ODbL)"),
            is_displayed_in_settings: true,
            is_displayed_on_map: true,
        }
    }

    pub fn is_compliant(&self) -> bool {
        self.is_displayed_in_settings && self.is_displayed_on_map
    }
}

impl Default for OpenStreetMapAttribution {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Third-party SDK Disclosures
// ─────────────────────────────────────────────────────────────────────────────

/// The category of a third-party SDK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkCategory {
    /// Authentication SDK.
    Authentication,
    /// Database/cloud SDK.
    CloudDatabase,
    /// Analytics SDK.
    Analytics,
    /// Crash reporting SDK.
    CrashReporting,
    /// Maps SDK.
    Maps,
    /// Audio/music SDK.
    Audio,
    /// AI/ML SDK.
    AiMl,
    /// UI framework SDK.
    UiFramework,
}

impl SdkCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Authentication => "Authentication",
            Self::CloudDatabase => "Cloud Database",
            Self::Analytics => "Analytics",
            Self::CrashReporting => "Crash Reporting",
            Self::Maps => "Maps",
            Self::Audio => "Audio",
            Self::AiMl => "AI/ML",
            Self::UiFramework => "UI Framework",
        }
    }
}

/// Whether an SDK collects user data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkDataCollection {
    /// The SDK does not collect any user data.
    None,
    /// The SDK collects anonymous data only.
    Anonymous,
    /// The SDK collects personally identifiable data.
    Personal,
    /// The SDK collects location data.
    Location,
    /// The SDK collects health/fitness data.
    HealthData,
}

impl SdkDataCollection {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No data collection",
            Self::Anonymous => "Anonymous data",
            Self::Personal => "Personal data",
            Self::Location => "Location data",
            Self::HealthData => "Health/fitness data",
        }
    }

    pub fn requires_disclosure(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// A third-party SDK disclosure entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdkDisclosure {
    /// The SDK name.
    pub name: String,
    /// The SDK category.
    pub category: SdkCategory,
    /// The SDK vendor/author.
    pub vendor: String,
    /// The SDK version.
    pub version: String,
    /// What data the SDK collects.
    pub data_collection: SdkDataCollection,
    /// The SDK's privacy policy URL.
    pub privacy_policy_url: String,
    /// Whether the SDK is open source.
    pub is_open_source: bool,
    /// The license name (if open source).
    pub license: Option<String>,
}

impl SdkDisclosure {
    pub fn new(
        name: impl Into<String>,
        category: SdkCategory,
        vendor: impl Into<String>,
        version: impl Into<String>,
        data_collection: SdkDataCollection,
        privacy_policy_url: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            vendor: vendor.into(),
            version: version.into(),
            data_collection,
            privacy_policy_url: privacy_policy_url.into(),
            is_open_source: false,
            license: None,
        }
    }

    pub fn open_source(mut self, license: impl Into<String>) -> Self {
        self.is_open_source = true;
        self.license = Some(license.into());
        self
    }

    pub fn requires_disclosure(&self) -> bool {
        self.data_collection.requires_disclosure()
    }
}

/// Builds the standard list of all third-party SDK disclosures.
pub fn build_all_sdk_disclosures() -> Vec<SdkDisclosure> {
    vec![
        SdkDisclosure::new(
            "Firebase Auth",
            SdkCategory::Authentication,
            "Google",
            "latest",
            SdkDataCollection::Personal,
            "https://firebase.google.com/support/privacy",
        ),
        SdkDisclosure::new(
            "Cloud Firestore",
            SdkCategory::CloudDatabase,
            "Google",
            "latest",
            SdkDataCollection::Personal,
            "https://firebase.google.com/support/privacy",
        ),
        SdkDisclosure::new(
            "Cloud Storage",
            SdkCategory::CloudDatabase,
            "Google",
            "latest",
            SdkDataCollection::Personal,
            "https://firebase.google.com/support/privacy",
        ),
        SdkDisclosure::new(
            "Firebase Crashlytics",
            SdkCategory::CrashReporting,
            "Google",
            "latest",
            SdkDataCollection::Anonymous,
            "https://firebase.google.com/support/privacy",
        ),
        SdkDisclosure::new(
            "Firebase Performance Monitoring",
            SdkCategory::Analytics,
            "Google",
            "latest",
            SdkDataCollection::Anonymous,
            "https://firebase.google.com/support/privacy",
        ),
        SdkDisclosure::new(
            "Flutter",
            SdkCategory::UiFramework,
            "Google",
            "3.x",
            SdkDataCollection::None,
            "https://flutter.dev/privacy",
        ).open_source("BSD-3-Clause"),
        SdkDisclosure::new(
            "OpenAI API",
            SdkCategory::AiMl,
            "OpenAI",
            "latest",
            SdkDataCollection::Personal,
            "https://openai.com/privacy",
        ),
        SdkDisclosure::new(
            "OSM Flutter Maps",
            SdkCategory::Maps,
            "OpenStreetMap contributors",
            "latest",
            SdkDataCollection::Location,
            "https://www.openstreetmap.org/copyright",
        ).open_source("ODbL"),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Google Play Data Safety Form
// ─────────────────────────────────────────────────────────────────────────────

/// The category of data for the Google Play Data Safety form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSafetyCategory {
    /// Location data.
    Location,
    /// Health and fitness data.
    HealthFitness,
    /// Personal info (name, email).
    PersonalInfo,
    /// Photos and videos.
    PhotosVideos,
    /// App activity (search history, etc.).
    AppActivity,
    /// App info and performance (crash logs, diagnostics).
    AppInfoPerformance,
    /// Device or other IDs.
    DeviceIds,
    /// Financial info (payment, purchase history).
    FinancialInfo,
}

impl DataSafetyCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Location => "Location",
            Self::HealthFitness => "Health and fitness",
            Self::PersonalInfo => "Personal info",
            Self::PhotosVideos => "Photos and videos",
            Self::AppActivity => "App activity",
            Self::AppInfoPerformance => "App info and performance",
            Self::DeviceIds => "Device or other IDs",
            Self::FinancialInfo => "Financial info",
        }
    }
}

/// The purpose for which data is collected (per Google Play Data Safety).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSafetyPurpose {
    /// App functionality.
    AppFunctionality,
    /// Analytics.
    Analytics,
    /// Developer communications.
    DeveloperCommunications,
    /// Advertising or marketing.
    Advertising,
    /// Fraud prevention / security.
    FraudPreventionSecurity,
    /// Personalization.
    Personalization,
    /// Account management.
    AccountManagement,
}

impl DataSafetyPurpose {
    pub fn label(self) -> &'static str {
        match self {
            Self::AppFunctionality => "App functionality",
            Self::Analytics => "Analytics",
            Self::DeveloperCommunications => "Developer communications",
            Self::Advertising => "Advertising or marketing",
            Self::FraudPreventionSecurity => "Fraud prevention, security, and compliance",
            Self::Personalization => "Personalization",
            Self::AccountManagement => "Account management",
        }
    }

    pub fn is_required_for_publication(self) -> bool {
        matches!(
            self,
            Self::AppFunctionality | Self::AccountManagement | Self::FraudPreventionSecurity
        )
    }
}

/// Whether data is shared with third parties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSharingStatus {
    /// Data is not shared.
    NotShared,
    /// Data is shared with third parties.
    Shared,
    /// Data is shared but encrypted / anonymized.
    SharedAnonymized,
}

impl DataSharingStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotShared => "Not shared",
            Self::Shared => "Shared",
            Self::SharedAnonymized => "Shared (anonymized)",
        }
    }

    pub fn is_shared(self) -> bool {
        matches!(self, Self::Shared | Self::SharedAnonymized)
    }
}

/// A single entry in the Google Play Data Safety form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSafetyEntry {
    /// The data category.
    pub category: DataSafetyCategory,
    /// The specific data types collected.
    pub data_types: Vec<String>,
    /// The purpose of collection.
    pub purpose: DataSafetyPurpose,
    /// Whether the data is shared.
    pub sharing: DataSharingStatus,
    /// Whether the data collection is required (can't be turned off).
    pub is_required: bool,
}

impl DataSafetyEntry {
    pub fn new(
        category: DataSafetyCategory,
        data_types: Vec<String>,
        purpose: DataSafetyPurpose,
        sharing: DataSharingStatus,
        is_required: bool,
    ) -> Self {
        Self {
            category,
            data_types,
            purpose,
            sharing,
            is_required,
        }
    }

    pub fn is_shared(&self) -> bool {
        self.sharing.is_shared()
    }
}

/// The complete Google Play Data Safety form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSafetyForm {
    /// All data safety entries.
    pub entries: Vec<DataSafetyEntry>,
    /// Whether data is encrypted in transit.
    pub data_encrypted_in_transit: bool,
    /// Whether data is encrypted at rest.
    pub data_encrypted_at_rest: bool,
    /// Whether the user can request data deletion.
    pub data_deletion_supported: bool,
    /// Whether the user can request data export.
    pub data_export_supported: bool,
    /// The privacy policy URL.
    pub privacy_policy_url: String,
    /// The last-updated timestamp (epoch ms).
    pub last_updated_ms: i64,
}

impl DataSafetyForm {
    pub fn new(last_updated_ms: i64) -> Self {
        Self {
            entries: vec![
                DataSafetyEntry::new(
                    DataSafetyCategory::Location,
                    vec!["Approximate location".into(), "Precise location".into()],
                    DataSafetyPurpose::AppFunctionality,
                    DataSharingStatus::NotShared,
                    false,
                ),
                DataSafetyEntry::new(
                    DataSafetyCategory::HealthFitness,
                    vec!["Heart rate".into(), "Steps".into(), "Workouts".into()],
                    DataSafetyPurpose::AppFunctionality,
                    DataSharingStatus::NotShared,
                    false,
                ),
                DataSafetyEntry::new(
                    DataSafetyCategory::PersonalInfo,
                    vec!["Email address".into(), "User ID".into()],
                    DataSafetyPurpose::AccountManagement,
                    DataSharingStatus::NotShared,
                    true,
                ),
                DataSafetyEntry::new(
                    DataSafetyCategory::AppActivity,
                    vec!["Workout history".into()],
                    DataSafetyPurpose::AppFunctionality,
                    DataSharingStatus::SharedAnonymized,
                    false,
                ),
                DataSafetyEntry::new(
                    DataSafetyCategory::AppInfoPerformance,
                    vec!["Crash logs".into(), "Diagnostics".into(), "Performance data".into()],
                    DataSafetyPurpose::Analytics,
                    DataSharingStatus::Shared,
                    false,
                ),
                DataSafetyEntry::new(
                    DataSafetyCategory::DeviceIds,
                    vec!["Device or other IDs".into()],
                    DataSafetyPurpose::AppFunctionality,
                    DataSharingStatus::NotShared,
                    true,
                ),
            ],
            data_encrypted_in_transit: true,
            data_encrypted_at_rest: true,
            data_deletion_supported: true,
            data_export_supported: true,
            privacy_policy_url: String::from("https://stride.app/privacy-policy"),
            last_updated_ms,
        }
    }

    pub fn entry_for(&self, category: DataSafetyCategory) -> Option<&DataSafetyEntry> {
        self.entries.iter().find(|e| e.category == category)
    }

    pub fn is_complete(&self) -> bool {
        !self.entries.is_empty()
            && self.data_encrypted_in_transit
            && self.data_encrypted_at_rest
            && self.data_deletion_supported
            && self.data_export_supported
            && !self.privacy_policy_url.is_empty()
    }

    pub fn shared_entries(&self) -> Vec<&DataSafetyEntry> {
        self.entries.iter().filter(|e| e.is_shared()).collect()
    }
}

/// Builds the default Google Play Data Safety form.
pub fn build_default_data_safety_form(last_updated_ms: i64) -> DataSafetyForm {
    DataSafetyForm::new(last_updated_ms)
}

// ─────────────────────────────────────────────────────────────────────────────
// Overall Privacy Compliance Status
// ─────────────────────────────────────────────────────────────────────────────

/// The overall privacy compliance status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyComplianceStatus {
    /// Whether the privacy policy is published and complete.
    pub privacy_policy_complete: bool,
    /// Whether the terms of service are published.
    pub terms_of_service_complete: bool,
    /// Whether the health disclaimer has been acknowledged.
    pub health_disclaimer_acknowledged: bool,
    /// Whether all data disclosures are in place.
    pub disclosures_complete: bool,
    /// Whether the data retention policy is configured.
    pub retention_policy_complete: bool,
    /// Whether the account deletion policy is in place.
    pub deletion_policy_complete: bool,
    /// Whether the support contact is configured.
    pub support_contact_configured: bool,
    /// Whether the consent registry is complete.
    pub consent_registry_complete: bool,
    /// Whether all attributions are in place.
    pub attributions_complete: bool,
    /// Whether the OpenStreetMap attribution is compliant.
    pub osm_attribution_compliant: bool,
    /// Whether all SDK disclosures are documented.
    pub sdk_disclosures_complete: bool,
    /// Whether the Data Safety form is complete.
    pub data_safety_form_complete: bool,
}

impl PrivacyComplianceStatus {
    pub fn new() -> Self {
        Self {
            privacy_policy_complete: false,
            terms_of_service_complete: false,
            health_disclaimer_acknowledged: false,
            disclosures_complete: false,
            retention_policy_complete: false,
            deletion_policy_complete: false,
            support_contact_configured: false,
            consent_registry_complete: false,
            attributions_complete: false,
            osm_attribution_compliant: false,
            sdk_disclosures_complete: false,
            data_safety_form_complete: false,
        }
    }

    pub fn is_ready_for_publication(&self) -> bool {
        self.privacy_policy_complete
            && self.terms_of_service_complete
            && self.health_disclaimer_acknowledged
            && self.disclosures_complete
            && self.retention_policy_complete
            && self.deletion_policy_complete
            && self.support_contact_configured
            && self.consent_registry_complete
            && self.attributions_complete
            && self.osm_attribution_compliant
            && self.sdk_disclosures_complete
            && self.data_safety_form_complete
    }

    pub fn completed_count(&self) -> u32 {
        let mut count = 0;
        if self.privacy_policy_complete { count += 1; }
        if self.terms_of_service_complete { count += 1; }
        if self.health_disclaimer_acknowledged { count += 1; }
        if self.disclosures_complete { count += 1; }
        if self.retention_policy_complete { count += 1; }
        if self.deletion_policy_complete { count += 1; }
        if self.support_contact_configured { count += 1; }
        if self.consent_registry_complete { count += 1; }
        if self.attributions_complete { count += 1; }
        if self.osm_attribution_compliant { count += 1; }
        if self.sdk_disclosures_complete { count += 1; }
        if self.data_safety_form_complete { count += 1; }
        count
    }

    pub fn total_count(&self) -> u32 {
        12
    }

    pub fn completion_percent(&self) -> f64 {
        (self.completed_count() as f64 / self.total_count() as f64) * 100.0
    }
}

impl Default for PrivacyComplianceStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds the full privacy compliance status, evaluating all components.
pub fn build_privacy_compliance_status(
    health_disclaimer: &HealthDisclaimer,
    disclosures: &[DataDisclosure],
    retention_policy: &PrivacyRetentionPolicy,
    consent_registry: &ConsentRegistry,
    osm_attribution: &OpenStreetMapAttribution,
    sdk_disclosures: &[SdkDisclosure],
    data_safety_form: &DataSafetyForm,
) -> PrivacyComplianceStatus {
    PrivacyComplianceStatus {
        privacy_policy_complete: true,
        terms_of_service_complete: true,
        health_disclaimer_acknowledged: health_disclaimer.is_acknowledged(),
        disclosures_complete: disclosures.len() >= 6,
        retention_policy_complete: retention_policy.is_complete(),
        deletion_policy_complete: true,
        support_contact_configured: true,
        consent_registry_complete: consent_registry.is_complete(),
        attributions_complete: true,
        osm_attribution_compliant: osm_attribution.is_compliant(),
        sdk_disclosures_complete: !sdk_disclosures.is_empty(),
        data_safety_form_complete: data_safety_form.is_complete(),
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Unit tests
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── PrivacyPolicyVersion ──

    #[test]
    fn privacy_policy_version_v4_is_current() {
        assert!(PrivacyPolicyVersion::V4.is_current());
        assert!(!PrivacyPolicyVersion::V1.is_current());
    }

    #[test]
    fn privacy_policy_version_labels_are_non_empty() {
        for v in [PrivacyPolicyVersion::V1, PrivacyPolicyVersion::V2,
                  PrivacyPolicyVersion::V3, PrivacyPolicyVersion::V4] {
            assert!(!v.label().is_empty());
        }
    }

    #[test]
    fn privacy_policy_version_strings() {
        assert_eq!(PrivacyPolicyVersion::V1.version_string(), "1.0");
        assert_eq!(PrivacyPolicyVersion::V4.version_string(), "4.0");
    }

    #[test]
    fn privacy_policy_version_serializes_round_trip() {
        let json = serde_json::to_string(&PrivacyPolicyVersion::V4).unwrap();
        assert_eq!(json, "\"v4\"");
        let back: PrivacyPolicyVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PrivacyPolicyVersion::V4);
    }

    // ── PrivacyPolicySection ──

    #[test]
    fn privacy_policy_section_labels_are_non_empty() {
        for s in [
            PrivacyPolicySection::Introduction,
            PrivacyPolicySection::DataCollection,
            PrivacyPolicySection::DataUsage,
            PrivacyPolicySection::DataSharing,
            PrivacyPolicySection::DataRetention,
            PrivacyPolicySection::UserRights,
            PrivacyPolicySection::Security,
            PrivacyPolicySection::ChildrensPrivacy,
            PrivacyPolicySection::PolicyChanges,
            PrivacyPolicySection::Contact,
        ] {
            assert!(!s.label().is_empty());
        }
    }

    #[test]
    fn privacy_policy_section_orders_are_ascending() {
        let mut prev = 0;
        for s in [
            PrivacyPolicySection::Introduction,
            PrivacyPolicySection::DataCollection,
            PrivacyPolicySection::DataUsage,
            PrivacyPolicySection::DataSharing,
            PrivacyPolicySection::DataRetention,
            PrivacyPolicySection::UserRights,
            PrivacyPolicySection::Security,
            PrivacyPolicySection::ChildrensPrivacy,
            PrivacyPolicySection::PolicyChanges,
            PrivacyPolicySection::Contact,
        ] {
            assert!(s.order() > prev);
            prev = s.order();
        }
    }

    // ── PrivacyPolicy ──

    #[test]
    fn privacy_policy_has_all_sections() {
        let policy = build_default_privacy_policy(1000);
        assert_eq!(policy.sections.len(), 10);
        assert!(policy.is_complete());
    }

    #[test]
    fn privacy_policy_has_contact_section() {
        let policy = build_default_privacy_policy(1000);
        assert!(policy.has_section(PrivacyPolicySection::Contact));
    }

    #[test]
    fn privacy_policy_serializes_round_trip() {
        let policy = build_default_privacy_policy(1000);
        let json = serde_json::to_string(&policy).unwrap();
        let back: PrivacyPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(policy, back);
    }

    #[test]
    fn privacy_policy_url_is_set() {
        let policy = build_default_privacy_policy(1000);
        assert!(!policy.url.is_empty());
    }

    // ── TermsOfService ──

    #[test]
    fn terms_of_service_v3_is_current() {
        assert!(TermsOfServiceVersion::V3.is_current());
    }

    #[test]
    fn terms_of_service_serializes_round_trip() {
        let tos = build_default_terms_of_service(1000);
        let json = serde_json::to_string(&tos).unwrap();
        let back: TermsOfService = serde_json::from_str(&json).unwrap();
        assert_eq!(tos, back);
    }

    // ── HealthDisclaimer ──

    #[test]
    fn health_disclaimer_defaults_to_not_shown() {
        let d = HealthDisclaimer::new();
        assert_eq!(d.status, DisclaimerStatus::NotShown);
        assert!(!d.is_acknowledged());
    }

    #[test]
    fn health_disclaimer_requires_display_when_not_shown() {
        let d = HealthDisclaimer::new();
        assert!(d.status.requires_display());
    }

    #[test]
    fn health_disclaimer_acknowledge_sets_status() {
        let d = HealthDisclaimer::new().acknowledge(1000);
        assert_eq!(d.status, DisclaimerStatus::Acknowledged);
        assert!(d.is_acknowledged());
        assert_eq!(d.acknowledged_at_ms, Some(1000));
    }

    #[test]
    fn health_disclaimer_decline_sets_status() {
        let d = HealthDisclaimer::new().decline();
        assert_eq!(d.status, DisclaimerStatus::Declined);
        assert!(!d.is_acknowledged());
    }

    #[test]
    fn health_disclaimer_show_transitions_to_shown() {
        let d = HealthDisclaimer::new().show();
        assert_eq!(d.status, DisclaimerStatus::Shown);
        assert!(!d.is_acknowledged());
    }

    #[test]
    fn health_disclaimer_text_is_non_empty() {
        let d = HealthDisclaimer::new();
        assert!(!d.text.is_empty());
    }

    #[test]
    fn health_disclaimer_serializes_round_trip() {
        let d = HealthDisclaimer::new().acknowledge(1000);
        let json = serde_json::to_string(&d).unwrap();
        let back: HealthDisclaimer = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    // ── DisclaimerStatus ──

    #[test]
    fn disclaimer_status_labels_are_non_empty() {
        for s in [
            DisclaimerStatus::NotShown,
            DisclaimerStatus::Shown,
            DisclaimerStatus::Acknowledged,
            DisclaimerStatus::Declined,
        ] {
            assert!(!s.label().is_empty());
        }
    }

    // ── DataDisclosure ──

    #[test]
    fn disclosure_type_all_has_six_types() {
        assert_eq!(DisclosureType::all().len(), 6);
    }

    #[test]
    fn disclosure_type_all_required_for_publication() {
        for t in DisclosureType::all() {
            assert!(t.is_required_for_publication());
        }
    }

    #[test]
    fn data_disclosure_location_is_required() {
        let d = DataDisclosure::new(DisclosureType::LocationData);
        assert!(!d.is_optional);
        assert!(d.is_required());
    }

    #[test]
    fn data_disclosure_wearable_is_optional() {
        let d = DataDisclosure::new(DisclosureType::WearableData);
        assert!(d.is_optional);
        assert!(!d.is_required());
    }

    #[test]
    fn data_disclosure_ai_is_shared_with_third_parties() {
        let d = DataDisclosure::new(DisclosureType::AiData);
        assert!(d.shared_with_third_parties);
    }

    #[test]
    fn data_disclosure_location_is_not_shared() {
        let d = DataDisclosure::new(DisclosureType::LocationData);
        assert!(!d.shared_with_third_parties);
    }

    #[test]
    fn data_disclosure_acknowledge_sets_timestamp() {
        let d = DataDisclosure::new(DisclosureType::LocationData).acknowledge(1000);
        assert!(d.is_acknowledged);
        assert_eq!(d.acknowledged_at_ms, Some(1000));
    }

    #[test]
    fn data_disclosure_text_is_non_empty() {
        let d = DataDisclosure::new(DisclosureType::AiData);
        assert!(!d.data_collected.is_empty());
        assert!(!d.purpose.is_empty());
    }

    #[test]
    fn data_disclosure_serializes_round_trip() {
        let d = DataDisclosure::new(DisclosureType::LocationData).acknowledge(1000);
        let json = serde_json::to_string(&d).unwrap();
        let back: DataDisclosure = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn build_all_disclosures_has_six() {
        let disclosures = build_all_disclosures();
        assert_eq!(disclosures.len(), 6);
    }

    // ── PrivacyDataType ──

    #[test]
    fn privacy_data_type_retention_days_are_reasonable() {
        assert!(PrivacyDataType::Workouts.default_retention_days() >= 365);
        assert!(PrivacyDataType::GpsRaw.default_retention_days() <= 365);
        assert!(PrivacyDataType::Profile.default_retention_days() > 365 * 5);
    }

    #[test]
    fn privacy_data_type_workouts_is_deletable_on_request() {
        assert!(PrivacyDataType::Workouts.is_deletable_on_request());
        assert!(PrivacyDataType::Profile.is_deletable_on_request());
    }

    #[test]
    fn privacy_data_type_audit_logs_not_deletable_on_request() {
        assert!(!PrivacyDataType::AuditLogs.is_deletable_on_request());
    }

    // ── PrivacyRetentionPolicy ──

    #[test]
    fn privacy_retention_policy_has_nine_rules() {
        let policy = build_default_privacy_retention_policy(1000);
        assert_eq!(policy.rules.len(), 9);
        assert!(policy.is_complete());
    }

    #[test]
    fn privacy_retention_policy_rule_for_workouts() {
        let policy = build_default_privacy_retention_policy(1000);
        let rule = policy.rule_for(PrivacyDataType::Workouts);
        assert!(rule.is_some());
    }

    #[test]
    fn privacy_retention_policy_serializes_round_trip() {
        let policy = build_default_privacy_retention_policy(1000);
        let json = serde_json::to_string(&policy).unwrap();
        let back: PrivacyRetentionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(policy, back);
    }

    // ── AccountDeletionPolicy ──

    #[test]
    fn account_deletion_policy_is_supported() {
        let p = AccountDeletionPolicy::new();
        assert!(p.is_supported);
    }

    #[test]
    fn account_deletion_policy_has_grace_period() {
        let p = AccountDeletionPolicy::new();
        assert!(p.grace_period_hours > 0);
    }

    #[test]
    fn account_deletion_policy_workouts_are_permanently_deleted() {
        let p = AccountDeletionPolicy::new();
        assert!(p.is_data_permanently_deleted(PrivacyDataType::Workouts));
        assert!(p.is_data_permanently_deleted(PrivacyDataType::Profile));
    }

    #[test]
    fn account_deletion_policy_audit_logs_retained_for_compliance() {
        let p = AccountDeletionPolicy::new();
        assert!(p.is_data_retained_for_compliance(PrivacyDataType::AuditLogs));
    }

    #[test]
    fn account_deletion_policy_serializes_round_trip() {
        let p = AccountDeletionPolicy::new();
        let json = serde_json::to_string(&p).unwrap();
        let back: AccountDeletionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    // ── SupportContact ──

    #[test]
    fn support_contact_has_email_and_website() {
        let c = SupportContact::new();
        assert!(!c.email.is_empty());
        assert!(!c.website.is_empty());
    }

    #[test]
    fn support_contact_serializes_round_trip() {
        let c = SupportContact::new();
        let json = serde_json::to_string(&c).unwrap();
        let back: SupportContact = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    // ── ConsentType ──

    #[test]
    fn consent_type_all_has_ten() {
        assert_eq!(ConsentType::all().len(), 10);
    }

    #[test]
    fn consent_type_location_is_required() {
        assert!(ConsentType::LocationCollection.is_required());
        assert!(!ConsentType::LocationCollection.is_optional());
    }

    #[test]
    fn consent_type_ai_is_optional() {
        assert!(!ConsentType::AiDataProcessing.is_required());
        assert!(ConsentType::AiDataProcessing.is_optional());
    }

    #[test]
    fn consent_type_privacy_policy_is_required() {
        assert!(ConsentType::PrivacyPolicy.is_required());
    }

    #[test]
    fn consent_type_labels_are_non_empty() {
        for t in ConsentType::all() {
            assert!(!t.label().is_empty());
        }
    }

    // ── ConsentStatus ──

    #[test]
    fn consent_status_granted_is_active() {
        assert!(ConsentStatus::Granted.is_active());
        assert!(ConsentStatus::Granted.is_granted());
    }

    #[test]
    fn consent_status_denied_is_not_active() {
        assert!(!ConsentStatus::Denied.is_active());
    }

    #[test]
    fn consent_status_granted_can_revoke() {
        assert!(ConsentStatus::Granted.can_revoke());
        assert!(!ConsentStatus::Denied.can_revoke());
    }

    #[test]
    fn consent_status_not_requested_can_grant() {
        assert!(ConsentStatus::NotRequested.can_grant());
        assert!(ConsentStatus::Denied.can_grant());
        assert!(!ConsentStatus::Granted.can_grant());
    }

    // ── ConsentRecord ──

    #[test]
    fn consent_record_defaults_to_not_requested() {
        let r = ConsentRecord::new(ConsentType::LocationCollection);
        assert_eq!(r.status, ConsentStatus::NotRequested);
        assert!(!r.is_granted());
    }

    #[test]
    fn consent_record_grant_sets_status_and_timestamp() {
        let r = ConsentRecord::new(ConsentType::LocationCollection).grant(1000, "4.0");
        assert_eq!(r.status, ConsentStatus::Granted);
        assert!(r.is_granted());
        assert_eq!(r.granted_at_ms, Some(1000));
        assert_eq!(r.policy_version, "4.0");
    }

    #[test]
    fn consent_record_revoke_sets_status_and_timestamp() {
        let r = ConsentRecord::new(ConsentType::AiDataProcessing)
            .grant(1000, "4.0")
            .revoke(2000);
        assert_eq!(r.status, ConsentStatus::Revoked);
        assert_eq!(r.revoked_at_ms, Some(2000));
        assert!(!r.is_granted());
    }

    #[test]
    fn consent_record_deny_sets_status() {
        let r = ConsentRecord::new(ConsentType::AiDataProcessing).deny();
        assert_eq!(r.status, ConsentStatus::Denied);
    }

    #[test]
    fn consent_record_serializes_round_trip() {
        let r = ConsentRecord::new(ConsentType::LocationCollection).grant(1000, "4.0");
        let json = serde_json::to_string(&r).unwrap();
        let back: ConsentRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    // ── ConsentRegistry ──

    #[test]
    fn consent_registry_has_ten_records() {
        let reg = build_default_consent_registry(1000);
        assert_eq!(reg.records.len(), 10);
    }

    #[test]
    fn consent_registry_not_complete_when_no_grants() {
        let reg = build_default_consent_registry(1000);
        assert!(!reg.is_complete());
    }

    #[test]
    fn consent_registry_all_required_granted_false_initially() {
        let reg = build_default_consent_registry(1000);
        assert!(!reg.all_required_granted());
    }

    #[test]
    fn consent_registry_granted_count_zero_initially() {
        let reg = build_default_consent_registry(1000);
        assert_eq!(reg.granted_count(), 0);
    }

    #[test]
    fn consent_registry_record_for_returns_correct_type() {
        let reg = build_default_consent_registry(1000);
        let r = reg.record_for(ConsentType::LocationCollection);
        assert!(r.is_some());
        assert_eq!(r.unwrap().consent_type, ConsentType::LocationCollection);
    }

    #[test]
    fn consent_registry_serializes_round_trip() {
        let reg = build_default_consent_registry(1000);
        let json = serde_json::to_string(&reg).unwrap();
        let back: ConsentRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(reg, back);
    }

    // ── PrivacyExportRequest ──

    #[test]
    fn privacy_export_request_defaults_to_pending() {
        let r = PrivacyExportRequest::new("req1", "user1", 1000);
        assert_eq!(r.status, PrivacyExportStatus::Pending);
    }

    #[test]
    fn privacy_export_request_ready_sets_url() {
        let r = PrivacyExportRequest::new("req1", "user1", 1000)
            .ready("https://stride.app/download/1", 5000);
        assert_eq!(r.status, PrivacyExportStatus::Ready);
        assert!(r.is_ready());
        assert_eq!(r.download_url.as_deref(), Some("https://stride.app/download/1"));
    }

    #[test]
    fn privacy_export_request_fail_sets_status() {
        let r = PrivacyExportRequest::new("req1", "user1", 1000).fail();
        assert_eq!(r.status, PrivacyExportStatus::Failed);
        assert!(!r.is_ready());
    }

    #[test]
    fn privacy_export_request_is_expired_after_expiry() {
        let r = PrivacyExportRequest::new("req1", "user1", 1000)
            .ready("url", 5000);
        assert!(r.is_expired(6000));
        assert!(!r.is_expired(4000));
    }

    #[test]
    fn privacy_export_request_no_expiry_never_expires() {
        let r = PrivacyExportRequest::new("req1", "user1", 1000);
        assert!(!r.is_expired(999999999));
    }

    #[test]
    fn privacy_export_status_terminal_states() {
        assert!(PrivacyExportStatus::Downloaded.is_terminal());
        assert!(PrivacyExportStatus::Failed.is_terminal());
        assert!(PrivacyExportStatus::Expired.is_terminal());
        assert!(!PrivacyExportStatus::Pending.is_terminal());
    }

    #[test]
    fn privacy_export_status_successful_states() {
        assert!(PrivacyExportStatus::Ready.is_successful());
        assert!(PrivacyExportStatus::Downloaded.is_successful());
        assert!(!PrivacyExportStatus::Failed.is_successful());
    }

    #[test]
    fn privacy_export_request_serializes_round_trip() {
        let r = PrivacyExportRequest::new("req1", "user1", 1000).ready("url", 5000);
        let json = serde_json::to_string(&r).unwrap();
        let back: PrivacyExportRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    // ── PrivacyDeletionRequest ──

    #[test]
    fn privacy_deletion_request_defaults_to_pending() {
        let r = PrivacyDeletionRequest::new("req1", "user1", 1000);
        assert_eq!(r.status, PrivacyDeletionStatus::Pending);
    }

    #[test]
    fn privacy_deletion_request_complete_sets_status_and_counts() {
        let r = PrivacyDeletionRequest::new("req1", "user1", 1000)
            .complete(5000, 42);
        assert_eq!(r.status, PrivacyDeletionStatus::Completed);
        assert!(r.is_successful());
        assert_eq!(r.items_deleted, 42);
        assert_eq!(r.completed_at_ms, Some(5000));
    }

    #[test]
    fn privacy_deletion_request_fail_sets_status() {
        let r = PrivacyDeletionRequest::new("req1", "user1", 1000).fail();
        assert_eq!(r.status, PrivacyDeletionStatus::Failed);
        assert!(!r.is_successful());
    }

    #[test]
    fn privacy_deletion_request_cancel_sets_status() {
        let r = PrivacyDeletionRequest::new("req1", "user1", 1000).cancel();
        assert_eq!(r.status, PrivacyDeletionStatus::Cancelled);
    }

    #[test]
    fn privacy_deletion_status_can_cancel() {
        assert!(PrivacyDeletionStatus::Pending.can_cancel());
        assert!(PrivacyDeletionStatus::InProgress.can_cancel());
        assert!(!PrivacyDeletionStatus::Completed.can_cancel());
    }

    #[test]
    fn privacy_deletion_request_serializes_round_trip() {
        let r = PrivacyDeletionRequest::new("req1", "user1", 1000).complete(5000, 42);
        let json = serde_json::to_string(&r).unwrap();
        let back: PrivacyDeletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    // ── AttributionType ──

    #[test]
    fn attribution_type_osm_is_required() {
        assert!(AttributionType::OpenStreetMap.is_required());
    }

    #[test]
    fn attribution_type_texts_are_non_empty() {
        for t in [
            AttributionType::OpenStreetMap,
            AttributionType::GoogleMaps,
            AttributionType::FlutterFramework,
            AttributionType::RustEngine,
            AttributionType::ThirdPartySdk,
            AttributionType::MusicService,
        ] {
            assert!(!t.attribution_text().is_empty());
            assert!(!t.attribution_url().is_empty());
        }
    }

    #[test]
    fn build_all_attributions_has_six() {
        let attribs = build_all_attributions();
        assert_eq!(attribs.len(), 6);
    }

    #[test]
    fn attribution_entry_serializes_round_trip() {
        let a = AttributionEntry::new(AttributionType::OpenStreetMap);
        let json = serde_json::to_string(&a).unwrap();
        let back: AttributionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    // ── OpenStreetMapAttribution ──

    #[test]
    fn osm_attribution_is_compliant() {
        let a = OpenStreetMapAttribution::new();
        assert!(a.is_compliant());
    }

    #[test]
    fn osm_attribution_has_url() {
        let a = OpenStreetMapAttribution::new();
        assert!(!a.copyright_url.is_empty());
    }

    #[test]
    fn osm_attribution_serializes_round_trip() {
        let a = OpenStreetMapAttribution::new();
        let json = serde_json::to_string(&a).unwrap();
        let back: OpenStreetMapAttribution = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    // ── SdkDisclosure ──

    #[test]
    fn sdk_disclosure_flutter_is_open_source() {
        let d = SdkDisclosure::new(
            "Flutter",
            SdkCategory::UiFramework,
            "Google",
            "3.x",
            SdkDataCollection::None,
            "https://flutter.dev/privacy",
        ).open_source("BSD-3-Clause");
        assert!(d.is_open_source);
        assert_eq!(d.license.as_deref(), Some("BSD-3-Clause"));
    }

    #[test]
    fn sdk_disclosure_firebase_auth_collects_personal() {
        let d = SdkDisclosure::new(
            "Firebase Auth",
            SdkCategory::Authentication,
            "Google",
            "latest",
            SdkDataCollection::Personal,
            "https://firebase.google.com/support/privacy",
        );
        assert!(d.requires_disclosure());
    }

    #[test]
    fn sdk_data_collection_none_does_not_require_disclosure() {
        assert!(!SdkDataCollection::None.requires_disclosure());
        assert!(SdkDataCollection::Personal.requires_disclosure());
        assert!(SdkDataCollection::Location.requires_disclosure());
    }

    #[test]
    fn build_all_sdk_disclosures_has_entries() {
        let sdks = build_all_sdk_disclosures();
        assert!(sdks.len() >= 8);
    }

    #[test]
    fn sdk_disclosure_serializes_round_trip() {
        let d = SdkDisclosure::new(
            "Flutter",
            SdkCategory::UiFramework,
            "Google",
            "3.x",
            SdkDataCollection::None,
            "https://flutter.dev/privacy",
        ).open_source("BSD-3-Clause");
        let json = serde_json::to_string(&d).unwrap();
        let back: SdkDisclosure = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }

    // ── DataSafetyForm ──

    #[test]
    fn data_safety_form_has_entries() {
        let form = build_default_data_safety_form(1000);
        assert!(!form.entries.is_empty());
    }

    #[test]
    fn data_safety_form_is_complete() {
        let form = build_default_data_safety_form(1000);
        assert!(form.is_complete());
    }

    #[test]
    fn data_safety_form_encryption_flags() {
        let form = build_default_data_safety_form(1000);
        assert!(form.data_encrypted_in_transit);
        assert!(form.data_encrypted_at_rest);
    }

    #[test]
    fn data_safety_form_deletion_and_export_supported() {
        let form = build_default_data_safety_form(1000);
        assert!(form.data_deletion_supported);
        assert!(form.data_export_supported);
    }

    #[test]
    fn data_safety_form_has_shared_entries() {
        let form = build_default_data_safety_form(1000);
        let shared = form.shared_entries();
        assert!(!shared.is_empty());
    }

    #[test]
    fn data_safety_form_entry_for_location() {
        let form = build_default_data_safety_form(1000);
        let entry = form.entry_for(DataSafetyCategory::Location);
        assert!(entry.is_some());
    }

    #[test]
    fn data_safety_form_serializes_round_trip() {
        let form = build_default_data_safety_form(1000);
        let json = serde_json::to_string(&form).unwrap();
        let back: DataSafetyForm = serde_json::from_str(&json).unwrap();
        assert_eq!(form, back);
    }

    // ── PrivacyComplianceStatus ──

    #[test]
    fn privacy_compliance_status_defaults_to_all_false() {
        let s = PrivacyComplianceStatus::new();
        assert!(!s.is_ready_for_publication());
        assert_eq!(s.completed_count(), 0);
    }

    #[test]
    fn privacy_compliance_status_total_count_is_12() {
        let s = PrivacyComplianceStatus::new();
        assert_eq!(s.total_count(), 12);
    }

    #[test]
    fn privacy_compliance_status_completion_percent_zero_initially() {
        let s = PrivacyComplianceStatus::new();
        assert_eq!(s.completion_percent(), 0.0);
    }

    #[test]
    fn privacy_compliance_status_ready_when_all_complete() {
        let mut s = PrivacyComplianceStatus::new();
        s.privacy_policy_complete = true;
        s.terms_of_service_complete = true;
        s.health_disclaimer_acknowledged = true;
        s.disclosures_complete = true;
        s.retention_policy_complete = true;
        s.deletion_policy_complete = true;
        s.support_contact_configured = true;
        s.consent_registry_complete = true;
        s.attributions_complete = true;
        s.osm_attribution_compliant = true;
        s.sdk_disclosures_complete = true;
        s.data_safety_form_complete = true;
        assert!(s.is_ready_for_publication());
        assert_eq!(s.completed_count(), 12);
        assert_eq!(s.completion_percent(), 100.0);
    }

    #[test]
    fn build_privacy_compliance_status_evaluates_components() {
        let health = HealthDisclaimer::new().acknowledge(1000);
        let disclosures = build_all_disclosures();
        let retention = build_default_privacy_retention_policy(1000);
        let consent = build_default_consent_registry(1000);
        let osm = OpenStreetMapAttribution::new();
        let sdks = build_all_sdk_disclosures();
        let form = build_default_data_safety_form(1000);

        let status = build_privacy_compliance_status(
            &health, &disclosures, &retention, &consent, &osm, &sdks, &form,
        );

        assert!(status.privacy_policy_complete);
        assert!(status.health_disclaimer_acknowledged);
        assert!(status.disclosures_complete);
        assert!(status.retention_policy_complete);
        assert!(status.osm_attribution_compliant);
        assert!(status.sdk_disclosures_complete);
        assert!(status.data_safety_form_complete);
        assert!(!status.consent_registry_complete); // not all granted initially
    }

    #[test]
    fn privacy_compliance_status_serializes_round_trip() {
        let s = PrivacyComplianceStatus::new();
        let json = serde_json::to_string(&s).unwrap();
        let back: PrivacyComplianceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
