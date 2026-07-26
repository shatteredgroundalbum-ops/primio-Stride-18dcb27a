//! Permissions manager (spec section 30).
//!
//! This module holds pure state/logic for what the engine believes the
//! current permission situation is, and what the app should do about it.
//! The actual OS-level permission prompts must happen on the Dart/native
//! side (Rust has no access to platform permission APIs) — this module
//! just centralizes the decision logic so it's consistent and testable.

use serde::{Deserialize, Serialize};

// NOTE for FFI callers (Dart): every enum below serializes with
// `#[serde(rename_all = "snake_case")]`, e.g. `PermissionState::NotRequested`
// <-> `"not_requested"`, `PermissionContext::DuringActiveWorkout` <->
// `"during_active_workout"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    NotRequested,
    Granted,
    Denied,
    /// User denied and checked "don't ask again" (Android) / has denied
    /// enough times that the OS won't show the prompt again (iOS).
    PermanentlyDenied,
    /// Was granted, but the OS/user revoked it after the workout started.
    RevokedDuringUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    PreciseLocation,
    BackgroundLocation,
    PhysicalActivity,
    Notifications,
    Bluetooth,
    NearbyDevices,
    HealthData,
    BatteryOptimizationExemption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredAction {
    /// Nothing to do; feature can proceed.
    None,
    /// Show the in-app rationale, then trigger the OS prompt.
    ShowRationaleThenRequest,
    /// Direct OS prompt without extra rationale (state is fresh).
    RequestDirectly,
    /// Permission is permanently denied; must deep-link to system settings.
    OpenAppSettings,
    /// Feature should degrade gracefully rather than block the user
    /// entirely (e.g. no background location -> foreground-only tracking).
    UseLimitedFallback,
    /// Permission was revoked mid-workout; pause tracking and prompt.
    PauseAndPromptReRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionContext {
    /// First time the user is being asked (no prior denial).
    Initial,
    /// User denied once already.
    DeniedOnce,
    /// A workout is currently in progress.
    DuringActiveWorkout,
}

pub fn required_action(
    state: PermissionState,
    context: PermissionContext,
) -> RequiredAction {
    match (state, context) {
        (PermissionState::Granted, _) => RequiredAction::None,
        (PermissionState::PermanentlyDenied, _) => RequiredAction::OpenAppSettings,
        (PermissionState::RevokedDuringUse, PermissionContext::DuringActiveWorkout) => {
            RequiredAction::PauseAndPromptReRequest
        }
        (PermissionState::NotRequested, PermissionContext::Initial) => {
            RequiredAction::RequestDirectly
        }
        (PermissionState::Denied, PermissionContext::DeniedOnce) => {
            RequiredAction::ShowRationaleThenRequest
        }
        (PermissionState::Denied, _) => RequiredAction::ShowRationaleThenRequest,
        _ => RequiredAction::UseLimitedFallback,
    }
}

/// Given the set of currently-granted permissions, determines whether the
/// workout can start in full-featured mode, foreground-only fallback, or
/// must be blocked entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkoutStartCapability {
    FullFeatured,
    ForegroundOnlyFallback,
    Blocked,
}

pub fn evaluate_start_capability(
    precise_location: PermissionState,
    background_location: PermissionState,
) -> WorkoutStartCapability {
    if !matches!(
        precise_location,
        PermissionState::Granted
    ) {
        return WorkoutStartCapability::Blocked;
    }
    if matches!(background_location, PermissionState::Granted) {
        WorkoutStartCapability::FullFeatured
    } else {
        WorkoutStartCapability::ForegroundOnlyFallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn granted_permission_requires_no_action() {
        assert_eq!(
            required_action(PermissionState::Granted, PermissionContext::Initial),
            RequiredAction::None
        );
    }

    #[test]
    fn permanently_denied_requires_opening_settings() {
        assert_eq!(
            required_action(PermissionState::PermanentlyDenied, PermissionContext::Initial),
            RequiredAction::OpenAppSettings
        );
    }

    #[test]
    fn revoked_during_workout_pauses_and_reprompts() {
        assert_eq!(
            required_action(
                PermissionState::RevokedDuringUse,
                PermissionContext::DuringActiveWorkout
            ),
            RequiredAction::PauseAndPromptReRequest
        );
    }

    #[test]
    fn first_time_request_is_direct() {
        assert_eq!(
            required_action(PermissionState::NotRequested, PermissionContext::Initial),
            RequiredAction::RequestDirectly
        );
    }

    #[test]
    fn denied_once_shows_rationale_before_reasking() {
        assert_eq!(
            required_action(PermissionState::Denied, PermissionContext::DeniedOnce),
            RequiredAction::ShowRationaleThenRequest
        );
    }

    #[test]
    fn full_featured_requires_both_permissions() {
        let cap = evaluate_start_capability(PermissionState::Granted, PermissionState::Granted);
        assert_eq!(cap, WorkoutStartCapability::FullFeatured);
    }

    #[test]
    fn foreground_only_fallback_without_background_location() {
        let cap = evaluate_start_capability(PermissionState::Granted, PermissionState::Denied);
        assert_eq!(cap, WorkoutStartCapability::ForegroundOnlyFallback);
    }

    #[test]
    fn blocked_without_precise_location() {
        let cap = evaluate_start_capability(PermissionState::Denied, PermissionState::Granted);
        assert_eq!(cap, WorkoutStartCapability::Blocked);
    }
}
