//! Achievement and badge engine (spec section 29).
//!
//! Pure function(s) that decide which one-time achievements/badges a
//! completed, validated workout newly qualifies for. Per the spec: "The
//! engine should emit achievement events, but the actual achievement
//! service may be separate" — this module only decides *which* badges are
//! newly earned; delivering/rendering them (toast, notification, profile
//! badge screen) is entirely a Dart/UI concern, same division of
//! responsibility as `personal_records.rs`.
//!
//! The engine has no storage of its own, so the caller (Dart) is
//! responsible for:
//!   1. loading an [`AchievementHistory`] snapshot (lifetime totals, which
//!      awards have already been granted, current streak) from local/cloud
//!      storage before calling [`detect_achievements`],
//!   2. persisting any newly-returned awards so `already_awarded` reflects
//!      them on every future call — this is what "must prevent the same
//!      one-time award from being issued repeatedly" (spec section 29)
//!      actually requires, since this module is stateless between calls.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::models::{ActivityType, WorkoutSummary};

/// One-time badge categories the engine can detect. See spec section 29.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AchievementType {
    FirstWalk,
    FirstRun,
    FirstMile,
    First5k,
    SevenDayStreak,
    TenTotalMiles,
    FiftyTotalMiles,
    NewPaceRecord,
    WeeklyGoalCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementEvent {
    pub achievement_type: AchievementType,
    pub workout_id: String,
    pub achieved_at: i64,
}

/// Lifetime context needed to decide badge eligibility, as known
/// immediately *before* this workout (this workout's own totals are
/// folded in by [`detect_achievements`] itself, not by the caller).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AchievementHistory {
    pub total_walks: u32,
    pub total_runs: u32,
    /// Lifetime distance across all completed workouts, not including the
    /// workout currently being evaluated.
    pub lifetime_distance_meters_before: f64,
    /// Consecutive-day streak count *not* including today, so this
    /// workout would extend it to `current_streak_days + 1` if today
    /// hadn't already been logged. The caller (Dart) owns the actual
    /// calendar-day bookkeeping; this module just compares against the
    /// 7-day threshold.
    pub current_streak_days: u32,
    /// Achievement types already granted at any point in the past — never
    /// re-issued regardless of whether criteria are met again. This is the
    /// mechanism that satisfies the spec's "must prevent the same one-time
    /// award from being issued repeatedly" requirement.
    #[serde(default)]
    pub already_awarded: HashSet<AchievementType>,
}

const METERS_PER_MILE: f64 = 1609.344;
const FIVE_K_METERS: f64 = 5000.0;

/// Decides which new one-time achievements this completed, valid workout
/// unlocks. Only ever call this for validated, non-blocked workouts (spec
/// section 26) — same constraint as `personal_records::detect_records`.
///
/// `weekly_goal_completed_by_this_workout` and `new_pace_record_set` are
/// supplied by the caller because they depend on state this module
/// doesn't own (the active weekly goal, and the result of
/// `personal_records::detect_records` for this same workout).
pub fn detect_achievements(
    summary: &WorkoutSummary,
    history: &AchievementHistory,
    weekly_goal_completed_by_this_workout: bool,
    new_pace_record_set: bool,
) -> Vec<AchievementEvent> {
    let mut out = Vec::new();
    let achieved_at = summary.ended_at;
    let workout_id = summary.workout_id.clone();

    let award = |out: &mut Vec<AchievementEvent>, t: AchievementType| {
        if !history.already_awarded.contains(&t) {
            out.push(AchievementEvent {
                achievement_type: t,
                workout_id: workout_id.clone(),
                achieved_at,
            });
        }
    };

    if history.total_walks == 0 && summary.activity_type == ActivityType::Walk {
        award(&mut out, AchievementType::FirstWalk);
    }
    if history.total_runs == 0 && summary.activity_type == ActivityType::Run {
        award(&mut out, AchievementType::FirstRun);
    }
    if summary.distance_meters >= METERS_PER_MILE {
        award(&mut out, AchievementType::FirstMile);
    }
    if summary.distance_meters >= FIVE_K_METERS {
        award(&mut out, AchievementType::First5k);
    }
    if history.current_streak_days + 1 >= 7 {
        award(&mut out, AchievementType::SevenDayStreak);
    }
    let lifetime_after = history.lifetime_distance_meters_before + summary.distance_meters;
    if lifetime_after >= 10.0 * METERS_PER_MILE {
        award(&mut out, AchievementType::TenTotalMiles);
    }
    if lifetime_after >= 50.0 * METERS_PER_MILE {
        award(&mut out, AchievementType::FiftyTotalMiles);
    }
    if new_pace_record_set {
        award(&mut out, AchievementType::NewPaceRecord);
    }
    if weekly_goal_completed_by_this_workout {
        award(&mut out, AchievementType::WeeklyGoalCompleted);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ActivityType;

    fn base_summary(distance: f64, activity: ActivityType) -> WorkoutSummary {
        WorkoutSummary {
            workout_id: "w1".into(),
            user_id: "u1".into(),
            activity_type: activity,
            started_at: 0,
            ended_at: 1_000_000,
            total_duration_ms: 1_800_000,
            moving_ms: 1_700_000,
            paused_ms: 100_000,
            distance_meters: distance,
            average_pace_sec_per_km: 300.0,
            average_speed_mps: 3.3,
            max_speed_mps: 4.5,
            calories_estimated: 350.0,
            calorie_estimate_method: "met".into(),
            step_count: 6000,
            average_cadence_spm: 165.0,
            average_heart_rate_bpm: None,
            max_heart_rate_bpm: None,
            min_heart_rate_bpm: None,
            elevation_gain_meters: 40.0,
            elevation_loss_meters: 35.0,
            splits: vec![],
            start_latitude: None,
            start_longitude: None,
            end_latitude: None,
            end_longitude: None,
            encoded_polyline: None,
            goal_result: None,
            has_flagged_segments: false,
            validation_warnings: vec![],
            is_blocked: false,
            block_reasons: vec![],
        }
    }

    #[test]
    fn first_walk_ever_is_awarded() {
        let summary = base_summary(500.0, ActivityType::Walk);
        let history = AchievementHistory::default();
        let events = detect_achievements(&summary, &history, false, false);
        assert!(events
            .iter()
            .any(|e| e.achievement_type == AchievementType::FirstWalk));
    }

    #[test]
    fn first_walk_not_reawarded_if_already_granted() {
        let summary = base_summary(500.0, ActivityType::Walk);
        let mut history = AchievementHistory::default();
        history.already_awarded.insert(AchievementType::FirstWalk);
        let events = detect_achievements(&summary, &history, false, false);
        assert!(!events
            .iter()
            .any(|e| e.achievement_type == AchievementType::FirstWalk));
    }

    #[test]
    fn first_walk_not_awarded_if_prior_walks_exist() {
        let summary = base_summary(500.0, ActivityType::Walk);
        let history = AchievementHistory {
            total_walks: 3,
            ..Default::default()
        };
        let events = detect_achievements(&summary, &history, false, false);
        assert!(!events
            .iter()
            .any(|e| e.achievement_type == AchievementType::FirstWalk));
    }

    #[test]
    fn first_mile_and_5k_awarded_together_when_qualifying() {
        let summary = base_summary(6000.0, ActivityType::Run);
        let history = AchievementHistory {
            total_runs: 5,
            ..Default::default()
        };
        let events = detect_achievements(&summary, &history, false, false);
        let types: Vec<_> = events.iter().map(|e| e.achievement_type).collect();
        assert!(types.contains(&AchievementType::FirstMile));
        assert!(types.contains(&AchievementType::First5k));
    }

    #[test]
    fn seven_day_streak_awarded_when_extending_to_seven() {
        let summary = base_summary(500.0, ActivityType::Walk);
        let history = AchievementHistory {
            total_walks: 6,
            current_streak_days: 6,
            ..Default::default()
        };
        let events = detect_achievements(&summary, &history, false, false);
        assert!(events
            .iter()
            .any(|e| e.achievement_type == AchievementType::SevenDayStreak));
    }

    #[test]
    fn ten_and_fifty_mile_lifetime_milestones() {
        let summary = base_summary(20_000.0, ActivityType::Run); // ~12.4 miles this workout
        let history = AchievementHistory {
            total_runs: 10,
            lifetime_distance_meters_before: 0.0,
            ..Default::default()
        };
        let events = detect_achievements(&summary, &history, false, false);
        let types: Vec<_> = events.iter().map(|e| e.achievement_type).collect();
        assert!(types.contains(&AchievementType::TenTotalMiles));
        assert!(!types.contains(&AchievementType::FiftyTotalMiles));
    }

    #[test]
    fn pace_record_and_weekly_goal_pass_through_from_caller() {
        let summary = base_summary(500.0, ActivityType::Walk);
        let history = AchievementHistory {
            total_walks: 5,
            ..Default::default()
        };
        let events = detect_achievements(&summary, &history, true, true);
        let types: Vec<_> = events.iter().map(|e| e.achievement_type).collect();
        assert!(types.contains(&AchievementType::NewPaceRecord));
        assert!(types.contains(&AchievementType::WeeklyGoalCompleted));
    }

    #[test]
    fn short_workout_with_full_history_awards_nothing_already_granted() {
        let summary = base_summary(200.0, ActivityType::Walk);
        let mut already = HashSet::new();
        already.insert(AchievementType::FirstWalk);
        already.insert(AchievementType::FirstRun);
        already.insert(AchievementType::TenTotalMiles);
        already.insert(AchievementType::FiftyTotalMiles);
        let history = AchievementHistory {
            total_walks: 10,
            total_runs: 10,
            lifetime_distance_meters_before: 100_000.0,
            current_streak_days: 1,
            already_awarded: already,
        };
        let events = detect_achievements(&summary, &history, false, false);
        assert!(events.is_empty());
    }
}
