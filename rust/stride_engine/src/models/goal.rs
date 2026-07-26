use serde::{Deserialize, Serialize};

/// The kind of target a workout may be tracked against. See section 18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    Distance,
    Duration,
    Calories,
    Pace,
    Steps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutGoal {
    pub goal_type: GoalType,
    /// Target value in canonical units (meters / ms / kcal / sec-per-km / steps).
    pub target_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgress {
    pub goal_type: GoalType,
    pub target_value: f64,
    pub current_value: f64,
    pub percent_complete: f64,
    pub remaining_value: f64,
    pub is_complete: bool,
    /// Only meaningful for distance/duration goals: the pace (sec/km)
    /// the user needs to hold to just reach the target, if applicable.
    pub pace_needed_sec_per_km: Option<f64>,
}

impl WorkoutGoal {
    /// Computes live progress against this goal.
    ///
    /// `current_value` must already be in the same canonical unit as
    /// `target_value` (e.g. meters for a distance goal).
    pub fn progress(
        &self,
        current_value: f64,
        elapsed_ms: i64,
        distance_meters: f64,
    ) -> GoalProgress {
        let target = self.target_value.max(0.0);
        let current = current_value.max(0.0);
        let percent = if target > 0.0 {
            (current / target * 100.0).min(999.0)
        } else {
            0.0
        };
        let remaining = (target - current).max(0.0);
        let is_complete = target > 0.0 && current >= target;

        let pace_needed = match self.goal_type {
            GoalType::Duration if distance_meters > 0.0 && remaining > 0.0 => {
                // remaining time / remaining distance-equivalent isn't well
                // defined without a distance target; skip.
                None
            }
            GoalType::Distance if !is_complete => {
                // pace needed to *maintain* to reach distance goal is only
                // meaningful if caller supplies a time budget; we instead
                // report current average pace extrapolation via elapsed_ms.
                if elapsed_ms > 0 && current > 0.0 {
                    let current_pace_sec_per_km = (elapsed_ms as f64 / 1000.0) / (current / 1000.0);
                    Some(current_pace_sec_per_km)
                } else {
                    None
                }
            }
            _ => None,
        };

        GoalProgress {
            goal_type: self.goal_type,
            target_value: target,
            current_value: current,
            percent_complete: percent,
            remaining_value: remaining,
            is_complete,
            pace_needed_sec_per_km: pace_needed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_goal_progress_basic() {
        let goal = WorkoutGoal {
            goal_type: GoalType::Distance,
            target_value: 3000.0, // 3 km target
        };
        let progress = goal.progress(2400.0, 20 * 60 * 1000, 2400.0);
        assert!((progress.percent_complete - 80.0).abs() < 1e-6);
        assert!((progress.remaining_value - 600.0).abs() < 1e-6);
        assert!(!progress.is_complete);
    }

    #[test]
    fn distance_goal_marks_complete_at_target() {
        let goal = WorkoutGoal {
            goal_type: GoalType::Distance,
            target_value: 1000.0,
        };
        let progress = goal.progress(1000.0, 10 * 60 * 1000, 1000.0);
        assert!(progress.is_complete);
        assert_eq!(progress.remaining_value, 0.0);
    }

    #[test]
    fn zero_target_does_not_divide_by_zero() {
        let goal = WorkoutGoal {
            goal_type: GoalType::Calories,
            target_value: 0.0,
        };
        let progress = goal.progress(50.0, 1000, 0.0);
        assert_eq!(progress.percent_complete, 0.0);
        assert!(progress.percent_complete.is_finite());
    }
}
