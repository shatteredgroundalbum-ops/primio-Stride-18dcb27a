//! Calorie estimation engine (spec section 15).
//!
//! Calories are always labeled as an *estimate* and tagged with the method
//! that produced them, per a strict source-priority order:
//!
//!   1. wearable   - device-reported active-energy value
//!   2. heart_rate - HR-based formula (Keytel et al.)
//!   3. met        - activity MET table × weight × time
//!   4. distance_weight - crude fallback using distance & weight only
//!
//! `estimate` never panics on missing inputs — it degrades gracefully to
//! the next lower-priority method and always returns a finite,
//! non-negative value.

use serde::{Deserialize, Serialize};

use crate::models::ActivityType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalorieMethod {
    Wearable,
    HeartRate,
    Met,
    DistanceWeight,
}

impl CalorieMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            CalorieMethod::Wearable => "wearable",
            CalorieMethod::HeartRate => "heart_rate",
            CalorieMethod::Met => "met",
            CalorieMethod::DistanceWeight => "distance_weight",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CalorieInputs {
    /// Device/wearable-reported active energy in kcal, if available and
    /// trusted (e.g. from Health Connect / Wear OS).
    pub wearable_kcal: Option<f64>,
    pub weight_kg: Option<f64>,
    pub duration_ms: i64,
    pub distance_meters: f64,
    pub average_speed_mps: f64,
    /// Average heart rate (bpm) during the interval, if available.
    pub average_heart_rate_bpm: Option<f64>,
    pub age_years: Option<u16>,
    /// `true` = male, `false` = female, `None` = not provided. Only used
    /// by the heart-rate formula; never required.
    pub is_male: Option<bool>,
    pub activity_type: ActivityType,
    pub elevation_gain_meters: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct CalorieEstimate {
    pub kcal: f64,
    pub method: CalorieMethod,
}

/// MET (Metabolic Equivalent of Task) lookup by activity type and speed,
/// following commonly published compendium-of-physical-activities values.
fn met_value(activity: ActivityType, speed_mps: f64) -> f64 {
    let kmh = speed_mps * 3.6;
    match activity {
        ActivityType::Walk | ActivityType::AutoDetect => {
            if kmh < 3.2 {
                2.8
            } else if kmh < 4.8 {
                3.5
            } else if kmh < 5.6 {
                4.3
            } else if kmh < 6.4 {
                5.0
            } else {
                7.0 // brisk/race walking
            }
        }
        ActivityType::Run => {
            if kmh < 8.0 {
                8.3
            } else if kmh < 9.7 {
                9.8
            } else if kmh < 11.3 {
                11.0
            } else if kmh < 12.9 {
                11.8
            } else if kmh < 14.5 {
                12.8
            } else {
                14.5
            }
        }
        ActivityType::Hike => 6.0,
    }
}

/// Adds a modest grade adjustment to the MET estimate based on cumulative
/// elevation gain relative to distance (a crude but bounded correction —
/// real grade-adjusted pace models are more precise but need per-point
/// grade, which belongs in a future server-side correction pass per the
/// spec's "optional server-side elevation correction" note).
fn grade_adjustment_factor(elevation_gain_m: f64, distance_m: f64) -> f64 {
    if distance_m <= 0.0 {
        return 1.0;
    }
    let grade_pct = (elevation_gain_m / distance_m) * 100.0;
    // +2% MET per 1% average grade, capped to avoid runaway values from
    // noisy elevation data.
    (1.0 + (grade_pct.clamp(0.0, 15.0) * 0.02)).clamp(1.0, 1.3)
}

/// Heart-rate based estimate (Keytel et al. 2005 regression), kcal/min:
///   Male:   ((-55.0969 + 0.6309*HR + 0.1988*W + 0.2017*A) / 4.184)
///   Female: ((-20.4022 + 0.4472*HR - 0.1263*W + 0.074*A) / 4.184)
/// Falls back to a sex-neutral average of both formulas if sex is unknown.
fn heart_rate_kcal_per_min(hr: f64, weight_kg: f64, age: f64, is_male: Option<bool>) -> f64 {
    let male = (-55.0969 + 0.6309 * hr + 0.1988 * weight_kg + 0.2017 * age) / 4.184;
    let female = (-20.4022 + 0.4472 * hr - 0.1263 * weight_kg + 0.074 * age) / 4.184;
    let value = match is_male {
        Some(true) => male,
        Some(false) => female,
        None => (male + female) / 2.0,
    };
    value.max(0.0)
}

/// Produces the best available calorie estimate for the given inputs,
/// following the documented source-priority order. Always returns a
/// finite, non-negative kcal value.
pub fn estimate(inputs: &CalorieInputs) -> CalorieEstimate {
    // 1. Wearable-reported value, if present and sane.
    if let Some(kcal) = inputs.wearable_kcal {
        if kcal.is_finite() && kcal >= 0.0 {
            return CalorieEstimate {
                kcal,
                method: CalorieMethod::Wearable,
            };
        }
    }

    let duration_min = inputs.duration_ms.max(0) as f64 / 60_000.0;

    // 2. Heart-rate based, if we have HR + weight + age.
    if let (Some(hr), Some(weight), Some(age)) = (
        inputs.average_heart_rate_bpm,
        inputs.weight_kg,
        inputs.age_years,
    ) {
        if hr.is_finite() && hr > 30.0 && weight > 0.0 {
            let kcal_per_min = heart_rate_kcal_per_min(hr, weight, age as f64, inputs.is_male);
            let kcal = (kcal_per_min * duration_min).max(0.0);
            if kcal.is_finite() {
                return CalorieEstimate {
                    kcal,
                    method: CalorieMethod::HeartRate,
                };
            }
        }
    }

    // 3. MET-based, if we at least have weight.
    if let Some(weight) = inputs.weight_kg {
        if weight > 0.0 {
            let met = met_value(inputs.activity_type, inputs.average_speed_mps);
            let grade_factor =
                grade_adjustment_factor(inputs.elevation_gain_meters, inputs.distance_meters);
            let hours = duration_min / 60.0;
            let kcal = (met * grade_factor * weight * hours).max(0.0);
            if kcal.is_finite() {
                return CalorieEstimate {
                    kcal,
                    method: CalorieMethod::Met,
                };
            }
        }
    }

    // 4. Distance + weight-only fallback (~0.75 kcal per kg per km walked,
    // a widely cited rough estimate for walking/running energy cost).
    let weight = inputs.weight_kg.unwrap_or(70.0);
    let km = inputs.distance_meters.max(0.0) / 1000.0;
    let kcal = (0.75 * weight * km).max(0.0);
    CalorieEstimate {
        kcal: if kcal.is_finite() { kcal } else { 0.0 },
        method: CalorieMethod::DistanceWeight,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_inputs() -> CalorieInputs {
        CalorieInputs {
            wearable_kcal: None,
            weight_kg: Some(70.0),
            duration_ms: 30 * 60 * 1000,
            distance_meters: 4000.0,
            average_speed_mps: 2.2,
            average_heart_rate_bpm: None,
            age_years: None,
            is_male: None,
            activity_type: ActivityType::Walk,
            elevation_gain_meters: 0.0,
        }
    }

    #[test]
    fn prefers_wearable_value_when_present() {
        let mut inputs = base_inputs();
        inputs.wearable_kcal = Some(250.0);
        let result = estimate(&inputs);
        assert_eq!(result.method, CalorieMethod::Wearable);
        assert_eq!(result.kcal, 250.0);
    }

    #[test]
    fn falls_back_to_heart_rate_without_wearable() {
        let mut inputs = base_inputs();
        inputs.average_heart_rate_bpm = Some(130.0);
        inputs.age_years = Some(30);
        inputs.is_male = Some(true);
        let result = estimate(&inputs);
        assert_eq!(result.method, CalorieMethod::HeartRate);
        assert!(result.kcal > 0.0 && result.kcal.is_finite());
    }

    #[test]
    fn falls_back_to_met_without_hr_or_wearable() {
        let inputs = base_inputs();
        let result = estimate(&inputs);
        assert_eq!(result.method, CalorieMethod::Met);
        assert!(result.kcal > 0.0);
    }

    #[test]
    fn falls_back_to_distance_weight_without_any_weight_or_hr() {
        let mut inputs = base_inputs();
        inputs.weight_kg = None;
        let result = estimate(&inputs);
        assert_eq!(result.method, CalorieMethod::DistanceWeight);
        assert!(result.kcal >= 0.0 && result.kcal.is_finite());
    }

    #[test]
    fn never_returns_negative_or_nan() {
        let mut inputs = base_inputs();
        inputs.weight_kg = Some(-10.0); // malformed input
        inputs.distance_meters = -50.0;
        let result = estimate(&inputs);
        assert!(result.kcal >= 0.0);
        assert!(result.kcal.is_finite());
    }

    #[test]
    fn running_produces_higher_met_than_walking_at_same_conditions() {
        let mut walk = base_inputs();
        walk.average_speed_mps = 2.5;
        walk.activity_type = ActivityType::Walk;

        let mut run = base_inputs();
        run.average_speed_mps = 2.5;
        run.activity_type = ActivityType::Run;

        let walk_result = estimate(&walk);
        let run_result = estimate(&run);
        assert!(run_result.kcal > walk_result.kcal);
    }

    #[test]
    fn grade_adjustment_increases_estimate_for_climbing() {
        let mut flat = base_inputs();
        flat.elevation_gain_meters = 0.0;

        let mut hilly = base_inputs();
        hilly.elevation_gain_meters = 200.0; // significant climb over 4km

        let flat_result = estimate(&flat);
        let hilly_result = estimate(&hilly);
        assert!(hilly_result.kcal > flat_result.kcal);
    }

    #[test]
    fn zero_duration_gives_zero_calories_for_met_method() {
        let mut inputs = base_inputs();
        inputs.duration_ms = 0;
        let result = estimate(&inputs);
        assert_eq!(result.kcal, 0.0);
    }
}
