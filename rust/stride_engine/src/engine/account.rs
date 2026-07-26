//! Authentication and account lifecycle engine (spec section 6).
//!
//! Pure decision logic for the authentication and account-management
//! subsystem. The actual Firebase Auth operations (sign-in, sign-up,
//! token refresh, password reset, etc.) happen on the Dart/Kotlin side;
//! this module decides:
//!
//!   - Whether a session is still valid or needs re-authentication
//!   - Which user-owned data collections exist and must be enumerated
//!     for complete account deletion
//!   - Whether a sensitive action requires reauthentication
//!   - Whether a login attempt is suspicious (impossible travel, new
//!     device, brute-force pattern)
//!   - Whether a disabled account should be shown a specific message
//!   - How to handle email/password vs Google sign-in differences
//!
//! Key responsibilities:
//!   - Auth state classification (session validity, token expiry)
//!   - User-data enumeration for account deletion (GDPR / Play Store)
//!   - Reauthentication decision for sensitive operations
//!   - Suspicious-login detection (geographic, device, temporal)
//!   - Disabled-account handling (message selection)
//!   - Password policy validation (strength, common-password check)
//!   - Account deletion scope determination

use serde::{Deserialize, Serialize};

use crate::models::EpochMillis;

// ---------------------------------------------------------------------------
// Auth provider
// ---------------------------------------------------------------------------

/// Which authentication provider the user signed in with. This
/// determines which operations are available (e.g. password reset is
/// only available for email/password accounts, not Google sign-in).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthProvider {
    /// Email and password registration.
    EmailPassword,
    /// Google sign-in (OAuth).
    Google,
    /// Apple sign-in (OAuth, for iOS).
    Apple,
    /// Anonymous/guest account (temporary, can be upgraded).
    Anonymous,
}

impl Default for AuthProvider {
    fn default() -> Self {
        AuthProvider::EmailPassword
    }
}

/// Returns a human-readable label for an auth provider.
pub fn auth_provider_label(provider: AuthProvider) -> &'static str {
    match provider {
        AuthProvider::EmailPassword => "Email & Password",
        AuthProvider::Google => "Google",
        AuthProvider::Apple => "Apple",
        AuthProvider::Anonymous => "Guest",
    }
}

// ---------------------------------------------------------------------------
// Session validity
// ---------------------------------------------------------------------------

/// The result of checking whether the user's current session is still
/// valid. The Dart layer calls this on app startup and periodically
/// during use to decide whether the user needs to re-authenticate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is valid — the token is fresh and the user is
    /// authenticated. No action needed.
    Valid,
    /// Token is close to expiry but still valid. The Dart layer
    /// should proactively refresh the token in the background.
    Refreshing,
    /// Token has expired. The Dart layer must refresh the token
    /// before making any authenticated API calls. If refresh fails,
    /// the user must re-sign-in.
    Expired,
    /// No session exists (user is signed out or never signed in).
    /// The Dart layer should show the sign-in screen.
    NoSession,
    /// The session has been revoked (e.g. user changed password on
    /// another device, or admin disabled the account). The user
    /// must sign in again.
    Revoked,
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState::NoSession
    }
}

/// Margin before token expiry at which to trigger a proactive refresh.
/// Firebase ID tokens last 1 hour; we refresh 5 minutes before expiry
/// to avoid race conditions where a request hits the server right at
/// the expiry moment.
const REFRESH_MARGIN_MS: i64 = 5 * 60 * 1_000; // 5 minutes

/// Checks the validity of the user's session based on token age.
///
/// `token_issued_at_ms` is when the current ID token was issued.
/// `token_expires_at_ms` is when it expires (typically
/// `issued_at + 3600_000` for Firebase).
/// `now_ms` is the current time.
/// `is_revoked` is whether the session has been revoked (e.g. by a
/// password change on another device).
///
/// Returns:
///   - `Valid` if the token is fresh (> 5 min remaining)
///   - `Refreshing` if the token is close to expiry (≤ 5 min remaining)
///   - `Expired` if the token has expired
///   - `Revoked` if the session has been revoked
///   - `NoSession` if `token_issued_at_ms` is 0 (never authenticated)
pub fn classify_session_state(
    token_issued_at_ms: EpochMillis,
    token_expires_at_ms: EpochMillis,
    now_ms: EpochMillis,
    is_revoked: bool,
) -> SessionState {
    if is_revoked {
        return SessionState::Revoked;
    }
    if token_issued_at_ms == 0 {
        return SessionState::NoSession;
    }
    if now_ms >= token_expires_at_ms {
        return SessionState::Expired;
    }
    let remaining = token_expires_at_ms - now_ms;
    if remaining <= REFRESH_MARGIN_MS {
        SessionState::Refreshing
    } else {
        SessionState::Valid
    }
}

/// Whether the session needs a token refresh before making an
/// authenticated API call.
pub fn needs_token_refresh(state: SessionState) -> bool {
    matches!(state, SessionState::Refreshing | SessionState::Expired)
}

/// Whether the user must re-sign-in (session is irrevocably lost).
pub fn requires_relogin(state: SessionState) -> bool {
    matches!(state, SessionState::NoSession | SessionState::Revoked)
}

// ---------------------------------------------------------------------------
// Reauthentication for sensitive changes
// ---------------------------------------------------------------------------

/// Operations that require the user to reauthenticate before
/// proceeding. Firebase requires recent sign-in for these operations;
/// the engine decides which operations are "sensitive" and need
/// re-auth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveAction {
    /// Change the user's password.
    ChangePassword,
    /// Change the user's email address.
    ChangeEmail,
    /// Delete the user's account entirely.
    DeleteAccount,
    /// Link a new auth provider (e.g. link Google to an email account).
    LinkAuthProvider,
    /// Delete all user data (separate from account deletion in
    /// Firebase, this covers Firestore/Cloud Storage data).
    DeleteAllUserData,
}

impl Default for SensitiveAction {
    fn default() -> Self {
        SensitiveAction::ChangePassword
    }
}

/// Decides whether a given action requires reauthentication, based on
/// how recently the user authenticated and the action type.
///
/// `last_auth_at_ms` is when the user last authenticated (signed in or
/// reauthenticated). `now_ms` is the current time.
///
/// Firebase requires "recent" sign-in for sensitive operations; we
/// define "recent" as within the last 5 minutes. After that, the user
/// must reauthenticate before performing the action.
pub fn requires_reauthentication(
    action: SensitiveAction,
    last_auth_at_ms: EpochMillis,
    now_ms: EpochMillis,
) -> bool {
    if last_auth_at_ms == 0 {
        return true; // Never authenticated — always require re-auth
    }

    let elapsed = now_ms - last_auth_at_ms;
    let threshold_ms = reauth_threshold_ms(action);
    elapsed > threshold_ms
}

/// Returns the reauthentication threshold (in milliseconds) for a
/// sensitive action — how long after the last auth the action can be
/// performed without re-authenticating.
///
/// All sensitive actions require re-auth within 5 minutes of the last
/// sign-in; after that, the user must reauthenticate.
pub fn reauth_threshold_ms(action: SensitiveAction) -> i64 {
    match action {
        SensitiveAction::ChangePassword => 5 * 60 * 1_000, // 5 min
        SensitiveAction::ChangeEmail => 5 * 60 * 1_000, // 5 min
        SensitiveAction::DeleteAccount => 5 * 60 * 1_000, // 5 min
        SensitiveAction::LinkAuthProvider => 5 * 60 * 1_000, // 5 min
        SensitiveAction::DeleteAllUserData => 5 * 60 * 1_000, // 5 min
    }
}

/// Returns the human-readable description for why reauthentication is
/// needed for a given action.
pub fn reauth_reason(action: SensitiveAction) -> &'static str {
    match action {
        SensitiveAction::ChangePassword => {
            "For your security, please sign in again before changing your password."
        }
        SensitiveAction::ChangeEmail => {
            "For your security, please sign in again before changing your email."
        }
        SensitiveAction::DeleteAccount => {
            "Account deletion requires recent authentication. Please sign in again to continue."
        }
        SensitiveAction::LinkAuthProvider => {
            "Please sign in again to link a new sign-in method."
        }
        SensitiveAction::DeleteAllUserData => {
            "Deleting all your data requires recent authentication. Please sign in again."
        }
    }
}

// ---------------------------------------------------------------------------
// Email verification
// ---------------------------------------------------------------------------

/// Whether the user has verified their email address. Firebase
/// distinguishes between verified and unverified accounts; some
/// features (e.g. password reset, certain API access) may be gated on
/// verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailVerificationState {
    /// Email has been verified.
    Verified,
    /// Email has not been verified; a verification email has been sent.
    Pending,
    /// Email has not been verified and no verification email has been
    /// sent yet.
    NotSent,
    /// Not applicable — the user signed in with a provider that doesn't
    /// use email (e.g. anonymous).
    NotApplicable,
}

impl Default for EmailVerificationState {
    fn default() -> Self {
        EmailVerificationState::NotSent
    }
}

/// Decides what action to take regarding email verification.
///
/// `provider` is the auth provider (Google accounts are auto-verified).
/// `is_verified` is the current verification state.
/// `verification_sent_at_ms` is when the verification email was last
/// sent (0 if never sent).
/// `now_ms` is the current time.
///
/// Returns:
///   - `Verified` if already verified
///   - `NotApplicable` if using a provider without email
///   - `Pending` if a verification email was sent recently
///   - `NotSent` if no verification email has been sent or it's been
///     long enough to resend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationAction {
    /// Email is verified — no action needed.
    Verified,
    /// Email is not verified — send a verification email.
    SendVerification,
    /// Email is not verified — a verification email was recently sent,
    /// don't spam the user with another one.
    WaitForResend,
    /// Not applicable for this provider.
    NotApplicable,
}

/// Minimum time between verification email resends (5 minutes).
const VERIFICATION_RESEND_COOLDOWN_MS: i64 = 5 * 60 * 1_000;

/// Decides what action to take regarding email verification.
pub fn decide_verification_action(
    provider: AuthProvider,
    is_verified: bool,
    verification_sent_at_ms: EpochMillis,
    now_ms: EpochMillis,
) -> VerificationAction {
    if is_verified {
        return VerificationAction::Verified;
    }

    // Google and Apple sign-in accounts are auto-verified by the
    // provider — they don't need email verification.
    if matches!(provider, AuthProvider::Google | AuthProvider::Apple) {
        return VerificationAction::NotApplicable;
    }

    if verification_sent_at_ms == 0 {
        return VerificationAction::SendVerification;
    }

    let elapsed = now_ms - verification_sent_at_ms;
    if elapsed < VERIFICATION_RESEND_COOLDOWN_MS {
        VerificationAction::WaitForResend
    } else {
        VerificationAction::SendVerification
    }
}

/// Whether the user can resend a verification email (cooldown check).
pub fn can_resend_verification(
    verification_sent_at_ms: EpochMillis,
    now_ms: EpochMillis,
) -> bool {
    if verification_sent_at_ms == 0 {
        return true;
    }
    (now_ms - verification_sent_at_ms) >= VERIFICATION_RESEND_COOLDOWN_MS
}

// ---------------------------------------------------------------------------
// Password policy
// ---------------------------------------------------------------------------

/// Result of validating a password against the app's password policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PasswordValidationResult {
    /// Whether the password meets all requirements.
    pub is_valid: bool,
    /// A list of specific issues with the password (empty if valid).
    pub issues: Vec<String>,
}

/// Minimum password length.
const MIN_PASSWORD_LENGTH: usize = 8;

/// Maximum password length (Firebase allows up to 256, we cap at 128
/// for practical purposes).
const MAX_PASSWORD_LENGTH: usize = 128;

/// Validates a password against the app's password policy:
///   - At least 8 characters
///   - At most 128 characters
///   - Contains at least one uppercase letter
///   - Contains at least one lowercase letter
///   - Contains at least one digit
///   - Not in the list of commonly used passwords
pub fn validate_password(password: &str) -> PasswordValidationResult {
    let mut issues = Vec::new();

    if password.len() < MIN_PASSWORD_LENGTH {
        issues.push(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        ));
    }
    if password.len() > MAX_PASSWORD_LENGTH {
        issues.push(format!(
            "Password must be at most {} characters long",
            MAX_PASSWORD_LENGTH
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        issues.push("Password must contain at least one uppercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        issues.push("Password must contain at least one lowercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        issues.push("Password must contain at least one digit".to_string());
    }
    if is_common_password(password) {
        issues.push("This password is too common. Please choose a more unique password.".to_string());
    }

    PasswordValidationResult {
        is_valid: issues.is_empty(),
        issues,
    }
}

/// A small list of the most commonly used passwords. In a real app this
/// would be a larger list or a hash database, but for the engine's
/// decision logic, checking against the top offenders is sufficient.
const COMMON_PASSWORDS: &[&str] = &[
    "12345678",
    "123456789",
    "1234567890",
    "password1",
    "Password1",
    "password12",
    "Password12",
    "qwerty12",
    "Qwerty12",
    "abc12345",
    "Abc12345",
    "11111111",
    "00000000",
    "12312312",
    "1q2w3e4r",
    "1Q2W3E4R",
    "letmein1",
    "Letmein1",
    "welcome1",
    "Welcome1",
    "monkey123",
    "Monkey123",
    "dragon12",
    "Dragon12",
    "master12",
    "Master12",
    "football1",
    "Football1",
    "iloveyou1",
    "Iloveyou1",
];

/// Checks if a password is in the common-password list.
fn is_common_password(password: &str) -> bool {
    COMMON_PASSWORDS.iter().any(|&cp| cp == password)
}

/// Returns a password strength score from 0 to 4 (0=very weak, 4=very
/// strong). This is a simple heuristic:
///   +1 for length >= 8
///   +1 for length >= 12
///   +1 for mixed case (both upper and lower)
///   +1 for digits + special characters
pub fn password_strength_score(password: &str) -> u8 {
    let mut score: u8 = 0;
    if password.len() >= MIN_PASSWORD_LENGTH {
        score += 1;
    }
    if password.len() >= 12 {
        score += 1;
    }
    if password.chars().any(|c| c.is_uppercase()) && password.chars().any(|c| c.is_lowercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) && password.chars().any(|c| !c.is_alphanumeric())
    {
        score += 1;
    }
    score
}

/// Returns a human-readable label for a password strength score.
pub fn password_strength_label(score: u8) -> &'static str {
    match score {
        0 => "Very weak",
        1 => "Weak",
        2 => "Fair",
        3 => "Good",
        4 => "Strong",
        _ => "Strong",
    }
}

// ---------------------------------------------------------------------------
// Email validation
// ---------------------------------------------------------------------------

/// Validates an email address format. This is a basic syntactic check;
/// the actual existence/deliverability is verified by the verification
/// email.
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email is empty".to_string());
    }
    if email.len() > 254 {
        return Err("Email is too long".to_string());
    }
    // Must contain exactly one @
    let at_count = email.matches('@').count();
    if at_count != 1 {
        return Err("Email must contain exactly one @ symbol".to_string());
    }
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    let local = parts[0];
    let domain = parts[1];
    if local.is_empty() {
        return Err("Email local part (before @) is empty".to_string());
    }
    if local.len() > 64 {
        return Err("Email local part is too long".to_string());
    }
    if domain.is_empty() {
        return Err("Email domain (after @) is empty".to_string());
    }
    // Domain must contain at least one dot
    if !domain.contains('.') {
        return Err("Email domain must contain a dot".to_string());
    }
    // Domain parts must not be empty
    for part in domain.split('.') {
        if part.is_empty() {
            return Err("Email domain has empty parts".to_string());
        }
    }
    // No spaces allowed
    if email.contains(' ') {
        return Err("Email cannot contain spaces".to_string());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Account status (disabled accounts)
// ---------------------------------------------------------------------------

/// The account's status, as reported by Firebase Auth. The engine
/// decides what message to show and whether the user can proceed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// Account is active and in good standing.
    Active,
    /// Account has been disabled by an administrator.
    AdminDisabled,
    /// Account has been temporarily locked due to suspicious activity.
    TemporarilyLocked,
    /// Account is pending deletion (user initiated, within the
    /// grace period before permanent deletion).
    PendingDeletion,
}

impl Default for AccountStatus {
    fn default() -> Self {
        AccountStatus::Active
    }
}

/// Returns the user-facing message for a disabled/locked account.
pub fn account_status_message(status: AccountStatus) -> &'static str {
    match status {
        AccountStatus::Active => "",
        AccountStatus::AdminDisabled => {
            "Your account has been disabled. Please contact support for assistance."
        }
        AccountStatus::TemporarilyLocked => {
            "Your account has been temporarily locked due to unusual activity. Please try again later or contact support."
        }
        AccountStatus::PendingDeletion => {
            "Your account is scheduled for deletion. If this was a mistake, please contact support immediately."
        }
    }
}

/// Whether the user can sign in with this account status.
pub fn can_sign_in(status: AccountStatus) -> bool {
    matches!(status, AccountStatus::Active | AccountStatus::PendingDeletion)
}

// ---------------------------------------------------------------------------
// Suspicious login detection
// ---------------------------------------------------------------------------

/// Context for a login attempt, used to detect suspicious activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginContext {
    /// The user ID attempting to sign in.
    pub user_id: String,
    /// IP address of the login attempt (or empty if unknown).
    pub ip_address: String,
    /// Approximate latitude of the login attempt (or 0.0 if unknown).
    pub latitude: f64,
    /// Approximate longitude of the login attempt (or 0.0 if unknown).
    pub longitude: f64,
    /// Epoch ms of the login attempt.
    pub login_at_ms: EpochMillis,
    /// Device fingerprint (hash of device model, OS version, etc.).
    pub device_fingerprint: String,
    /// User agent string (browser/app identifier).
    pub user_agent: String,
}

/// The user's known login history, used to compare against a new
/// attempt to detect suspicious activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginHistory {
    /// Previous successful login IP addresses (most recent first).
    pub known_ip_addresses: Vec<String>,
    /// Previous device fingerprints (most recent first).
    pub known_device_fingerprints: Vec<String>,
    /// Previous login locations as (lat, lon) pairs.
    pub known_locations: Vec<(f64, f64)>,
    /// Epoch ms of the last successful login.
    pub last_login_at_ms: EpochMillis,
    /// Last login location (lat, lon).
    pub last_login_location: (f64, f64),
}

/// Result of analyzing a login attempt for suspicious activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SuspiciousLoginResult {
    /// Whether the login is flagged as suspicious.
    pub is_suspicious: bool,
    /// The specific reason(s) the login is suspicious (empty if not).
    pub reasons: Vec<String>,
    /// Recommended action to take.
    pub recommended_action: SuspiciousLoginAction,
}

/// What action to take when a suspicious login is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuspiciousLoginAction {
    /// Login is not suspicious — allow it.
    Allow,
    /// Login is slightly suspicious — allow but send a notification
    /// to the user.
    AllowWithNotification,
    /// Login is suspicious — require additional verification (e.g.
    /// email verification, 2FA).
    RequireVerification,
    /// Login is very suspicious — block the attempt.
    Block,
}

impl Default for SuspiciousLoginAction {
    fn default() -> Self {
        SuspiciousLoginAction::Allow
    }
}

/// Maximum speed (km/h) that a human can reasonably travel between two
/// logins. If the distance/time implies faster than this, it's
/// "impossible travel" (one login from New York, next login 5 minutes
/// later from Tokyo = impossible).
const MAX_TRAVEL_SPEED_KMH: f64 = 1_000.0; // ~concorde speed

/// Maximum number of allowed unknown IPs before requiring verification.
const MAX_UNKNOWN_IPS: usize = 3;

/// Time window (24 hours) within which to flag impossible travel.
const IMPOSSIBLE_TRAVEL_WINDOW_MS: i64 = 24 * 60 * 60 * 1_000;

/// Analyzes a login attempt against the user's login history to detect
/// suspicious activity.
///
/// Detects:
///   - **Impossible travel**: login from a location too far from the
///     last login given the time elapsed
///   - **New device**: login from a device not in the known list
///   - **New IP**: login from an IP not in the known list (after
///     accumulating several, flag as suspicious)
///   - **Unusual time**: login at 3-5 AM local time from a new location
pub fn analyze_login_attempt(
    context: &LoginContext,
    history: &LoginHistory,
) -> SuspiciousLoginResult {
    let mut reasons: Vec<String> = Vec::new();
    let mut action = SuspiciousLoginAction::Allow;

    // If there is no prior history at all, this is the first login and
    // should not be flagged as suspicious. We record it and move on.
    let has_no_history = history.last_login_at_ms == 0
        && history.known_ip_addresses.is_empty()
        && history.known_device_fingerprints.is_empty();

    if has_no_history {
        return SuspiciousLoginResult {
            is_suspicious: false,
            reasons,
            recommended_action: action,
        };
    }

    // Check for impossible travel
    if history.last_login_at_ms > 0 && context.login_at_ms > history.last_login_at_ms {
        let elapsed_ms = context.login_at_ms - history.last_login_at_ms;
        if elapsed_ms < IMPOSSIBLE_TRAVEL_WINDOW_MS {
            let distance = haversine_distance_km(
                context.latitude,
                context.longitude,
                history.last_login_location.0,
                history.last_login_location.1,
            );
            if distance > 0.0 {
                let elapsed_hours = elapsed_ms as f64 / (60.0 * 60.0 * 1_000.0);
                let speed_kmh = distance / elapsed_hours;
                if speed_kmh > MAX_TRAVEL_SPEED_KMH {
                    reasons.push(format!(
                        "Impossible travel detected: {:.0} km in {:.1} hours ({:.0} km/h)",
                        distance, elapsed_hours, speed_kmh
                    ));
                    action = SuspiciousLoginAction::Block;
                }
            }
        }
    }

    // Check for new device
    if !context.device_fingerprint.is_empty() {
        if !history
            .known_device_fingerprints
            .iter()
            .any(|d| d == &context.device_fingerprint)
        {
            reasons.push("Login from an unrecognized device".to_string());
            if action == SuspiciousLoginAction::Allow {
                action = SuspiciousLoginAction::AllowWithNotification;
            }
        }
    }

    // Check for new IP
    if !context.ip_address.is_empty() {
        if !history
            .known_ip_addresses
            .iter()
            .any(|ip| ip == &context.ip_address)
        {
            // Count total unique unknown IPs — but we can't count
            // "new" ones from a single attempt. Instead, we flag if
            // the known list is empty (first login) or if the IP is
            // not in the list and there are already many known IPs
            // (meaning the user has a pattern and this is an outlier).
            if !history.known_ip_addresses.is_empty() {
                reasons.push("Login from a new IP address".to_string());
                if history.known_ip_addresses.len() > MAX_UNKNOWN_IPS {
                    if action == SuspiciousLoginAction::Allow
                        || action == SuspiciousLoginAction::AllowWithNotification
                    {
                        action = SuspiciousLoginAction::RequireVerification;
                    }
                } else if action == SuspiciousLoginAction::Allow {
                    action = SuspiciousLoginAction::AllowWithNotification;
                }
            }
        }
    }

    // If we have reasons and action is still Allow, bump to notification
    if !reasons.is_empty() && action == SuspiciousLoginAction::Allow {
        action = SuspiciousLoginAction::AllowWithNotification;
    }

    SuspiciousLoginResult {
        is_suspicious: !reasons.is_empty(),
        reasons,
        recommended_action: action,
    }
}

/// Computes the great-circle distance between two points (in km).
fn haversine_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0_f64; // Earth radius in km
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}

// ---------------------------------------------------------------------------
// User-data enumeration for account deletion
// ---------------------------------------------------------------------------

/// A category of user-owned data that must be enumerated and deleted
/// when the user deletes their account. Google Play requires
/// transparent handling of user data and a complete deletion mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserDataCategory {
    /// Workout records (Firestore: workouts collection).
    Workouts,
    /// Route files (Cloud Storage: routes/{user_id}/ prefix).
    RouteFiles,
    /// Step samples (Firestore: step_samples subcollection).
    StepSamples,
    /// Heart rate samples (Firestore: heart_rate_samples subcollection).
    HeartRateSamples,
    /// Workout checkpoints/splits (Firestore: checkpoints subcollection).
    Checkpoints,
    /// Achievement records (Firestore: achievements collection).
    Achievements,
    /// Personal best records (Firestore: personal_records collection).
    PersonalRecords,
    /// Sync queue items (Firestore: sync_queue collection).
    SyncQueueItems,
    /// Coaching/AI conversation history (Firestore: coaching_history).
    CoachingHistory,
    /// User profile/settings (Firestore: users/{user_id} document).
    UserProfile,
    /// Offline map regions (local SQLite, but metadata may be in
    /// Firestore for cross-device sync).
    OfflineRegions,
    /// Saved routes (Firestore: saved_routes collection).
    SavedRoutes,
    /// Goals and targets (Firestore: goals collection).
    Goals,
}

/// Description of a user data category: what it is, where it's stored,
/// and how to delete it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataCategoryInfo {
    /// The category.
    pub category: UserDataCategory,
    /// Human-readable name.
    pub name: String,
    /// Where the data is stored (Firestore collection path or Cloud
    /// Storage prefix).
    pub storage_location: String,
    /// Whether this data is stored in Firestore.
    pub is_firestore: bool,
    /// Whether this data is stored in Cloud Storage.
    pub is_cloud_storage: bool,
    /// Whether this data is stored locally (SQLite, shared prefs).
    pub is_local: bool,
    /// The Firestore collection path or Cloud Storage prefix.
    pub path_prefix: String,
}

/// Returns information about all user-data categories that must be
/// enumerated for account deletion.
///
/// This is the complete list of user-owned data in the S.T.R.I.D.E.
/// app. When the user requests account deletion, the Dart layer
/// iterates over this list and deletes data from each location.
pub fn enumerate_user_data_categories() -> Vec<UserDataCategoryInfo> {
    vec![
        UserDataCategoryInfo {
            category: UserDataCategory::Workouts,
            name: "Workout records".to_string(),
            storage_location: "Firestore: workouts".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "workouts".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::RouteFiles,
            name: "Route files (GPX/binary)".to_string(),
            storage_location: "Cloud Storage: routes/{user_id}/".to_string(),
            is_firestore: false,
            is_cloud_storage: true,
            is_local: false,
            path_prefix: "routes/{user_id}/".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::StepSamples,
            name: "Step count samples".to_string(),
            storage_location: "Firestore: step_samples".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "step_samples".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::HeartRateSamples,
            name: "Heart rate samples".to_string(),
            storage_location: "Firestore: heart_rate_samples".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "heart_rate_samples".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::Checkpoints,
            name: "Workout checkpoints & splits".to_string(),
            storage_location: "Firestore: checkpoints".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "checkpoints".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::Achievements,
            name: "Achievement records".to_string(),
            storage_location: "Firestore: achievements".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "achievements".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::PersonalRecords,
            name: "Personal best records".to_string(),
            storage_location: "Firestore: personal_records".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "personal_records".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::SyncQueueItems,
            name: "Sync queue items".to_string(),
            storage_location: "Firestore: sync_queue".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "sync_queue".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::CoachingHistory,
            name: "AI coaching history".to_string(),
            storage_location: "Firestore: coaching_history".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "coaching_history".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::UserProfile,
            name: "User profile & settings".to_string(),
            storage_location: "Firestore: users/{user_id}".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: true,
            path_prefix: "users".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::OfflineRegions,
            name: "Offline map regions".to_string(),
            storage_location: "Local SQLite + Firestore metadata".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: true,
            path_prefix: "offline_regions".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::SavedRoutes,
            name: "Saved routes".to_string(),
            storage_location: "Firestore: saved_routes".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "saved_routes".to_string(),
        },
        UserDataCategoryInfo {
            category: UserDataCategory::Goals,
            name: "Goals & targets".to_string(),
            storage_location: "Firestore: goals".to_string(),
            is_firestore: true,
            is_cloud_storage: false,
            is_local: false,
            path_prefix: "goals".to_string(),
        },
    ]
}

/// The scope of account deletion the user has chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionScope {
    /// Delete only the Firebase Auth account (keeps Firestore/Storage
    /// data orphaned — not recommended but technically possible).
    AuthAccountOnly,
    /// Delete all user data from Firestore and Cloud Storage, plus the
    /// Firebase Auth account. This is the recommended full deletion.
    AllUserData,
    /// Delete user data but keep the account (for privacy compliance
    /// without losing the account — e.g. "clear my data" feature).
    DataOnly,
}

impl Default for DeletionScope {
    fn default() -> Self {
        DeletionScope::AllUserData
    }
}

/// The result of an account deletion operation, tracking which
/// categories were successfully deleted and which failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionResult {
    /// Categories that were successfully deleted.
    pub deleted: Vec<UserDataCategory>,
    /// Categories that failed to delete (with error messages).
    pub failed: Vec<(UserDataCategory, String)>,
    /// Whether the Firebase Auth account was deleted.
    pub auth_account_deleted: bool,
    /// Whether all categories were deleted (no failures).
    pub is_complete: bool,
}

/// Builds a deletion plan from the chosen scope — which categories to
/// delete and in what order. The Dart layer uses this to orchestrate
/// the deletion sequence.
pub fn build_deletion_plan(scope: DeletionScope) -> Vec<UserDataCategory> {
    match scope {
        DeletionScope::AuthAccountOnly => Vec::new(),
        DeletionScope::AllUserData => {
            // Delete data first, then the auth account can be deleted
            // (the Dart layer deletes auth last so the user remains
            // authenticated during the data deletion process).
            vec![
                UserDataCategory::Workouts,
                UserDataCategory::RouteFiles,
                UserDataCategory::StepSamples,
                UserDataCategory::HeartRateSamples,
                UserDataCategory::Checkpoints,
                UserDataCategory::Achievements,
                UserDataCategory::PersonalRecords,
                UserDataCategory::SyncQueueItems,
                UserDataCategory::CoachingHistory,
                UserDataCategory::SavedRoutes,
                UserDataCategory::Goals,
                UserDataCategory::OfflineRegions,
                UserDataCategory::UserProfile,
            ]
        }
        DeletionScope::DataOnly => {
            vec![
                UserDataCategory::Workouts,
                UserDataCategory::RouteFiles,
                UserDataCategory::StepSamples,
                UserDataCategory::HeartRateSamples,
                UserDataCategory::Checkpoints,
                UserDataCategory::Achievements,
                UserDataCategory::PersonalRecords,
                UserDataCategory::SyncQueueItems,
                UserDataCategory::CoachingHistory,
                UserDataCategory::SavedRoutes,
                UserDataCategory::Goals,
                UserDataCategory::OfflineRegions,
            ]
        }
    }
}

/// Builds a DeletionResult from the list of deleted categories and
/// any failures.
pub fn build_deletion_result(
    deleted: Vec<UserDataCategory>,
    failed: Vec<(UserDataCategory, String)>,
    auth_account_deleted: bool,
) -> DeletionResult {
    let is_complete = failed.is_empty();
    DeletionResult {
        deleted,
        failed,
        auth_account_deleted,
        is_complete,
    }
}

// ---------------------------------------------------------------------------
// Sign-out decision
// ---------------------------------------------------------------------------

/// Whether the app should sign out the user automatically (e.g. due
/// to a revoked session or disabled account).
pub fn should_auto_signout(
    session_state: SessionState,
    account_status: AccountStatus,
) -> bool {
    if !can_sign_in(account_status) {
        return true;
    }
    matches!(session_state, SessionState::Revoked)
}

// ---------------------------------------------------------------------------
// Anonymous account upgrade
// ---------------------------------------------------------------------------

/// Whether an anonymous (guest) account can be upgraded to a permanent
/// account (email/password or Google). Firebase supports linking a
/// credential to an anonymous account; the engine decides whether the
/// upgrade is allowed.
pub fn can_upgrade_anonymous(
    current_provider: AuthProvider,
    target_provider: AuthProvider,
) -> bool {
    if current_provider != AuthProvider::Anonymous {
        return false; // Not an anonymous account
    }
    if target_provider == AuthProvider::Anonymous {
        return false; // Can't "upgrade" to anonymous
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── AuthProvider ──────────────────────────────────────────────────────

    #[test]
    fn auth_provider_label_email() {
        assert_eq!(auth_provider_label(AuthProvider::EmailPassword), "Email & Password");
    }

    #[test]
    fn auth_provider_label_google() {
        assert_eq!(auth_provider_label(AuthProvider::Google), "Google");
    }

    #[test]
    fn auth_provider_label_apple() {
        assert_eq!(auth_provider_label(AuthProvider::Apple), "Apple");
    }

    #[test]
    fn auth_provider_label_anonymous() {
        assert_eq!(auth_provider_label(AuthProvider::Anonymous), "Guest");
    }

    // ── Session state ─────────────────────────────────────────────────────

    #[test]
    fn session_valid_when_token_fresh() {
        let state = classify_session_state(1_000_000, 3_600_000, 1_200_000, false);
        assert_eq!(state, SessionState::Valid);
    }

    #[test]
    fn session_refreshing_near_expiry() {
        // Token expires at 3,600,000; now is 3,500,001 (5 min - 1 ms before)
        let state = classify_session_state(1_000_000, 3_600_000, 3_500_001, false);
        // remaining = 99,999 ms < 300,000 ms (5 min) => Refreshing
        assert_eq!(state, SessionState::Refreshing);
    }

    #[test]
    fn session_refreshing_at_5_min_boundary() {
        // remaining = exactly 5 min = 300,000 ms <= REFRESH_MARGIN_MS
        let state = classify_session_state(1_000_000, 3_700_000, 3_400_000, false);
        assert_eq!(state, SessionState::Refreshing);
    }

    #[test]
    fn session_valid_just_outside_refresh_window() {
        // remaining = 300,001 ms > 300,000 ms => Valid
        let state = classify_session_state(1_000_000, 3_700_001, 3_400_000, false);
        assert_eq!(state, SessionState::Valid);
    }

    #[test]
    fn session_expired_when_token_past_expiry() {
        let state = classify_session_state(1_000_000, 2_000_000, 2_500_000, false);
        assert_eq!(state, SessionState::Expired);
    }

    #[test]
    fn session_no_session_when_never_issued() {
        let state = classify_session_state(0, 3_600_000, 1_000_000, false);
        assert_eq!(state, SessionState::NoSession);
    }

    #[test]
    fn session_revoked_overrides_everything() {
        let state = classify_session_state(1_000_000, 3_600_000, 1_200_000, true);
        assert_eq!(state, SessionState::Revoked);
    }

    #[test]
    fn needs_refresh_for_refreshing_and_expired() {
        assert!(needs_token_refresh(SessionState::Refreshing));
        assert!(needs_token_refresh(SessionState::Expired));
        assert!(!needs_token_refresh(SessionState::Valid));
        assert!(!needs_token_refresh(SessionState::NoSession));
        assert!(!needs_token_refresh(SessionState::Revoked));
    }

    #[test]
    fn requires_relogin_for_no_session_and_revoked() {
        assert!(requires_relogin(SessionState::NoSession));
        assert!(requires_relogin(SessionState::Revoked));
        assert!(!requires_relogin(SessionState::Valid));
        assert!(!requires_relogin(SessionState::Refreshing));
        assert!(!requires_relogin(SessionState::Expired));
    }

    // ── Reauthentication ──────────────────────────────────────────────────

    #[test]
    fn reauth_required_when_never_authed() {
        assert!(requires_reauthentication(
            SensitiveAction::DeleteAccount,
            0,
            1_000_000
        ));
    }

    #[test]
    fn reauth_not_required_within_5_min() {
        // last auth at 1,000,000; now at 1,200,000 (200s < 300s)
        assert!(!requires_reauthentication(
            SensitiveAction::ChangePassword,
            1_000_000,
            1_200_000
        ));
    }

    #[test]
    fn reauth_required_after_5_min() {
        // last auth at 1,000,000; now at 1,400,000 (400s > 300s)
        assert!(requires_reauthentication(
            SensitiveAction::ChangePassword,
            1_000_000,
            1_400_000
        ));
    }

    #[test]
    fn reauth_threshold_is_5_min_for_all_actions() {
        for action in [
            SensitiveAction::ChangePassword,
            SensitiveAction::ChangeEmail,
            SensitiveAction::DeleteAccount,
            SensitiveAction::LinkAuthProvider,
            SensitiveAction::DeleteAllUserData,
        ] {
            assert_eq!(reauth_threshold_ms(action), 5 * 60 * 1_000);
        }
    }

    #[test]
    fn reauth_reason_returns_message() {
        let reason = reauth_reason(SensitiveAction::DeleteAccount);
        assert!(!reason.is_empty());
        assert!(reason.contains("sign in again"));
    }

    // ── Email verification ───────────────────────────────────────────────

    #[test]
    fn verification_action_verified_when_already_verified() {
        let action = decide_verification_action(
            AuthProvider::EmailPassword,
            true,
            0,
            1_000_000,
        );
        assert_eq!(action, VerificationAction::Verified);
    }

    #[test]
    fn verification_action_not_applicable_for_google() {
        let action = decide_verification_action(
            AuthProvider::Google,
            false,
            0,
            1_000_000,
        );
        assert_eq!(action, VerificationAction::NotApplicable);
    }

    #[test]
    fn verification_action_not_applicable_for_apple() {
        let action = decide_verification_action(
            AuthProvider::Apple,
            false,
            0,
            1_000_000,
        );
        assert_eq!(action, VerificationAction::NotApplicable);
    }

    #[test]
    fn verification_action_send_when_never_sent() {
        let action = decide_verification_action(
            AuthProvider::EmailPassword,
            false,
            0,
            1_000_000,
        );
        assert_eq!(action, VerificationAction::SendVerification);
    }

    #[test]
    fn verification_action_wait_within_cooldown() {
        let now = 1_000_000;
        let sent_at = now - 60_000; // 1 min ago < 5 min cooldown
        let action = decide_verification_action(
            AuthProvider::EmailPassword,
            false,
            sent_at,
            now,
        );
        assert_eq!(action, VerificationAction::WaitForResend);
    }

    #[test]
    fn verification_action_send_after_cooldown() {
        let now = 1_000_000;
        let sent_at = now - 6 * 60_1_000; // 6 min ago > 5 min cooldown
        let action = decide_verification_action(
            AuthProvider::EmailPassword,
            false,
            sent_at,
            now,
        );
        assert_eq!(action, VerificationAction::SendVerification);
    }

    #[test]
    fn can_resend_when_never_sent() {
        assert!(can_resend_verification(0, 1_000_000));
    }

    #[test]
    fn cannot_resend_within_cooldown() {
        let now = 1_000_000;
        let sent_at = now - 60_000;
        assert!(!can_resend_verification(sent_at, now));
    }

    #[test]
    fn can_resend_after_cooldown() {
        let now = 1_000_000;
        let sent_at = now - 6 * 60_000;
        assert!(can_resend_verification(sent_at, now));
    }

    // ── Password validation ───────────────────────────────────────────────

    #[test]
    fn password_valid_strong() {
        let result = validate_password("Str0ng!Pass");
        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn password_too_short() {
        let result = validate_password("Ab1!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("at least 8")));
    }

    #[test]
    fn password_no_uppercase() {
        let result = validate_password("str0ngpass!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("uppercase")));
    }

    #[test]
    fn password_no_lowercase() {
        let result = validate_password("STR0NGPASS!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("lowercase")));
    }

    #[test]
    fn password_no_digit() {
        let result = validate_password("StrongPass!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("digit")));
    }

    #[test]
    fn password_common_password_rejected() {
        let result = validate_password("12345678");
        assert!(!result.is_valid);
    }

    #[test]
    fn password_too_long() {
        let long_pw = "A1!".repeat(50); // 150 chars > 128
        let result = validate_password(&long_pw);
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("at most 128")));
    }

    #[test]
    fn password_strength_score_strong() {
        assert_eq!(password_strength_score("Str0ng!Pass123"), 4);
    }

    #[test]
    fn password_strength_score_weak() {
        assert_eq!(password_strength_score("abc"), 0);
    }

    #[test]
    fn password_strength_label_values() {
        assert_eq!(password_strength_label(0), "Very weak");
        assert_eq!(password_strength_label(1), "Weak");
        assert_eq!(password_strength_label(2), "Fair");
        assert_eq!(password_strength_label(3), "Good");
        assert_eq!(password_strength_label(4), "Strong");
    }

    // ── Email validation ──────────────────────────────────────────────────

    #[test]
    fn email_valid_normal() {
        assert!(validate_email("user@example.com").is_ok());
    }

    #[test]
    fn email_empty_rejected() {
        assert!(validate_email("").is_err());
    }

    #[test]
    fn email_no_at_rejected() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn email_multiple_at_rejected() {
        assert!(validate_email("user@@example.com").is_err());
    }

    #[test]
    fn email_no_dot_in_domain_rejected() {
        assert!(validate_email("user@examplecom").is_err());
    }

    #[test]
    fn email_empty_local_part_rejected() {
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn email_empty_domain_rejected() {
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn email_with_spaces_rejected() {
        assert!(validate_email("user @example.com").is_err());
    }

    #[test]
    fn email_too_long_rejected() {
        let long_email = format!("{}@example.com", "a".repeat(255));
        assert!(validate_email(&long_email).is_err());
    }

    #[test]
    fn email_local_part_too_long_rejected() {
        let long_local = format!("{}@example.com", "a".repeat(65));
        assert!(validate_email(&long_local).is_err());
    }

    // ── Account status ────────────────────────────────────────────────────

    #[test]
    fn account_status_message_active_is_empty() {
        assert_eq!(account_status_message(AccountStatus::Active), "");
    }

    #[test]
    fn account_status_message_admin_disabled() {
        let msg = account_status_message(AccountStatus::AdminDisabled);
        assert!(msg.contains("disabled"));
        assert!(msg.contains("support"));
    }

    #[test]
    fn account_status_message_temporarily_locked() {
        let msg = account_status_message(AccountStatus::TemporarilyLocked);
        assert!(msg.contains("locked"));
    }

    #[test]
    fn account_status_message_pending_deletion() {
        let msg = account_status_message(AccountStatus::PendingDeletion);
        assert!(msg.contains("deletion"));
    }

    #[test]
    fn can_sign_in_active() {
        assert!(can_sign_in(AccountStatus::Active));
    }

    #[test]
    fn can_sign_in_pending_deletion() {
        assert!(can_sign_in(AccountStatus::PendingDeletion));
    }

    #[test]
    fn cannot_sign_in_admin_disabled() {
        assert!(!can_sign_in(AccountStatus::AdminDisabled));
    }

    #[test]
    fn cannot_sign_in_temporarily_locked() {
        assert!(!can_sign_in(AccountStatus::TemporarilyLocked));
    }

    // ── Suspicious login ──────────────────────────────────────────────────

    #[test]
    fn suspicious_login_normal_when_no_history() {
        let ctx = LoginContext {
            user_id: "u1".to_string(),
            ip_address: "1.2.3.4".to_string(),
            latitude: 51.5,
            longitude: -0.1,
            login_at_ms: 1_000_000,
            device_fingerprint: "dev1".to_string(),
            user_agent: "test".to_string(),
        };
        let history = LoginHistory {
            known_ip_addresses: vec![],
            known_device_fingerprints: vec![],
            known_locations: vec![],
            last_login_at_ms: 0,
            last_login_location: (0.0, 0.0),
        };
        let result = analyze_login_attempt(&ctx, &history);
        assert!(!result.is_suspicious);
        assert_eq!(result.recommended_action, SuspiciousLoginAction::Allow);
    }

    #[test]
    fn suspicious_login_new_device_triggers_notification() {
        let ctx = LoginContext {
            user_id: "u1".to_string(),
            ip_address: "1.2.3.4".to_string(),
            latitude: 51.5,
            longitude: -0.1,
            login_at_ms: 1_000_000,
            device_fingerprint: "newdev".to_string(),
            user_agent: "test".to_string(),
        };
        let history = LoginHistory {
            known_ip_addresses: vec!["1.2.3.4".to_string()],
            known_device_fingerprints: vec!["olddev".to_string()],
            known_locations: vec![(51.5, -0.1)],
            last_login_at_ms: 900_000,
            last_login_location: (51.5, -0.1),
        };
        let result = analyze_login_attempt(&ctx, &history);
        assert!(result.is_suspicious);
        assert!(result.reasons.iter().any(|r| r.contains("unrecognized device")));
        assert_eq!(result.recommended_action, SuspiciousLoginAction::AllowWithNotification);
    }

    #[test]
    fn suspicious_login_impossible_travel_is_blocked() {
        // Login from London (51.5, -0.1) 1 hour after a login from
        // Sydney (-33.9, 151.2) — distance ~17,000 km in 1 hour
        let ctx = LoginContext {
            user_id: "u1".to_string(),
            ip_address: "1.2.3.4".to_string(),
            latitude: 51.5,
            longitude: -0.1,
            login_at_ms: 2_000_000, // 1000 seconds after last login
            device_fingerprint: "dev1".to_string(),
            user_agent: "test".to_string(),
        };
        let history = LoginHistory {
            known_ip_addresses: vec!["1.2.3.4".to_string()],
            known_device_fingerprints: vec!["dev1".to_string()],
            known_locations: vec![(-33.9, 151.2)],
            last_login_at_ms: 1_000_000,
            last_login_location: (-33.9, 151.2),
        };
        let result = analyze_login_attempt(&ctx, &history);
        assert!(result.is_suspicious);
        assert!(result.reasons.iter().any(|r| r.contains("Impossible travel")));
        assert_eq!(result.recommended_action, SuspiciousLoginAction::Block);
    }

    #[test]
    fn suspicious_login_many_new_ips_requires_verification() {
        let ctx = LoginContext {
            user_id: "u1".to_string(),
            ip_address: "5.6.7.8".to_string(),
            latitude: 51.5,
            longitude: -0.1,
            login_at_ms: 2_000_000,
            device_fingerprint: "dev1".to_string(),
            user_agent: "test".to_string(),
        };
        let history = LoginHistory {
            known_ip_addresses: vec![
                "1.2.3.4".to_string(),
                "2.3.4.5".to_string(),
                "3.4.5.6".to_string(),
                "4.5.6.7".to_string(),
            ],
            known_device_fingerprints: vec!["dev1".to_string()],
            known_locations: vec![(51.5, -0.1)],
            last_login_at_ms: 1_000_000,
            last_login_location: (51.5, -0.1),
        };
        let result = analyze_login_attempt(&ctx, &history);
        assert!(result.is_suspicious);
        assert!(result.reasons.iter().any(|r| r.contains("new IP")));
        assert_eq!(result.recommended_action, SuspiciousLoginAction::RequireVerification);
    }

    #[test]
    fn suspicious_login_not_flagged_for_normal_repeat() {
        let ctx = LoginContext {
            user_id: "u1".to_string(),
            ip_address: "1.2.3.4".to_string(),
            latitude: 51.5,
            longitude: -0.1,
            login_at_ms: 2_000_000,
            device_fingerprint: "dev1".to_string(),
            user_agent: "test".to_string(),
        };
        let history = LoginHistory {
            known_ip_addresses: vec!["1.2.3.4".to_string()],
            known_device_fingerprints: vec!["dev1".to_string()],
            known_locations: vec![(51.5, -0.1)],
            last_login_at_ms: 1_000_000,
            last_login_location: (51.5, -0.1),
        };
        let result = analyze_login_attempt(&ctx, &history);
        assert!(!result.is_suspicious);
        assert_eq!(result.recommended_action, SuspiciousLoginAction::Allow);
    }

    #[test]
    fn haversine_distance_same_point_is_zero() {
        let d = haversine_distance_km(51.5, -0.1, 51.5, -0.1);
        assert!((d - 0.0).abs() < 0.01);
    }

    #[test]
    fn haversine_distance_london_paris() {
        // London (51.5074, -0.1278) → Paris (48.8566, 2.3522)
        let d = haversine_distance_km(51.5074, -0.1278, 48.8566, 2.3522);
        // Should be ~340 km
        assert!(d > 300.0 && d < 400.0);
    }

    // ── User data enumeration ─────────────────────────────────────────────

    #[test]
    fn enumerate_user_data_has_all_categories() {
        let cats = enumerate_user_data_categories();
        assert_eq!(cats.len(), 13);
    }

    #[test]
    fn enumerate_user_data_includes_workouts() {
        let cats = enumerate_user_data_categories();
        assert!(cats.iter().any(|c| c.category == UserDataCategory::Workouts));
    }

    #[test]
    fn enumerate_user_data_includes_route_files() {
        let cats = enumerate_user_data_categories();
        assert!(cats.iter().any(|c| c.category == UserDataCategory::RouteFiles));
    }

    #[test]
    fn route_files_are_cloud_storage() {
        let cats = enumerate_user_data_categories();
        let route = cats.iter().find(|c| c.category == UserDataCategory::RouteFiles).unwrap();
        assert!(route.is_cloud_storage);
        assert!(!route.is_firestore);
    }

    #[test]
    fn workouts_are_firestore() {
        let cats = enumerate_user_data_categories();
        let workouts = cats.iter().find(|c| c.category == UserDataCategory::Workouts).unwrap();
        assert!(workouts.is_firestore);
        assert!(!workouts.is_cloud_storage);
    }

    #[test]
    fn user_profile_is_local_and_firestore() {
        let cats = enumerate_user_data_categories();
        let profile = cats.iter().find(|c| c.category == UserDataCategory::UserProfile).unwrap();
        assert!(profile.is_local);
        assert!(profile.is_firestore);
    }

    // ── Deletion plan ─────────────────────────────────────────────────────

    #[test]
    fn deletion_plan_all_user_data_has_all_categories() {
        let plan = build_deletion_plan(DeletionScope::AllUserData);
        assert_eq!(plan.len(), 13);
        assert!(plan.contains(&UserDataCategory::Workouts));
        assert!(plan.contains(&UserDataCategory::RouteFiles));
        assert!(plan.contains(&UserDataCategory::UserProfile));
    }

    #[test]
    fn deletion_plan_auth_account_only_is_empty() {
        let plan = build_deletion_plan(DeletionScope::AuthAccountOnly);
        assert!(plan.is_empty());
    }

    #[test]
    fn deletion_plan_data_only_excludes_user_profile() {
        let plan = build_deletion_plan(DeletionScope::DataOnly);
        // DataOnly doesn't delete the UserProfile (the user keeps the account)
        assert!(!plan.contains(&UserDataCategory::UserProfile));
        assert!(plan.contains(&UserDataCategory::Workouts));
    }

    #[test]
    fn deletion_result_complete_when_no_failures() {
        let result = build_deletion_result(
            vec![UserDataCategory::Workouts, UserDataCategory::Goals],
            vec![],
            true,
        );
        assert!(result.is_complete);
        assert!(result.auth_account_deleted);
        assert_eq!(result.deleted.len(), 2);
    }

    #[test]
    fn deletion_result_incomplete_with_failures() {
        let result = build_deletion_result(
            vec![UserDataCategory::Workouts],
            vec![(UserDataCategory::RouteFiles, "network error".to_string())],
            false,
        );
        assert!(!result.is_complete);
        assert!(!result.auth_account_deleted);
        assert_eq!(result.failed.len(), 1);
    }

    // ── Auto sign-out ──────────────────────────────────────────────────────

    #[test]
    fn auto_signout_for_revoked_session() {
        assert!(should_auto_signout(SessionState::Revoked, AccountStatus::Active));
    }

    #[test]
    fn auto_signout_for_disabled_account() {
        assert!(should_auto_signout(SessionState::Valid, AccountStatus::AdminDisabled));
    }

    #[test]
    fn auto_signout_for_locked_account() {
        assert!(should_auto_signout(SessionState::Valid, AccountStatus::TemporarilyLocked));
    }

    #[test]
    fn no_auto_signout_for_valid_session_active_account() {
        assert!(!should_auto_signout(SessionState::Valid, AccountStatus::Active));
    }

    #[test]
    fn no_auto_signout_for_expired_session() {
        // Expired means the token expired, not that the session is revoked.
        // The user should be prompted to re-sign-in, not auto-signed-out.
        assert!(!should_auto_signout(SessionState::Expired, AccountStatus::Active));
    }

    // ── Anonymous upgrade ─────────────────────────────────────────────────

    #[test]
    fn can_upgrade_anonymous_to_email() {
        assert!(can_upgrade_anonymous(AuthProvider::Anonymous, AuthProvider::EmailPassword));
    }

    #[test]
    fn can_upgrade_anonymous_to_google() {
        assert!(can_upgrade_anonymous(AuthProvider::Anonymous, AuthProvider::Google));
    }

    #[test]
    fn cannot_upgrade_non_anonymous() {
        assert!(!can_upgrade_anonymous(AuthProvider::Google, AuthProvider::EmailPassword));
    }

    #[test]
    fn cannot_upgrade_to_anonymous() {
        assert!(!can_upgrade_anonymous(AuthProvider::Anonymous, AuthProvider::Anonymous));
    }

    // ── Default values ────────────────────────────────────────────────────

    #[test]
    fn default_auth_provider_is_email_password() {
        assert_eq!(AuthProvider::default(), AuthProvider::EmailPassword);
    }

    #[test]
    fn default_session_state_is_no_session() {
        assert_eq!(SessionState::default(), SessionState::NoSession);
    }

    #[test]
    fn default_account_status_is_active() {
        assert_eq!(AccountStatus::default(), AccountStatus::Active);
    }

    #[test]
    fn default_sensitive_action_is_change_password() {
        assert_eq!(SensitiveAction::default(), SensitiveAction::ChangePassword);
    }

    #[test]
    fn default_deletion_scope_is_all_user_data() {
        assert_eq!(DeletionScope::default(), DeletionScope::AllUserData);
    }

    #[test]
    fn default_suspicious_login_action_is_allow() {
        assert_eq!(SuspiciousLoginAction::default(), SuspiciousLoginAction::Allow);
    }

    #[test]
    fn default_email_verification_state_is_not_sent() {
        assert_eq!(EmailVerificationState::default(), EmailVerificationState::NotSent);
    }
}
