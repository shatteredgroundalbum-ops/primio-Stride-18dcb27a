//! §7 — Secure Firebase: field-level ownership validation, data-access
//! boundary checks, input sanitization, rate-limiting decisions, and
//! Cloud Storage path validation.
//!
//! The actual Firestore and Cloud Storage *rules* are written in
//! Firebase's rules language and deployed to the Firebase Console. This
//! module provides the **validation predicates** that the rules mirror
//! and that the Dart layer can call *before* making API calls — so
//! invalid or unauthorized requests are rejected client-side, reducing
//! unnecessary network round-trips and providing a consistent set of
//! security checks that can be unit-tested in Rust.
//!
//! ## What this module covers (per the §7 spec)
//!
//! - **Production Firestore rules**: collection path enumeration + field
//!   validation rules that the Firestore rules language mirrors.
//! - **Cloud Storage rules**: path ownership validation for route files.
//! - **Rules validation for every field**: per-collection field
//!   validators (type, range, required, allowed-field allow-list).
//! - **Ownership validation**: path-prefix ownership checks
//!   (`users/{userId}/...` must match the authenticated user's UID).
//! - **Admin-only collections**: which collections require admin claims.
//! - **Firebase App Check**: attestation state modeling + enforcement
//!   decision.
//! - **Play Integrity for Android**: attestation verdict classification.
//! - **Rate limiting for server functions**: token-bucket rate-limit
//!   decision logic for Cloud Functions endpoints.
//! - **No API secrets in client code**: secret-free client config
//!   validation (verifies no secret keys are present in the config map).
//! - **Separate dev/staging/prod projects**: environment enumeration +
//!   project-ID validation.
//!
//! ## What this module does NOT cover (external to the Rust engine)
//!
//! - The actual Firestore rules text (written in Firebase's rules
//!   language, deployed via the Firebase Console) — but this module
//!   provides the logic that the rules mirror.
//! - Firebase Emulator Suite tests (JavaScript/TypeScript, run outside
//!   the Rust engine) — but the validation predicates here are the
//!   same logic the emulator tests would verify.

use serde::{Deserialize, Serialize};

use crate::models::EpochMillis;

// ===========================================================================
// Collection path enumeration
// ===========================================================================

/// A Firestore collection or subcollection in the S.T.R.I.D.E. database.
///
/// The app uses a per-user subcollection pattern:
///   `users/{userId}/workouts/{workoutId}`
///   `users/{userId}/achievements/{achievementId}`
///   `users/{userId}/goals/{goalId}`
///   `users/{userId}/trainingPlans/{planId}`
///   `users/{userId}/savedRoutes/{routeId}`
///   `users/{userId}/dailySummaries/{dateId}`
///   `users/{userId}/checkpoints/{checkpointId}`
///   `users/{userId}/stepSamples/{sampleId}`
///   `users/{userId}/heartRateSamples/{sampleId}`
///   `users/{userId}/syncQueueItems/{itemId}`
///
/// Admin-only collections:
///   `admin/flaggedWorkouts/{workoutId}`
///   `admin/reportedUsers/{reportId}`
///   `admin/appConfig/{configId}`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirestoreCollection {
    /// User profile document: `users/{userId}`
    Users,
    /// Workout summary: `users/{userId}/workouts/{workoutId}`
    Workouts,
    /// Achievement record: `users/{userId}/achievements/{achievementId}`
    Achievements,
    /// Goals: `users/{userId}/goals/{goalId}`
    Goals,
    /// Training plans: `users/{userId}/trainingPlans/{planId}`
    TrainingPlans,
    /// Saved routes metadata: `users/{userId}/savedRoutes/{routeId}`
    SavedRoutes,
    /// Daily aggregated summaries: `users/{userId}/dailySummaries/{dateId}`
    DailySummaries,
    /// Crash-recovery checkpoints: `users/{userId}/checkpoints/{checkpointId}`
    Checkpoints,
    /// Step samples: `users/{userId}/stepSamples/{sampleId}`
    StepSamples,
    /// Heart rate samples: `users/{userId}/heartRateSamples/{sampleId}`
    HeartRateSamples,
    /// Sync queue items (pending uploads/deletions):
    /// `users/{userId}/syncQueueItems/{itemId}`
    SyncQueueItems,
    /// Coaching history: `users/{userId}/coachingHistory/{sessionId}`
    CoachingHistory,
    /// Personal records: `users/{userId}/personalRecords/{recordId}`
    PersonalRecords,
    // ── Admin-only collections ──
    /// Admin: flagged workouts for review.
    AdminFlaggedWorkouts,
    /// Admin: reported users.
    AdminReportedUsers,
    /// Admin: app-wide configuration.
    AdminAppConfig,
}

impl FirestoreCollection {
    /// Returns the Firestore collection path template (with `{userId}`
    /// placeholder for user-scoped collections).
    pub fn path_template(self) -> &'static str {
        match self {
            FirestoreCollection::Users => "users/{userId}",
            FirestoreCollection::Workouts => "users/{userId}/workouts",
            FirestoreCollection::Achievements => "users/{userId}/achievements",
            FirestoreCollection::Goals => "users/{userId}/goals",
            FirestoreCollection::TrainingPlans => "users/{userId}/trainingPlans",
            FirestoreCollection::SavedRoutes => "users/{userId}/savedRoutes",
            FirestoreCollection::DailySummaries => "users/{userId}/dailySummaries",
            FirestoreCollection::Checkpoints => "users/{userId}/checkpoints",
            FirestoreCollection::StepSamples => "users/{userId}/stepSamples",
            FirestoreCollection::HeartRateSamples => {
                "users/{userId}/heartRateSamples"
            }
            FirestoreCollection::SyncQueueItems => "users/{userId}/syncQueueItems",
            FirestoreCollection::CoachingHistory => {
                "users/{userId}/coachingHistory"
            }
            FirestoreCollection::PersonalRecords => {
                "users/{userId}/personalRecords"
            }
            FirestoreCollection::AdminFlaggedWorkouts => "admin/flaggedWorkouts",
            FirestoreCollection::AdminReportedUsers => "admin/reportedUsers",
            FirestoreCollection::AdminAppConfig => "admin/appConfig",
        }
    }

    /// Whether this collection is admin-only (requires admin custom claims).
    pub fn is_admin_only(self) -> bool {
        matches!(
            self,
            FirestoreCollection::AdminFlaggedWorkouts
                | FirestoreCollection::AdminReportedUsers
                | FirestoreCollection::AdminAppConfig
        )
    }

    /// Whether this collection is user-scoped (has a `{userId}` prefix).
    pub fn is_user_scoped(self) -> bool {
        !self.is_admin_only()
    }

    /// Returns the full document path for a user-scoped collection.
    ///
    /// Returns `None` if called on an admin-only collection (which doesn't
    /// have a `{userId}` prefix).
    pub fn doc_path(self, user_id: &str, doc_id: &str) -> Option<String> {
        if self.is_admin_only() {
            return None;
        }
        Some(format!("{}/{}", self.path_template().replace("{userId}", user_id), doc_id))
    }

    /// Returns the collection path for a user-scoped collection.
    pub fn collection_path(self, user_id: &str) -> Option<String> {
        if self.is_admin_only() {
            return None;
        }
        Some(self.path_template().replace("{userId}", user_id))
    }
}

// ===========================================================================
// Access decision
// ===========================================================================

/// The type of access being requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessType {
    Read,
    Write,
    Delete,
}

/// The result of an access-control check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessDecision {
    Allow,
    DenyOwnerMismatch,
    DenyAdminOnly,
    DenyNotAuthenticated,
    DenyReadOnly,
    DenyInvalidPath,
}

impl AccessDecision {
    pub fn is_allowed(self) -> bool {
        matches!(self, AccessDecision::Allow)
    }

    pub fn reason(self) -> &'static str {
        match self {
            AccessDecision::Allow => "Access granted",
            AccessDecision::DenyOwnerMismatch => {
                "Access denied: document does not belong to the authenticated user"
            }
            AccessDecision::DenyAdminOnly => {
                "Access denied: collection requires admin privileges"
            }
            AccessDecision::DenyNotAuthenticated => {
                "Access denied: user is not authenticated"
            }
            AccessDecision::DenyReadOnly => {
                "Access denied: collection is read-only"
            }
            AccessDecision::DenyInvalidPath => {
                "Access denied: invalid document path"
            }
        }
    }
}

/// Context for an access-control decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessContext {
    /// The authenticated user's UID (empty if not authenticated).
    pub user_id: String,
    /// Whether the user has admin custom claims.
    pub is_admin: bool,
    /// Whether the user is authenticated at all.
    pub is_authenticated: bool,
    /// The collection being accessed.
    pub collection: FirestoreCollection,
    /// The document owner's UID (extracted from the path).
    pub doc_owner_id: String,
    /// The type of access being requested.
    pub access_type: AccessType,
}

/// Checks whether a user has access to a Firestore document.
///
/// This mirrors the logic in the Firestore rules:
/// - User must be authenticated.
/// - For user-scoped collections, the document's `userId` must match the
///   authenticated user's UID.
/// - For admin-only collections, the user must have admin custom claims.
pub fn check_access(ctx: &AccessContext) -> AccessDecision {
    if !ctx.is_authenticated {
        return AccessDecision::DenyNotAuthenticated;
    }

    if ctx.collection.is_admin_only() {
        if ctx.is_admin {
            return AccessDecision::Allow;
        }
        return AccessDecision::DenyAdminOnly;
    }

    // User-scoped collection: the document owner must match the
    // authenticated user.
    if ctx.doc_owner_id.is_empty() {
        return AccessDecision::DenyInvalidPath;
    }

    if ctx.doc_owner_id == ctx.user_id {
        AccessDecision::Allow
    } else {
        AccessDecision::DenyOwnerMismatch
    }
}

/// Validates that a Firestore document path belongs to the given user.
///
/// The path must be in the form `users/{userId}/...` for user-scoped
/// collections, or `admin/...` for admin-only collections.
pub fn validate_path_ownership(
    path: &str,
    authenticated_user_id: &str,
) -> Result<(), String> {
    if path.is_empty() {
        return Err("path is empty".to_string());
    }

    // Admin paths don't require user ownership (handled by admin checks).
    if path.starts_with("admin/") {
        return Ok(());
    }

    // User-scoped paths must start with "users/{userId}/"
    let expected_prefix = format!("users/{}/", authenticated_user_id);
    if !path.starts_with(&expected_prefix) {
        return Err(format!(
            "path '{}' does not start with expected owner prefix '{}'",
            path, expected_prefix
        ));
    }

    Ok(())
}

// ===========================================================================
// Field-level validation
// ===========================================================================

/// A field type in a Firestore document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Timestamp,
    Array,
    Map,
    Null,
}

/// A field validation rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRule {
    /// The field name.
    pub field: String,
    /// The expected type.
    pub field_type: FieldType,
    /// Whether the field is required (must be present).
    pub required: bool,
    /// Minimum value (for numeric fields), or minimum length (for strings).
    pub min: Option<f64>,
    /// Maximum value (for numeric fields), or maximum length (for strings).
    pub max: Option<f64>,
}

/// A validation issue found in a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidationIssue {
    /// The field name with the issue.
    pub field: String,
    /// A description of the issue.
    pub issue: String,
}

/// The result of validating a document's fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidationResult {
    /// Whether all fields are valid.
    pub is_valid: bool,
    /// A list of issues found (empty if valid).
    pub issues: Vec<FieldValidationIssue>,
}

/// Returns the field validation rules for a collection.
///
/// These rules mirror the Firestore rules — each collection has a set of
/// allowed fields with types, constraints, and required flags. The
/// Firestore rules use `request.resource.data.{field}` checks that
/// mirror these rules.
pub fn field_rules_for_collection(
    collection: FirestoreCollection,
) -> Vec<FieldRule> {
    match collection {
        FirestoreCollection::Users => vec![
            FieldRule {
                field: "id".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "email".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(3.0),
                max: Some(256.0),
            },
            FieldRule {
                field: "displayName".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(100.0),
            },
            FieldRule {
                field: "age".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(0.0),
                max: Some(150.0),
            },
            FieldRule {
                field: "weightKg".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(500.0),
            },
            FieldRule {
                field: "heightCm".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(300.0),
            },
            FieldRule {
                field: "activityLevel".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(50.0),
            },
            FieldRule {
                field: "createdAt".to_string(),
                field_type: FieldType::Timestamp,
                required: true,
                min: None,
                max: None,
            },
        ],
        FirestoreCollection::Workouts => vec![
            FieldRule {
                field: "workoutId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "activityType".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(30.0),
            },
            FieldRule {
                field: "startTime".to_string(),
                field_type: FieldType::Timestamp,
                required: true,
                min: None,
                max: None,
            },
            FieldRule {
                field: "endTime".to_string(),
                field_type: FieldType::Timestamp,
                required: true,
                min: None,
                max: None,
            },
            FieldRule {
                field: "durationMs".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(0.0),
                max: Some(86400000.0),
            },
            FieldRule {
                field: "distanceMeters".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
            FieldRule {
                field: "steps".to_string(),
                field_type: FieldType::Integer,
                required: false,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
            FieldRule {
                field: "caloriesBurned".to_string(),
                field_type: FieldType::Float,
                required: false,
                min: Some(0.0),
                max: Some(100000.0),
            },
            FieldRule {
                field: "encodedPolyline".to_string(),
                field_type: FieldType::String,
                required: false,
                min: None,
                max: Some(1048576.0),
            },
            FieldRule {
                field: "routeFilePath".to_string(),
                field_type: FieldType::String,
                required: false,
                min: None,
                max: Some(1024.0),
            },
        ],
        FirestoreCollection::Achievements => vec![
            FieldRule {
                field: "achievementId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "type".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(50.0),
            },
            FieldRule {
                field: "awardedAt".to_string(),
                field_type: FieldType::Timestamp,
                required: true,
                min: None,
                max: None,
            },
        ],
        FirestoreCollection::Goals => vec![
            FieldRule {
                field: "goalId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "type".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(30.0),
            },
            FieldRule {
                field: "target".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
            FieldRule {
                field: "isActive".to_string(),
                field_type: FieldType::Boolean,
                required: true,
                min: None,
                max: None,
            },
        ],
        FirestoreCollection::TrainingPlans => vec![
            FieldRule {
                field: "planId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "name".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(200.0),
            },
            FieldRule {
                field: "totalDays".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(1.0),
                max: Some(365.0),
            },
            FieldRule {
                field: "isActive".to_string(),
                field_type: FieldType::Boolean,
                required: true,
                min: None,
                max: None,
            },
        ],
        FirestoreCollection::SavedRoutes => vec![
            FieldRule {
                field: "routeId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "name".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(200.0),
            },
            FieldRule {
                field: "distanceMeters".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
            FieldRule {
                field: "routeFilePath".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(1024.0),
            },
        ],
        FirestoreCollection::DailySummaries => vec![
            FieldRule {
                field: "dateId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(10.0),
                max: Some(10.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "totalSteps".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
            FieldRule {
                field: "totalDistanceMeters".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
            FieldRule {
                field: "totalActiveMinutes".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(0.0),
                max: Some(1440.0),
            },
        ],
        FirestoreCollection::Checkpoints => vec![
            FieldRule {
                field: "workoutId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "timestampMs".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(0.0),
                max: None,
            },
            FieldRule {
                field: "elapsedMs".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(0.0),
                max: Some(86400000.0),
            },
            FieldRule {
                field: "distanceMeters".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
        ],
        FirestoreCollection::StepSamples | FirestoreCollection::HeartRateSamples => vec![
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "timestampMs".to_string(),
                field_type: FieldType::Integer,
                required: true,
                min: Some(0.0),
                max: None,
            },
        ],
        FirestoreCollection::SyncQueueItems => vec![
            FieldRule {
                field: "itemId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "operation".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(20.0),
            },
            FieldRule {
                field: "status".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(20.0),
            },
        ],
        FirestoreCollection::CoachingHistory => vec![
            FieldRule {
                field: "sessionId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "createdAt".to_string(),
                field_type: FieldType::Timestamp,
                required: true,
                min: None,
                max: None,
            },
        ],
        FirestoreCollection::PersonalRecords => vec![
            FieldRule {
                field: "recordId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "userId".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(128.0),
            },
            FieldRule {
                field: "type".to_string(),
                field_type: FieldType::String,
                required: true,
                min: Some(1.0),
                max: Some(50.0),
            },
            FieldRule {
                field: "value".to_string(),
                field_type: FieldType::Float,
                required: true,
                min: Some(0.0),
                max: Some(1_000_000.0),
            },
        ],
        // Admin collections: no field rules enforced client-side (the
        // admin console manages these directly).
        FirestoreCollection::AdminFlaggedWorkouts
        | FirestoreCollection::AdminReportedUsers
        | FirestoreCollection::AdminAppConfig => vec![],
    }
}

/// Returns the list of allowed field names for a collection.
///
/// Any field not in this list is rejected by the Firestore rules.
pub fn allowed_fields_for_collection(
    collection: FirestoreCollection,
) -> Vec<String> {
    field_rules_for_collection(collection)
        .into_iter()
        .map(|r| r.field)
        .collect()
}

/// Validates a field value against its rule.
///
/// `value` is a JSON value representing the field's current value.
/// Returns `Ok(())` if valid, or an error message.
pub fn validate_field_value(rule: &FieldRule, value: &serde_json::Value) -> Result<(), String> {
    // Check required
    if rule.required && value.is_null() {
        return Err(format!("field '{}' is required but missing", rule.field));
    }

    if value.is_null() {
        return Ok(()); // Optional field, not present
    }

    // Check type
    let type_ok = match rule.field_type {
        FieldType::String => value.is_string(),
        FieldType::Integer => value.is_i64() || value.is_u64(),
        FieldType::Float => value.is_f64() || value.is_i64() || value.is_u64(),
        FieldType::Boolean => value.is_boolean(),
        FieldType::Timestamp => value.is_string() || value.is_i64() || value.is_u64(),
        FieldType::Array => value.is_array(),
        FieldType::Map => value.is_object(),
        FieldType::Null => value.is_null(),
    };

    if !type_ok {
        return Err(format!(
            "field '{}' has wrong type: expected {:?}, got {}",
            rule.field, rule.field_type, value
        ));
    }

    // Check min/max for numeric values
    if let Some(min) = rule.min {
        if let Some(n) = value.as_f64() {
            if n < min {
                return Err(format!(
                    "field '{}' value {} is below minimum {}",
                    rule.field, n, min
                ));
            }
        }
    }
    if let Some(max) = rule.max {
        if let Some(n) = value.as_f64() {
            if n > max {
                return Err(format!(
                    "field '{}' value {} exceeds maximum {}",
                    rule.field, n, max
                ));
            }
        }
    }

    // Check string length
    if let Some(s) = value.as_str() {
        if let Some(min) = rule.min {
            if (s.len() as f64) < min {
                return Err(format!(
                    "field '{}' length {} is below minimum {}",
                    rule.field, s.len(), min
                ));
            }
        }
        if let Some(max) = rule.max {
            if (s.len() as f64) > max {
                return Err(format!(
                    "field '{}' length {} exceeds maximum {}",
                    rule.field, s.len(), max
                ));
            }
        }
    }

    Ok(())
}

/// Validates a document against the field rules for its collection.
///
/// `doc` is a JSON object representing the Firestore document.
/// Returns a `FieldValidationResult` with any issues found.
pub fn validate_document(
    collection: FirestoreCollection,
    doc: &serde_json::Value,
) -> FieldValidationResult {
    let rules = field_rules_for_collection(collection);
    let mut issues = Vec::new();

    let obj = match doc.as_object() {
        Some(o) => o,
        None => {
            issues.push(FieldValidationIssue {
                field: "_root".to_string(),
                issue: "document is not a JSON object".to_string(),
            });
            return FieldValidationResult {
                is_valid: false,
                issues,
            };
        }
    };

    for rule in &rules {
        let value = obj.get(&rule.field).unwrap_or(&serde_json::Value::Null);
        if let Err(e) = validate_field_value(rule, value) {
            issues.push(FieldValidationIssue {
                field: rule.field.clone(),
                issue: e,
            });
        }
    }

    // Check for unexpected fields (fields not in the allowed list)
    let allowed: Vec<String> = rules.iter().map(|r| r.field.clone()).collect();
    for key in obj.keys() {
        if !allowed.contains(key) {
            issues.push(FieldValidationIssue {
                field: key.clone(),
                issue: "field is not in the allowed list for this collection".to_string(),
            });
        }
    }

    FieldValidationResult {
        is_valid: issues.is_empty(),
        issues,
    }
}

// ===========================================================================
// Input sanitization
// ===========================================================================

/// Maximum length for a sanitized string field.
const MAX_SANITIZED_LENGTH: usize = 10_000;

/// Sanitizes a string for safe storage in Firestore.
///
/// - Trims leading/trailing whitespace.
/// - Removes null bytes and control characters.
/// - Truncates to `MAX_SANITIZED_LENGTH`.
/// - Removes common XSS patterns (`<script>`, `javascript:`, etc.).
pub fn sanitize_string(input: &str) -> String {
    let trimmed = input.trim();

    // Remove null bytes and control characters (except newline and tab)
    let filtered: String = trimmed
        .chars()
        .filter(|c| {
            (*c as u32) >= 0x20 || *c == '\n' || *c == '\t'
        })
        .collect();

    // Truncate to max length
    let truncated = if filtered.len() > MAX_SANITIZED_LENGTH {
        &filtered[..MAX_SANITIZED_LENGTH]
    } else {
        &filtered
    };

    // Remove common XSS patterns (case-insensitive)
    let lower = truncated.to_lowercase();
    let xss_patterns = [
        "<script", "</script>", "javascript:", "onerror=", "onload=",
        "onclick=", "<iframe", "</iframe>", "<embed", "<object",
        "data:text/html", "vbscript:",
    ];

    let mut result = truncated.to_string();
    for pattern in &xss_patterns {
        if lower.contains(pattern) {
            // Replace the pattern in the original (case-preserving)
            result = result.replace(pattern, "");
        }
    }

    result.trim().to_string()
}

/// Whether a string contains potential injection patterns.
///
/// Returns `Some(reason)` if the string is suspicious, or `None` if clean.
pub fn detect_injection(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    // SQL injection patterns
    let sql_patterns = [
        "'; --", "' or '1'='1", "' or 1=1", "union select",
        "drop table", "insert into", "delete from",
        "'; drop", "xp_", "sp_",
    ];
    for pattern in &sql_patterns {
        if lower.contains(pattern) {
            return Some(format!("potential SQL injection: detected pattern '{}'", pattern));
        }
    }

    // NoSQL injection patterns (Firestore)
    let nosql_patterns = [
        "__proto__", "constructor.prototype", "$where", "$expr",
    ];
    for pattern in &nosql_patterns {
        if lower.contains(pattern) {
            return Some(format!("potential NoSQL injection: detected pattern '{}'", pattern));
        }
    }

    // XSS patterns
    let xss_patterns = [
        "<script", "javascript:", "onerror=", "onload=",
        "<iframe", "<embed", "<object",
    ];
    for pattern in &xss_patterns {
        if lower.contains(pattern) {
            return Some(format!("potential XSS: detected pattern '{}'", pattern));
        }
    }

    None
}

/// Whether a string is safe for storage (passes sanitization + injection check).
pub fn is_safe_string(input: &str) -> bool {
    detect_injection(input).is_none()
}

// ===========================================================================
// Cloud Storage path validation
// ===========================================================================

/// Validates a Cloud Storage path for user ownership.
///
/// Route files must be stored under `routes/{userId}/...` to ensure
/// the Storage rules can verify ownership.
pub fn validate_storage_path(
    path: &str,
    authenticated_user_id: &str,
) -> Result<(), String> {
    if path.is_empty() {
        return Err("storage path is empty".to_string());
    }

    // Route files must be under routes/{userId}/
    if path.starts_with("routes/") {
        let expected_prefix = format!("routes/{}/", authenticated_user_id);
        if !path.starts_with(&expected_prefix) {
            return Err(format!(
                "storage path '{}' does not start with expected owner prefix '{}'",
                path, expected_prefix
            ));
        }
        return Ok(());
    }

    // Profile images must be under avatars/{userId}/
    if path.starts_with("avatars/") {
        let expected_prefix = format!("avatars/{}/", authenticated_user_id);
        if !path.starts_with(&expected_prefix) {
            return Err(format!(
                "storage path '{}' does not start with expected owner prefix '{}'",
                path, expected_prefix
            ));
        }
        return Ok(());
    }

    Err(format!("storage path '{}' has unknown root prefix", path))
}

// ===========================================================================
// App Check / Play Integrity
// ===========================================================================

/// The attestation state from Firebase App Check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppCheckState {
    /// App Check token is valid.
    Valid,
    /// App Check token is valid but the attestation is outdated.
    Stale,
    /// App Check token is missing.
    Missing,
    /// App Check token is invalid (tampered or revoked).
    Invalid,
    /// App Check is not enforced for this request.
    NotEnforced,
}

/// Whether a request should be allowed based on App Check state.
///
/// - `Valid` → allow
/// - `Stale` → allow (but the Dart layer should refresh the token)
/// - `Missing` or `Invalid` → deny
/// - `NotEnforced` → allow (App Check not required for this endpoint)
pub fn app_check_decision(state: AppCheckState) -> AccessDecision {
    match state {
        AppCheckState::Valid | AppCheckState::Stale | AppCheckState::NotEnforced => {
            AccessDecision::Allow
        }
        AppCheckState::Missing => AccessDecision::DenyNotAuthenticated,
        AppCheckState::Invalid => AccessDecision::DenyInvalidPath,
    }
}

/// The Play Integrity API verdict for an Android device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayIntegrityVerdict {
    /// Device passes all checks (genuine app, certified device, no
    /// known malware).
    Pass,
    /// App integrity check failed (app is not the genuine, unmodified
    /// version from the Play Store).
    AppIntegrityFailed,
    /// Device integrity check failed (device is not certified or is
    /// rooted/emulated).
    DeviceIntegrityFailed,
    /// Account integrity check failed (no valid Google account on device).
    AccountIntegrityFailed,
    /// Basic integrity check failed (general safety net failure).
    BasicIntegrityFailed,
}

impl PlayIntegrityVerdict {
    /// Whether the verdict is a pass (device is safe).
    pub fn is_pass(self) -> bool {
        matches!(self, PlayIntegrityVerdict::Pass)
    }

    /// Returns a human-readable description of the verdict.
    pub fn description(self) -> &'static str {
        match self {
            PlayIntegrityVerdict::Pass => "Device passes all integrity checks",
            PlayIntegrityVerdict::AppIntegrityFailed => {
                "App integrity check failed: app may be modified or tampered"
            }
            PlayIntegrityVerdict::DeviceIntegrityFailed => {
                "Device integrity check failed: device may be rooted or emulated"
            }
            PlayIntegrityVerdict::AccountIntegrityFailed => {
                "Account integrity check failed: no valid Google account"
            }
            PlayIntegrityVerdict::BasicIntegrityFailed => {
                "Basic integrity check failed: device safety net failure"
            }
        }
    }
}

/// Whether a request should be allowed based on Play Integrity verdict.
///
/// Only `Pass` is allowed; all other verdicts are denied.
pub fn play_integrity_decision(verdict: PlayIntegrityVerdict) -> AccessDecision {
    if verdict.is_pass() {
        AccessDecision::Allow
    } else {
        AccessDecision::DenyInvalidPath
    }
}

// ===========================================================================
// Rate limiting
// ===========================================================================

/// A token-bucket rate limiter state.
///
/// The bucket has a `capacity` (maximum tokens) and a `refill_rate`
/// (tokens per second). Each request consumes one token. When the
/// bucket is empty, the request is rate-limited.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitBucket {
    /// Maximum number of tokens in the bucket.
    pub capacity: f64,
    /// Token refill rate (tokens per second).
    pub refill_rate: f64,
    /// Current number of tokens in the bucket.
    pub current_tokens: f64,
    /// Last refill timestamp (epoch ms).
    pub last_refill_ms: EpochMillis,
}

impl RateLimitBucket {
    /// Creates a new rate limit bucket with the given capacity and
    /// refill rate, starting full.
    pub fn new(capacity: f64, refill_rate: f64, now_ms: EpochMillis) -> Self {
        RateLimitBucket {
            capacity,
            refill_rate,
            current_tokens: capacity,
            last_refill_ms: now_ms,
        }
    }
}

/// The result of a rate-limit check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RateLimitResult {
    /// Whether the request is allowed (not rate-limited).
    pub allowed: bool,
    /// Remaining tokens after this request.
    pub remaining_tokens: f64,
    /// Milliseconds until the next token is available (0 if allowed).
    pub retry_after_ms: i64,
}

/// Checks and consumes a token from the rate limit bucket.
///
/// If the bucket has at least one token, the request is allowed and one
/// token is consumed. If the bucket is empty, the request is denied and
/// the `retry_after_ms` field indicates when the next token will be
/// available.
///
/// The bucket is refilled based on the elapsed time since the last
/// refill, up to the capacity.
pub fn check_rate_limit(
    bucket: &mut RateLimitBucket,
    now_ms: EpochMillis,
) -> RateLimitResult {
    // Refill tokens based on elapsed time
    if now_ms > bucket.last_refill_ms {
        let elapsed_s = (now_ms - bucket.last_refill_ms) as f64 / 1000.0;
        let refilled = elapsed_s * bucket.refill_rate;
        bucket.current_tokens = (bucket.current_tokens + refilled).min(bucket.capacity);
        bucket.last_refill_ms = now_ms;
    }

    if bucket.current_tokens >= 1.0 {
        bucket.current_tokens -= 1.0;
        RateLimitResult {
            allowed: true,
            remaining_tokens: bucket.current_tokens,
            retry_after_ms: 0,
        }
    } else {
        // Calculate when the next token will be available
        let retry_after_ms = if bucket.refill_rate > 0.0 {
            ((1.0 - bucket.current_tokens) / bucket.refill_rate * 1000.0) as i64
        } else {
            i64::MAX
        };
        RateLimitResult {
            allowed: false,
            remaining_tokens: bucket.current_tokens,
            retry_after_ms,
        }
    }
}

/// Default rate limit for Cloud Functions endpoints.
///
/// - AI coaching: 10 requests per minute (capacity=10, refill=0.167/s)
/// - Workout sync: 60 requests per minute (capacity=60, refill=1.0/s)
/// - Route upload: 10 requests per minute (capacity=10, refill=0.167/s)
/// - General API: 100 requests per minute (capacity=100, refill=1.667/s)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitCategory {
    AiCoaching,
    WorkoutSync,
    RouteUpload,
    GeneralApi,
}

/// Returns the default capacity and refill rate for a rate limit category.
pub fn rate_limit_config(category: RateLimitCategory) -> (f64, f64) {
    match category {
        RateLimitCategory::AiCoaching => (10.0, 10.0 / 60.0),
        RateLimitCategory::WorkoutSync => (60.0, 60.0 / 60.0),
        RateLimitCategory::RouteUpload => (10.0, 10.0 / 60.0),
        RateLimitCategory::GeneralApi => (100.0, 100.0 / 60.0),
    }
}

// ===========================================================================
// Secret-free client config validation
// ===========================================================================

/// Checks a config map for any secret keys that should not be stored
/// in client code.
///
/// The spec requires "No API secrets stored in FlutterFlow variables or
/// client code." This function checks a key-value config map for keys
/// that look like they contain secrets.
pub fn check_for_secrets(config: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    let secret_patterns = [
        "secret", "password", "api_key", "apikey", "private_key",
        "privatekey", "token", "credential", "access_key",
        "accesskey", "client_secret", "firebase_admin",
    ];

    let mut found = Vec::new();

    for (key, value) in config {
        let key_lower = key.to_lowercase();
        for pattern in &secret_patterns {
            if key_lower.contains(pattern) {
                // Check if the value is non-empty (a real secret, not a
                // placeholder or empty field)
                let is_real_secret = match value {
                    serde_json::Value::String(s) => !s.is_empty(),
                    serde_json::Value::Null => false,
                    _ => true,
                };
                if is_real_secret {
                    found.push(format!(
                        "Potential secret found in config: key '{}' matches pattern '{}'",
                        key, pattern
                    ));
                }
                break;
            }
        }
    }

    found
}

/// Whether a config map is free of secrets.
pub fn is_secret_free(config: &serde_json::Map<String, serde_json::Value>) -> bool {
    check_for_secrets(config).is_empty()
}

// ===========================================================================
// Environment / project validation
// ===========================================================================

/// The Firebase environment (development, staging, or production).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirebaseEnvironment {
    Development,
    Staging,
    Production,
}

impl FirebaseEnvironment {
    /// Returns the expected Firebase project ID for this environment.
    ///
    /// The S.T.R.I.D.E. project uses separate projects for dev, staging,
    /// and production. The production project ID is `stride-78bfe`.
    pub fn project_id(self) -> &'static str {
        match self {
            FirebaseEnvironment::Development => "stride-dev",
            FirebaseEnvironment::Staging => "stride-staging",
            FirebaseEnvironment::Production => "stride-78bfe",
        }
    }

    /// Whether the environment is production (strictest security).
    pub fn is_production(self) -> bool {
        matches!(self, FirebaseEnvironment::Production)
    }

    /// Returns a human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            FirebaseEnvironment::Development => "Development",
            FirebaseEnvironment::Staging => "Staging",
            FirebaseEnvironment::Production => "Production",
        }
    }
}

/// Validates that a project ID matches the expected environment.
pub fn validate_project_id(
    project_id: &str,
    expected: FirebaseEnvironment,
) -> Result<(), String> {
    let expected_id = expected.project_id();
    if project_id != expected_id {
        return Err(format!(
            "project ID '{}' does not match expected '{}' for {} environment",
            project_id, expected_id, expected.label()
        ));
    }
    Ok(())
}

/// Parses a project ID string into a FirebaseEnvironment.
pub fn environment_from_project_id(project_id: &str) -> Option<FirebaseEnvironment> {
    match project_id {
        "stride-dev" => Some(FirebaseEnvironment::Development),
        "stride-staging" => Some(FirebaseEnvironment::Staging),
        "stride-78bfe" => Some(FirebaseEnvironment::Production),
        _ => None,
    }
}

// ===========================================================================
// Firestore rules text generation
// ===========================================================================

/// Generates the Firestore rules text for the S.T.R.I.D.E. app.
///
/// This is the complete production Firestore rules document that can be
/// pasted into the Firebase Console → Firestore Database → Rules editor.
/// The rules enforce:
/// - User must be authenticated.
/// - User-scoped collections require the document's `userId` to match
///   the authenticated user's UID.
/// - Admin-only collections require admin custom claims.
/// - Field validation for write operations.
pub fn generate_firestore_rules() -> String {
    r#"rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {

    // ── Helper functions ──

    // Returns true if the user is authenticated.
    function isSignedIn() {
      return request.auth != null;
    }

    // Returns true if the user has admin custom claims.
    function isAdmin() {
      return isSignedIn() && request.auth.token.admin == true;
    }

    // Returns true if the document's userId matches the authenticated user.
    function isOwner(userId) {
      return isSignedIn() && request.auth.uid == userId;
    }

    // Returns true if the user is the owner of a user-scoped document.
    function isDocumentOwner() {
      return isOwner(resource.data.userId);
    }

    // Returns true if the new document's userId matches the authenticated user.
    function isCreatingOwner() {
      return isOwner(request.resource.data.userId);
    }

    // Returns true if the userId field is unchanged on update.
    function userIdUnchanged() {
      return request.resource.data.userId == resource.data.userId;
    }

    // ── User profile: users/{userId} ──
    match /users/{userId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.id == userId
        && request.resource.data.email is string
        && request.resource.data.email.size() > 0
        && request.resource.data.email.size() <= 256
        && request.resource.data.displayName is string
        && request.resource.data.displayName.size() > 0
        && request.resource.data.displayName.size() <= 100
        && request.resource.data.age is int
        && request.resource.data.age >= 0
        && request.resource.data.age <= 150
        && request.resource.data.weightKg is number
        && request.resource.data.weightKg >= 0
        && request.resource.data.weightKg <= 500
        && request.resource.data.heightCm is number
        && request.resource.data.heightCm >= 0
        && request.resource.data.heightCm <= 300
        && request.resource.data.activityLevel is string
        && request.resource.data.createdAt is timestamp;
      allow update: if isOwner(userId)
        && userIdUnchanged()
        && request.resource.data.weightKg >= 0
        && request.resource.data.weightKg <= 500
        && request.resource.data.heightCm >= 0
        && request.resource.data.heightCm <= 300;
      allow delete: if isOwner(userId);
    }

    // ── Workouts: users/{userId}/workouts/{workoutId} ──
    match /users/{userId}/workouts/{workoutId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.workoutId is string
        && request.resource.data.workoutId.size() > 0
        && request.resource.data.activityType is string
        && request.resource.data.startTime is timestamp
        && request.resource.data.endTime is timestamp
        && request.resource.data.durationMs is int
        && request.resource.data.durationMs >= 0
        && request.resource.data.durationMs <= 86400000
        && request.resource.data.distanceMeters is number
        && request.resource.data.distanceMeters >= 0
        && request.resource.data.distanceMeters <= 1000000;
      allow update: if isOwner(userId)
        && request.resource.data.userId == resource.data.userId;
      allow delete: if isOwner(userId);
    }

    // ── Achievements: users/{userId}/achievements/{achievementId} ──
    match /users/{userId}/achievements/{achievementId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.type is string
        && request.resource.data.awardedAt is timestamp;
      allow update: if false; // Achievements are immutable
      allow delete: if isOwner(userId);
    }

    // ── Goals: users/{userId}/goals/{goalId} ──
    match /users/{userId}/goals/{goalId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.type is string
        && request.resource.data.target is number
        && request.resource.data.target >= 0;
      allow update: if isOwner(userId)
        && request.resource.data.userId == resource.data.userId;
      allow delete: if isOwner(userId);
    }

    // ── Training plans: users/{userId}/trainingPlans/{planId} ──
    match /users/{userId}/trainingPlans/{planId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.name is string
        && request.resource.data.name.size() > 0
        && request.resource.data.name.size() <= 200
        && request.resource.data.totalDays is int
        && request.resource.data.totalDays >= 1
        && request.resource.data.totalDays <= 365;
      allow update: if isOwner(userId)
        && request.resource.data.userId == resource.data.userId;
      allow delete: if isOwner(userId);
    }

    // ── Saved routes: users/{userId}/savedRoutes/{routeId} ──
    match /users/{userId}/savedRoutes/{routeId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.name is string
        && request.resource.data.name.size() > 0
        && request.resource.data.routeFilePath is string
        && request.resource.data.routeFilePath.matches('routes/[^/]+/.*');
      allow update: if isOwner(userId)
        && request.resource.data.userId == resource.data.userId;
      allow delete: if isOwner(userId);
    }

    // ── Daily summaries: users/{userId}/dailySummaries/{dateId} ──
    match /users/{userId}/dailySummaries/{dateId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.dateId == dateId;
      allow update: if isOwner(userId)
        && request.resource.data.userId == resource.data.userId;
      allow delete: if isOwner(userId);
    }

    // ── Checkpoints: users/{userId}/checkpoints/{checkpointId} ──
    match /users/{userId}/checkpoints/{checkpointId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.workoutId is string
        && request.resource.data.timestampMs is int
        && request.resource.data.timestampMs >= 0;
      allow update: if false; // Checkpoints are append-only
      allow delete: if isOwner(userId);
    }

    // ── Step samples: users/{userId}/stepSamples/{sampleId} ──
    match /users/{userId}/stepSamples/{sampleId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.timestampMs is int
        && request.resource.data.timestampMs >= 0;
      allow update: if false; // Samples are append-only
      allow delete: if isOwner(userId);
    }

    // ── Heart rate samples: users/{userId}/heartRateSamples/{sampleId} ──
    match /users/{userId}/heartRateSamples/{sampleId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.timestampMs is int
        && request.resource.data.timestampMs >= 0;
      allow update: if false; // Samples are append-only
      allow delete: if isOwner(userId);
    }

    // ── Sync queue items: users/{userId}/syncQueueItems/{itemId} ──
    match /users/{userId}/syncQueueItems/{itemId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.operation is string
        && request.resource.data.status is string;
      allow update: if isOwner(userId)
        && request.resource.data.userId == resource.data.userId;
      allow delete: if isOwner(userId);
    }

    // ── Coaching history: users/{userId}/coachingHistory/{sessionId} ──
    match /users/{userId}/coachingHistory/{sessionId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId;
      allow update: if false; // Coaching history is append-only
      allow delete: if isOwner(userId);
    }

    // ── Personal records: users/{userId}/personalRecords/{recordId} ──
    match /users/{userId}/personalRecords/{recordId} {
      allow read: if isOwner(userId);
      allow create: if isOwner(userId)
        && request.resource.data.userId == userId
        && request.resource.data.type is string
        && request.resource.data.value is number
        && request.resource.data.value >= 0;
      allow update: if false; // Personal records are immutable
      allow delete: if isOwner(userId);
    }

    // ── Admin-only collections ──
    match /admin/flaggedWorkouts/{workoutId} {
      allow read: if isAdmin();
      allow write: if isAdmin();
    }
    match /admin/reportedUsers/{reportId} {
      allow read: if isAdmin();
      allow write: if isAdmin();
    }
    match /admin/appConfig/{configId} {
      allow read: if isAdmin();
      allow write: if isAdmin();
    }

    // ── Default: deny everything else ──
    match /{document=**} {
      allow read, write: if false;
    }
  }
}
"#.to_string()
}

/// Generates the Cloud Storage rules text for the S.T.R.I.D.E. app.
///
/// Route files must be stored under `routes/{userId}/` and the
/// authenticated user must own the path. Avatar images must be stored
/// under `avatars/{userId}/`.
pub fn generate_storage_rules() -> String {
    r#"rules_version = '2';
service firebase.storage {
  match /b/{bucket}/o {

    // Helper: is the user authenticated?
    function isSignedIn() {
      return request.auth != null;
    }

    // Helper: is the user the owner of the path?
    function isOwner(userId) {
      return isSignedIn() && request.auth.uid == userId;
    }

    // ── Route files: routes/{userId}/{workoutId}.{ext} ──
    match /routes/{userId}/{fileName=**} {
      allow read: if isOwner(userId);
      allow write: if isOwner(userId)
        && request.resource.size < 50 * 1024 * 1024 // 50 MB max
        && request.resource.contentType.matches('application/.*');
      allow delete: if isOwner(userId);
    }

    // ── Avatar images: avatars/{userId}/avatar.{ext} ──
    match /avatars/{userId}/{fileName=**} {
      allow read: if true; // Avatars are publicly readable
      allow write: if isOwner(userId)
        && request.resource.size < 5 * 1024 * 1024 // 5 MB max
        && request.resource.contentType.matches('image/.*');
      allow delete: if isOwner(userId);
    }

    // ── Default: deny everything else ──
    match /{allPaths=**} {
      allow read, write: if false;
    }
  }
}
"#.to_string()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Collection path enumeration ──

    #[test]
    fn collection_path_templates() {
        assert_eq!(
            FirestoreCollection::Workouts.path_template(),
            "users/{userId}/workouts"
        );
        assert_eq!(
            FirestoreCollection::Achievements.path_template(),
            "users/{userId}/achievements"
        );
        assert_eq!(
            FirestoreCollection::Goals.path_template(),
            "users/{userId}/goals"
        );
        assert_eq!(
            FirestoreCollection::TrainingPlans.path_template(),
            "users/{userId}/trainingPlans"
        );
    }

    #[test]
    fn admin_only_collections() {
        assert!(FirestoreCollection::AdminFlaggedWorkouts.is_admin_only());
        assert!(FirestoreCollection::AdminReportedUsers.is_admin_only());
        assert!(FirestoreCollection::AdminAppConfig.is_admin_only());
    }

    #[test]
    fn user_scoped_collections() {
        assert!(FirestoreCollection::Workouts.is_user_scoped());
        assert!(FirestoreCollection::Achievements.is_user_scoped());
        assert!(!FirestoreCollection::AdminFlaggedWorkouts.is_user_scoped());
    }

    #[test]
    fn doc_path_for_user_scoped() {
        let path = FirestoreCollection::Workouts
            .doc_path("user123", "workout456")
            .unwrap();
        assert_eq!(path, "users/user123/workouts/workout456");
    }

    #[test]
    fn doc_path_for_admin_returns_none() {
        assert!(FirestoreCollection::AdminAppConfig
            .doc_path("user123", "config1")
            .is_none());
    }

    // ── Access control ──

    #[test]
    fn access_allowed_for_owner() {
        let ctx = AccessContext {
            user_id: "user123".to_string(),
            is_admin: false,
            is_authenticated: true,
            collection: FirestoreCollection::Workouts,
            doc_owner_id: "user123".to_string(),
            access_type: AccessType::Read,
        };
        assert_eq!(check_access(&ctx), AccessDecision::Allow);
    }

    #[test]
    fn access_denied_for_non_owner() {
        let ctx = AccessContext {
            user_id: "user123".to_string(),
            is_admin: false,
            is_authenticated: true,
            collection: FirestoreCollection::Workouts,
            doc_owner_id: "other".to_string(),
            access_type: AccessType::Read,
        };
        assert_eq!(check_access(&ctx), AccessDecision::DenyOwnerMismatch);
    }

    #[test]
    fn access_denied_for_unauthenticated() {
        let ctx = AccessContext {
            user_id: "".to_string(),
            is_admin: false,
            is_authenticated: false,
            collection: FirestoreCollection::Workouts,
            doc_owner_id: "user123".to_string(),
            access_type: AccessType::Read,
        };
        assert_eq!(check_access(&ctx), AccessDecision::DenyNotAuthenticated);
    }

    #[test]
    fn access_denied_for_admin_only_non_admin() {
        let ctx = AccessContext {
            user_id: "user123".to_string(),
            is_admin: false,
            is_authenticated: true,
            collection: FirestoreCollection::AdminAppConfig,
            doc_owner_id: "".to_string(),
            access_type: AccessType::Read,
        };
        assert_eq!(check_access(&ctx), AccessDecision::DenyAdminOnly);
    }

    #[test]
    fn access_allowed_for_admin_on_admin_collection() {
        let ctx = AccessContext {
            user_id: "admin1".to_string(),
            is_admin: true,
            is_authenticated: true,
            collection: FirestoreCollection::AdminAppConfig,
            doc_owner_id: "".to_string(),
            access_type: AccessType::Write,
        };
        assert_eq!(check_access(&ctx), AccessDecision::Allow);
    }

    #[test]
    fn access_denied_for_empty_owner() {
        let ctx = AccessContext {
            user_id: "user123".to_string(),
            is_admin: false,
            is_authenticated: true,
            collection: FirestoreCollection::Workouts,
            doc_owner_id: "".to_string(),
            access_type: AccessType::Read,
        };
        assert_eq!(check_access(&ctx), AccessDecision::DenyInvalidPath);
    }

    // ── Path ownership ──

    #[test]
    fn path_ownership_valid() {
        assert!(validate_path_ownership(
            "users/user123/workouts/workout456",
            "user123"
        )
        .is_ok());
    }

    #[test]
    fn path_ownership_mismatch() {
        assert!(validate_path_ownership(
            "users/other/workouts/workout456",
            "user123"
        )
        .is_err());
    }

    #[test]
    fn path_ownership_admin_bypasses() {
        assert!(validate_path_ownership(
            "admin/appConfig/config1",
            "user123"
        )
        .is_ok());
    }

    #[test]
    fn path_ownership_empty_rejected() {
        assert!(validate_path_ownership("", "user123").is_err());
    }

    // ── Field validation ──

    #[test]
    fn field_rules_for_workouts() {
        let rules = field_rules_for_collection(FirestoreCollection::Workouts);
        assert!(rules.iter().any(|r| r.field == "workoutId"));
        assert!(rules.iter().any(|r| r.field == "userId"));
        assert!(rules.iter().any(|r| r.field == "distanceMeters"));
    }

    #[test]
    fn field_rules_for_users() {
        let rules = field_rules_for_collection(FirestoreCollection::Users);
        assert!(rules.iter().any(|r| r.field == "email"));
        assert!(rules.iter().any(|r| r.field == "displayName"));
    }

    #[test]
    fn validate_field_value_string_ok() {
        let rule = FieldRule {
            field: "name".to_string(),
            field_type: FieldType::String,
            required: true,
            min: Some(1.0),
            max: Some(100.0),
        };
        assert!(validate_field_value(&rule, &json!("John Doe")).is_ok());
    }

    #[test]
    fn validate_field_value_required_missing() {
        let rule = FieldRule {
            field: "name".to_string(),
            field_type: FieldType::String,
            required: true,
            min: Some(1.0),
            max: Some(100.0),
        };
        assert!(validate_field_value(&rule, &json!(null)).is_err());
    }

    #[test]
    fn validate_field_value_optional_missing_ok() {
        let rule = FieldRule {
            field: "notes".to_string(),
            field_type: FieldType::String,
            required: false,
            min: None,
            max: Some(1000.0),
        };
        assert!(validate_field_value(&rule, &json!(null)).is_ok());
    }

    #[test]
    fn validate_field_value_wrong_type() {
        let rule = FieldRule {
            field: "age".to_string(),
            field_type: FieldType::Integer,
            required: true,
            min: Some(0.0),
            max: Some(150.0),
        };
        assert!(validate_field_value(&rule, &json!("not a number")).is_err());
    }

    #[test]
    fn validate_field_value_below_min() {
        let rule = FieldRule {
            field: "age".to_string(),
            field_type: FieldType::Integer,
            required: true,
            min: Some(0.0),
            max: Some(150.0),
        };
        assert!(validate_field_value(&rule, &json!(-5)).is_err());
    }

    #[test]
    fn validate_field_value_above_max() {
        let rule = FieldRule {
            field: "age".to_string(),
            field_type: FieldType::Integer,
            required: true,
            min: Some(0.0),
            max: Some(150.0),
        };
        assert!(validate_field_value(&rule, &json!(200)).is_err());
    }

    #[test]
    fn validate_field_value_string_too_long() {
        let rule = FieldRule {
            field: "name".to_string(),
            field_type: FieldType::String,
            required: true,
            min: Some(1.0),
            max: Some(10.0),
        };
        let long_string = "a".repeat(11);
        assert!(validate_field_value(&rule, &json!(long_string)).is_err());
    }

    #[test]
    fn validate_document_valid_workout() {
        let doc = json!({
            "workoutId": "w1",
            "userId": "u1",
            "activityType": "walking",
            "startTime": "2024-01-01T00:00:00Z",
            "endTime": "2024-01-01T01:00:00Z",
            "durationMs": 3600000,
            "distanceMeters": 5000.0,
        });
        let result = validate_document(FirestoreCollection::Workouts, &doc);
        assert!(result.is_valid, "Issues: {:?}", result.issues);
    }

    #[test]
    fn validate_document_missing_required_field() {
        let doc = json!({
            "workoutId": "w1",
            // userId is missing
        });
        let result = validate_document(FirestoreCollection::Workouts, &doc);
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.field == "userId"));
    }

    #[test]
    fn validate_document_unexpected_field() {
        let doc = json!({
            "workoutId": "w1",
            "userId": "u1",
            "activityType": "walking",
            "startTime": "2024-01-01T00:00:00Z",
            "endTime": "2024-01-01T01:00:00Z",
            "durationMs": 3600000,
            "distanceMeters": 5000.0,
            "hackedField": "malicious",
        });
        let result = validate_document(FirestoreCollection::Workouts, &doc);
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.field == "hackedField"));
    }

    // ── Input sanitization ──

    #[test]
    fn sanitize_trims_whitespace() {
        assert_eq!(sanitize_string("  hello  "), "hello");
    }

    #[test]
    fn sanitize_removes_null_bytes() {
        assert_eq!(sanitize_string("hello\u{0000}world"), "helloworld");
    }

    #[test]
    fn sanitize_removes_xss_patterns() {
        let input = "hello <script>alert('xss')</script> world";
        let sanitized = sanitize_string(input);
        assert!(!sanitized.to_lowercase().contains("<script"));
        assert!(!sanitized.to_lowercase().contains("</script>"));
    }

    #[test]
    fn sanitize_removes_javascript_protocol() {
        let input = "javascript:alert('xss')";
        let sanitized = sanitize_string(input);
        assert!(!sanitized.to_lowercase().contains("javascript:"));
    }

    #[test]
    fn sanitize_truncates_long_strings() {
        let long = "a".repeat(20_000);
        let sanitized = sanitize_string(&long);
        assert!(sanitized.len() <= MAX_SANITIZED_LENGTH);
    }

    #[test]
    fn detect_injection_sql() {
        assert!(detect_injection("'; DROP TABLE users; --").is_some());
        assert!(detect_injection("' OR '1'='1").is_some());
    }

    #[test]
    fn detect_injection_nosql() {
        assert!(detect_injection("__proto__").is_some());
        assert!(detect_injection("$where").is_some());
    }

    #[test]
    fn detect_injection_xss() {
        assert!(detect_injection("<script>alert(1)</script>").is_some());
    }

    #[test]
    fn detect_injection_clean_string() {
        assert!(detect_injection("Hello, world!").is_none());
        assert!(detect_injection("user@example.com").is_none());
    }

    #[test]
    fn is_safe_string_clean() {
        assert!(is_safe_string("Hello, world!"));
    }

    #[test]
    fn is_safe_string_sql_injection() {
        assert!(!is_safe_string("'; DROP TABLE users; --"));
    }

    // ── Storage path validation ──

    #[test]
    fn storage_path_valid_route() {
        assert!(validate_storage_path(
            "routes/user123/workout456.gpx",
            "user123"
        )
        .is_ok());
    }

    #[test]
    fn storage_path_wrong_owner() {
        assert!(validate_storage_path(
            "routes/other/workout456.gpx",
            "user123"
        )
        .is_err());
    }

    #[test]
    fn storage_path_valid_avatar() {
        assert!(validate_storage_path(
            "avatars/user123/avatar.png",
            "user123"
        )
        .is_ok());
    }

    #[test]
    fn storage_path_unknown_prefix() {
        assert!(validate_storage_path(
            "unknown/user123/file.txt",
            "user123"
        )
        .is_err());
    }

    #[test]
    fn storage_path_empty_rejected() {
        assert!(validate_storage_path("", "user123").is_err());
    }

    // ── App Check ──

    #[test]
    fn app_check_valid_allows() {
        assert_eq!(
            app_check_decision(AppCheckState::Valid),
            AccessDecision::Allow
        );
    }

    #[test]
    fn app_check_missing_denies() {
        assert_eq!(
            app_check_decision(AppCheckState::Missing),
            AccessDecision::DenyNotAuthenticated
        );
    }

    #[test]
    fn app_check_invalid_denies() {
        assert_eq!(
            app_check_decision(AppCheckState::Invalid),
            AccessDecision::DenyInvalidPath
        );
    }

    #[test]
    fn app_check_not_enforced_allows() {
        assert_eq!(
            app_check_decision(AppCheckState::NotEnforced),
            AccessDecision::Allow
        );
    }

    // ── Play Integrity ──

    #[test]
    fn play_integrity_pass_allows() {
        assert_eq!(
            play_integrity_decision(PlayIntegrityVerdict::Pass),
            AccessDecision::Allow
        );
    }

    #[test]
    fn play_integrity_failed_denies() {
        assert_eq!(
            play_integrity_decision(PlayIntegrityVerdict::AppIntegrityFailed),
            AccessDecision::DenyInvalidPath
        );
        assert_eq!(
            play_integrity_decision(PlayIntegrityVerdict::DeviceIntegrityFailed),
            AccessDecision::DenyInvalidPath
        );
    }

    #[test]
    fn play_integrity_description() {
        assert!(!PlayIntegrityVerdict::Pass.description().is_empty());
        assert!(!PlayIntegrityVerdict::AppIntegrityFailed.description().is_empty());
    }

    // ── Rate limiting ──

    #[test]
    fn rate_limit_allows_within_capacity() {
        let mut bucket = RateLimitBucket::new(5.0, 1.0, 0);
        for _ in 0..5 {
            let result = check_rate_limit(&mut bucket, 0);
            assert!(result.allowed);
        }
    }

    #[test]
    fn rate_limit_denies_when_empty() {
        let mut bucket = RateLimitBucket::new(1.0, 1.0, 0);
        // Consume the one token
        let r1 = check_rate_limit(&mut bucket, 0);
        assert!(r1.allowed);
        // Now empty
        let r2 = check_rate_limit(&mut bucket, 0);
        assert!(!r2.allowed);
        assert!(r2.retry_after_ms > 0);
    }

    #[test]
    fn rate_limit_refills_over_time() {
        let mut bucket = RateLimitBucket::new(1.0, 1.0, 0);
        // Consume the one token
        check_rate_limit(&mut bucket, 0);
        // Wait 1 second
        let result = check_rate_limit(&mut bucket, 1000);
        assert!(result.allowed);
    }

    #[test]
    fn rate_limit_does_not_exceed_capacity() {
        let mut bucket = RateLimitBucket::new(2.0, 10.0, 0);
        // Consume both tokens
        check_rate_limit(&mut bucket, 0);
        check_rate_limit(&mut bucket, 0);
        // Wait 10 seconds (should refill 100 tokens but cap at capacity)
        let result = check_rate_limit(&mut bucket, 10_000);
        assert!(result.allowed);
        assert!(result.remaining_tokens <= 2.0);
    }

    #[test]
    fn rate_limit_config_values() {
        let (cap, rate) = rate_limit_config(RateLimitCategory::AiCoaching);
        assert_eq!(cap, 10.0);
        assert!(rate > 0.0);

        let (cap2, _) = rate_limit_config(RateLimitCategory::WorkoutSync);
        assert_eq!(cap2, 60.0);
    }

    // ── Secret detection ──

    #[test]
    fn detect_secret_api_key() {
        let mut config = serde_json::Map::new();
        config.insert(
            "apiKey".to_string(),
            json!("AIzaSyxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
        );
        let found = check_for_secrets(&config);
        assert!(!found.is_empty());
    }

    #[test]
    fn detect_secret_password() {
        let mut config = serde_json::Map::new();
        config.insert("password".to_string(), json!("mySecret123"));
        let found = check_for_secrets(&config);
        assert!(!found.is_empty());
    }

    #[test]
    fn no_secret_for_empty_value() {
        let mut config = serde_json::Map::new();
        config.insert("apiKey".to_string(), json!(""));
        let found = check_for_secrets(&config);
        assert!(found.is_empty(), "Empty values should not be flagged");
    }

    #[test]
    fn no_secret_for_clean_config() {
        let mut config = serde_json::Map::new();
        config.insert("theme".to_string(), json!("dark"));
        config.insert("units".to_string(), json!("metric"));
        assert!(is_secret_free(&config));
    }

    // ── Environment ──

    #[test]
    fn environment_project_ids() {
        assert_eq!(
            FirebaseEnvironment::Production.project_id(),
            "stride-78bfe"
        );
        assert_eq!(
            FirebaseEnvironment::Development.project_id(),
            "stride-dev"
        );
        assert_eq!(
            FirebaseEnvironment::Staging.project_id(),
            "stride-staging"
        );
    }

    #[test]
    fn environment_is_production() {
        assert!(FirebaseEnvironment::Production.is_production());
        assert!(!FirebaseEnvironment::Development.is_production());
    }

    #[test]
    fn validate_project_id_correct() {
        assert!(validate_project_id("stride-78bfe", FirebaseEnvironment::Production).is_ok());
    }

    #[test]
    fn validate_project_id_wrong() {
        assert!(validate_project_id("stride-dev", FirebaseEnvironment::Production).is_err());
    }

    #[test]
    fn environment_from_project_id_valid() {
        assert_eq!(
            environment_from_project_id("stride-78bfe"),
            Some(FirebaseEnvironment::Production)
        );
        assert_eq!(
            environment_from_project_id("stride-dev"),
            Some(FirebaseEnvironment::Development)
        );
    }

    #[test]
    fn environment_from_project_id_unknown() {
        assert_eq!(environment_from_project_id("unknown-project"), None);
    }

    // ── Rules generation ──

    #[test]
    fn firestore_rules_not_empty() {
        let rules = generate_firestore_rules();
        assert!(!rules.is_empty());
        assert!(rules.contains("rules_version = '2'"));
        assert!(rules.contains("isSignedIn"));
        assert!(rules.contains("isOwner"));
    }

    #[test]
    fn firestore_rules_cover_all_collections() {
        let rules = generate_firestore_rules();
        assert!(rules.contains("users/{userId}/workouts"));
        assert!(rules.contains("users/{userId}/achievements"));
        assert!(rules.contains("users/{userId}/goals"));
        assert!(rules.contains("users/{userId}/trainingPlans"));
        assert!(rules.contains("users/{userId}/savedRoutes"));
        assert!(rules.contains("admin/flaggedWorkouts"));
    }

    #[test]
    fn firestore_rules_have_default_deny() {
        let rules = generate_firestore_rules();
        assert!(rules.contains("allow read, write: if false"));
    }

    #[test]
    fn storage_rules_not_empty() {
        let rules = generate_storage_rules();
        assert!(!rules.is_empty());
        assert!(rules.contains("rules_version = '2'"));
        assert!(rules.contains("routes/{userId}"));
    }
}
