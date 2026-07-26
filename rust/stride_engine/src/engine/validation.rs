//! Workout validation (spec section 26).
//!
//! Runs a battery of sanity checks against a near-final workout before it
//! is allowed to be saved. Per the spec, legitimate unusual workouts
//! should generally still be *allowed* to save — this module produces
//! non-blocking warnings for most conditions, and only a small number of
//! hard-blocking failures (missing timestamps, corrupt/negative totals).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationInput {
    pub started_at: i64,
    pub ended_at: i64,
    pub distance_meters: f64,
    pub duration_ms: i64,
    pub average_speed_mps: f64,
    pub max_speed_mps: f64,
    pub calories_estimated: f64,
    pub route_point_count: u32,
    pub has_vehicle_flagged_segment: bool,
    pub gps_gap_count: u32,
    pub is_duplicate_of_existing_workout: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_blocked: bool,
    pub block_reasons: Vec<String>,
    pub warnings: Vec<String>,
}

/// Hard ceiling used to flag (not necessarily block) implausible speeds;
/// above elite marathon pace by a wide margin. Kept generous because
/// legitimate downhill running or GPS-corrected sprints can spike briefly.
const IMPLAUSIBLE_SPEED_MPS: f64 = 12.5;
/// Calorie-per-hour ceiling above which we flag "excessive calorie
/// estimate" as a warning (roughly 2x elite ultramarathon burn rates).
const EXCESSIVE_KCAL_PER_HOUR: f64 = 1800.0;

pub fn validate(input: &ValidationInput) -> ValidationResult {
    let mut block_reasons = Vec::new();
    let mut warnings = Vec::new();

    // --- Hard blocks: data integrity problems that make the record unsafe
    // to persist at all. ---
    if input.started_at <= 0 || input.ended_at <= 0 {
        block_reasons.push("Missing start or end time.".to_string());
    }
    if input.ended_at > 0 && input.started_at > 0 && input.ended_at < input.started_at {
        block_reasons.push("End time precedes start time.".to_string());
    }
    if input.distance_meters < 0.0 || input.duration_ms < 0 || input.calories_estimated < 0.0 {
        block_reasons.push("Negative totals detected (corrupt record).".to_string());
    }
    if !input.distance_meters.is_finite()
        || !input.average_speed_mps.is_finite()
        || !input.calories_estimated.is_finite()
    {
        block_reasons.push("Non-finite (NaN/Infinity) value detected in totals.".to_string());
    }
    if input.is_duplicate_of_existing_workout {
        block_reasons.push("This workout appears to be a duplicate of an existing record.".to_string());
    }

    // --- Soft warnings: unusual but not disqualifying. ---
    if input.distance_meters == 0.0 && input.duration_ms > 0 {
        warnings.push("Zero-distance workout.".to_string());
    }
    if input.duration_ms > 0 && input.duration_ms < 30_000 {
        warnings.push("Very short workout (under 30 seconds).".to_string());
    }
    if input.average_speed_mps > IMPLAUSIBLE_SPEED_MPS
        || input.max_speed_mps > IMPLAUSIBLE_SPEED_MPS
    {
        warnings.push("Unusually high speed detected for a walk/run.".to_string());
    }
    if input.route_point_count == 0 && input.distance_meters > 0.0 {
        warnings.push("Distance recorded without any route points (corrupt route data).".to_string());
    }
    if input.gps_gap_count > 3 {
        warnings.push(format!(
            "{} significant GPS gaps detected during this workout.",
            input.gps_gap_count
        ));
    }
    if input.has_vehicle_flagged_segment {
        warnings.push("One or more segments looked like vehicle motion.".to_string());
    }
    if input.duration_ms > 0 {
        let hours = input.duration_ms as f64 / 3_600_000.0;
        if hours > 0.0 {
            let kcal_per_hour = input.calories_estimated / hours;
            if kcal_per_hour > EXCESSIVE_KCAL_PER_HOUR {
                warnings.push("Calorie estimate is unusually high for the recorded duration.".to_string());
            }
        }
    }

    ValidationResult {
        is_blocked: !block_reasons.is_empty(),
        block_reasons,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_input() -> ValidationInput {
        ValidationInput {
            started_at: 1_000,
            ended_at: 1_800_000,
            distance_meters: 3000.0,
            duration_ms: 1_799_000,
            average_speed_mps: 1.8,
            max_speed_mps: 2.5,
            calories_estimated: 200.0,
            route_point_count: 400,
            has_vehicle_flagged_segment: false,
            gps_gap_count: 0,
            is_duplicate_of_existing_workout: false,
        }
    }

    #[test]
    fn valid_workout_has_no_blocks_or_warnings() {
        let result = validate(&good_input());
        assert!(!result.is_blocked);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn missing_timestamps_blocks() {
        let mut input = good_input();
        input.started_at = 0;
        let result = validate(&input);
        assert!(result.is_blocked);
    }

    #[test]
    fn negative_totals_block() {
        let mut input = good_input();
        input.distance_meters = -10.0;
        let result = validate(&input);
        assert!(result.is_blocked);
    }

    #[test]
    fn duplicate_workout_blocks() {
        let mut input = good_input();
        input.is_duplicate_of_existing_workout = true;
        let result = validate(&input);
        assert!(result.is_blocked);
    }

    #[test]
    fn zero_distance_warns_but_does_not_block() {
        let mut input = good_input();
        input.distance_meters = 0.0;
        input.route_point_count = 0;
        let result = validate(&input);
        assert!(!result.is_blocked);
        assert!(result.warnings.iter().any(|w| w.contains("Zero-distance")));
    }

    #[test]
    fn vehicle_flag_produces_warning_not_block() {
        let mut input = good_input();
        input.has_vehicle_flagged_segment = true;
        let result = validate(&input);
        assert!(!result.is_blocked);
        assert!(result.warnings.iter().any(|w| w.contains("vehicle")));
    }

    #[test]
    fn end_before_start_blocks() {
        let mut input = good_input();
        input.ended_at = 500;
        let result = validate(&input);
        assert!(result.is_blocked);
    }
}
