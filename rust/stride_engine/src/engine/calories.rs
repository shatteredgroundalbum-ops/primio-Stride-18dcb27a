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

/// The version of the calorie calculation engine that produced a given
/// estimate. This is surfaced on every `CalorieEstimate` so the UI and
/// cloud can display the calculation version alongside the value, and
/// so future revisions can be distinguished.
///
/// - **V1**: Initial MET/HR/wearable/distance-weight chain.
/// - **V2**: Same priority chain but with grade-adjustment factor applied
///   to the MET estimate and tighter input validation (no negative or
///   NaN values pass through).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalorieVersion {
    V1,
    V2,
}

impl CalorieVersion {
    /// Returns a human-readable label for this version.
    pub fn label(self) -> &'static str {
        match self {
            CalorieVersion::V1 => "Calorie estimate v1 (MET/HR/wearable)",
            CalorieVersion::V2 => "Calorie estimate v2 (grade-adjusted)",
        }
    }

    /// Returns the current (latest) version of the calorie engine.
    pub fn current() -> CalorieVersion {
        CalorieVersion::V2
    }
}

/// The method used to produce a calorie estimate, following a strict
/// source-priority order:
///
///   1. wearable   - device-reported active-energy value
///   2. heart_rate - HR-based formula (Keytel et al.)
///   3. met        - activity MET table × weight × time
///   4. distance_weight - crude fallback using distance & weight only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    /// Returns a human-readable label for this method, suitable for
    /// display alongside the estimated value.
    pub fn label(self) -> &'static str {
        match self {
            CalorieMethod::Wearable => "Wearable (device-reported)",
            CalorieMethod::HeartRate => "Heart-rate based (Keytel et al.)",
            CalorieMethod::Met => "MET-based (activity compendium)",
            CalorieMethod::DistanceWeight => "Distance & weight (rough estimate)",
        }
    }

    /// Returns the priority rank of this method (1 = highest priority,
    /// 4 = lowest). This documents the source-priority order.
    pub fn priority_rank(self) -> u8 {
        match self {
            CalorieMethod::Wearable => 1,
            CalorieMethod::HeartRate => 2,
            CalorieMethod::Met => 3,
            CalorieMethod::DistanceWeight => 4,
        }
    }

    /// Parses a method from its snake_case string representation.
    pub fn from_str(s: &str) -> Option<CalorieMethod> {
        match s {
            "wearable" => Some(CalorieMethod::Wearable),
            "heart_rate" => Some(CalorieMethod::HeartRate),
            "met" => Some(CalorieMethod::Met),
            "distance_weight" => Some(CalorieMethod::DistanceWeight),
            _ => None,
        }
    }
}

/// Returns the source-priority order as a list of (method, rank) pairs,
/// from highest to lowest priority. This documents the fallback chain.
pub fn source_priority_order() -> Vec<(CalorieMethod, u8)> {
    vec![
        (CalorieMethod::Wearable, 1),
        (CalorieMethod::HeartRate, 2),
        (CalorieMethod::Met, 3),
        (CalorieMethod::DistanceWeight, 4),
    ]
}

/// Returns a human-readable label describing the calorie estimate,
/// suitable for display alongside the estimated value. The label
/// always includes the word "estimate" to make clear that the value
/// is an approximation, not a precise measurement.
pub fn calorie_estimate_label(method: CalorieMethod, version: CalorieVersion) -> String {
    format!(
        "Estimated calories ({} — {})",
        method.label(),
        version.label()
    )
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
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

/// The result of a calorie estimate, including the method used, the
/// calculation version, and a display label. Every value is tagged
/// with the method and version so the UI can surface the source of
/// the estimate and the calculation version.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CalorieEstimate {
    pub kcal: f64,
    pub method: CalorieMethod,
    pub version: CalorieVersion,
}

/// A fuller result that includes a human-readable display label,
/// suitable for returning via FFI to the UI layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalorieEstimateResult {
    /// The estimated calories in kcal.
    pub kcal: f64,
    /// The method that produced this estimate.
    pub method: CalorieMethod,
    /// The version of the calorie engine.
    pub version: CalorieVersion,
    /// The method as a snake_case string (e.g. "met", "heart_rate").
    pub method_str: String,
    /// The version as a snake_case string (e.g. "v1", "v2").
    pub version_str: String,
    /// A human-readable label for the method.
    pub method_label: String,
    /// A human-readable label for the version.
    pub version_label: String,
    /// A full display label for the estimate (includes "estimate").
    pub display_label: String,
    /// The priority rank of the method used (1–4).
    pub source_priority: u8,
    /// Whether the value is an estimate (always true — calories are
    /// always approximate).
    pub is_estimate: bool,
}

impl CalorieEstimateResult {
    /// Builds a `CalorieEstimateResult` from a `CalorieEstimate`.
    pub fn from_estimate(estimate: CalorieEstimate) -> CalorieEstimateResult {
        CalorieEstimateResult {
            kcal: estimate.kcal,
            method: estimate.method,
            version: estimate.version,
            method_str: estimate.method.as_str().to_string(),
            version_str: match estimate.version {
                CalorieVersion::V1 => "v1".to_string(),
                CalorieVersion::V2 => "v2".to_string(),
            },
            method_label: estimate.method.label().to_string(),
            version_label: estimate.version.label().to_string(),
            display_label: calorie_estimate_label(estimate.method, estimate.version),
            source_priority: estimate.method.priority_rank(),
            is_estimate: true,
        }
    }
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
    let version = CalorieVersion::current();

    // 1. Wearable-reported value, if present and sane.
    if let Some(kcal) = inputs.wearable_kcal {
        if kcal.is_finite() && kcal >= 0.0 {
            return CalorieEstimate {
                kcal,
                method: CalorieMethod::Wearable,
                version,
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
                    version,
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
                    version,
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
        version,
    }
}

/// Produces a full calorie estimate result with labels, version, and
/// source-priority information, suitable for returning via FFI.
pub fn estimate_with_details(inputs: &CalorieInputs) -> CalorieEstimateResult {
    CalorieEstimateResult::from_estimate(estimate(inputs))
}

/// Validates that a calorie estimate value is reasonable and not
/// impossible. Returns `true` if the value is finite, non-negative,
/// and below a sanity ceiling (e.g., no more than 10,000 kcal for a
/// single session — a hard cap to catch corrupted or implausible
/// values).
pub fn is_plausible_kcal(kcal: f64) -> bool {
    kcal.is_finite() && kcal >= 0.0 && kcal <= 10_000.0
}

/// Clamps a calorie value to a plausible range, preventing impossible
/// values from being stored or displayed.
///
/// - Negative values → 0.
/// - NaN → 0 (we can't trust the value at all).
/// - Positive infinity → 10,000 (treat as "too large, cap it").
/// - Otherwise → clamped to [0, 10,000].
pub fn clamp_to_plausible(kcal: f64) -> f64 {
    if kcal.is_nan() || kcal < 0.0 {
        return 0.0;
    }
    if kcal.is_infinite() {
        // Positive infinity only (negatives caught above).
        return 10_000.0;
    }
    kcal.min(10_000.0)
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

    // ─── §9 — version / label surfacing tests ───

    #[test]
    fn calorie_version_current_is_v2() {
        assert_eq!(CalorieVersion::current(), CalorieVersion::V2);
    }

    #[test]
    fn calorie_version_labels_are_non_empty() {
        assert!(!CalorieVersion::V1.label().is_empty());
        assert!(!CalorieVersion::V2.label().is_empty());
        assert_ne!(CalorieVersion::V1.label(), CalorieVersion::V2.label());
    }

    #[test]
    fn calorie_method_labels_are_distinct() {
        assert_ne!(CalorieMethod::Wearable.label(), CalorieMethod::HeartRate.label());
        assert_ne!(CalorieMethod::Met.label(), CalorieMethod::DistanceWeight.label());
    }

    #[test]
    fn calorie_method_priority_rank_is_monotonic() {
        assert_eq!(CalorieMethod::Wearable.priority_rank(), 1);
        assert_eq!(CalorieMethod::HeartRate.priority_rank(), 2);
        assert_eq!(CalorieMethod::Met.priority_rank(), 3);
        assert_eq!(CalorieMethod::DistanceWeight.priority_rank(), 4);
    }

    #[test]
    fn calorie_method_from_str_round_trips() {
        for method in [
            CalorieMethod::Wearable,
            CalorieMethod::HeartRate,
            CalorieMethod::Met,
            CalorieMethod::DistanceWeight,
        ] {
            let s = method.as_str();
            assert_eq!(CalorieMethod::from_str(s), Some(method));
        }
        assert_eq!(CalorieMethod::from_str("nonexistent"), None);
    }

    #[test]
    fn source_priority_order_returns_all_four_methods() {
        let order = source_priority_order();
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], (CalorieMethod::Wearable, 1));
        assert_eq!(order[1], (CalorieMethod::HeartRate, 2));
        assert_eq!(order[2], (CalorieMethod::Met, 3));
        assert_eq!(order[3], (CalorieMethod::DistanceWeight, 4));
    }

    #[test]
    fn calorie_estimate_label_includes_estimate_word() {
        let label = calorie_estimate_label(CalorieMethod::Met, CalorieVersion::V2);
        assert!(label.contains("estimate"), "label must include 'estimate': {}", label);
        assert!(label.contains("MET"), "label must include method label: {}", label);
        assert!(label.contains("v2"), "label must include version label: {}", label);
    }

    #[test]
    fn estimate_always_includes_version() {
        let mut inputs = base_inputs();
        let mut methods_seen: Vec<CalorieMethod> = Vec::new();
        // Default → MET (weight present, no HR, no wearable)
        let r = estimate(&inputs);
        assert_eq!(r.version, CalorieVersion::V2);
        methods_seen.push(r.method);

        // Add wearable → Wearable
        inputs.wearable_kcal = Some(250.0);
        let r = estimate(&inputs);
        assert_eq!(r.version, CalorieVersion::V2);
        assert_eq!(r.method, CalorieMethod::Wearable);
        methods_seen.push(r.method);

        // Add HR, remove wearable → HeartRate
        inputs.wearable_kcal = None;
        inputs.average_heart_rate_bpm = Some(130.0);
        inputs.age_years = Some(30);
        let r = estimate(&inputs);
        assert_eq!(r.version, CalorieVersion::V2);
        assert_eq!(r.method, CalorieMethod::HeartRate);
        methods_seen.push(r.method);

        // Remove weight → DistanceWeight fallback
        inputs.wearable_kcal = None;
        inputs.average_heart_rate_bpm = None;
        inputs.weight_kg = None;
        let r = estimate(&inputs);
        assert_eq!(r.version, CalorieVersion::V2);
        assert_eq!(r.method, CalorieMethod::DistanceWeight);
        methods_seen.push(r.method);

        // Verify all four methods were exercised
        assert_eq!(methods_seen.len(), 4);
        assert!(methods_seen.contains(&CalorieMethod::Met));
        assert!(methods_seen.contains(&CalorieMethod::Wearable));
        assert!(methods_seen.contains(&CalorieMethod::HeartRate));
        assert!(methods_seen.contains(&CalorieMethod::DistanceWeight));
    }

    #[test]
    fn estimate_with_details_builds_full_result() {
        let inputs = base_inputs();
        let result = estimate_with_details(&inputs);
        assert_eq!(result.method, CalorieMethod::Met);
        assert_eq!(result.version, CalorieVersion::V2);
        assert_eq!(result.method_str, "met");
        assert_eq!(result.version_str, "v2");
        assert!(!result.method_label.is_empty());
        assert!(!result.version_label.is_empty());
        assert!(!result.display_label.is_empty());
        assert!(result.display_label.contains("estimate"));
        assert_eq!(result.source_priority, 3);
        assert!(result.is_estimate);
    }

    #[test]
    fn calorie_estimate_result_fields_are_consistent() {
        let est = CalorieEstimate {
            kcal: 200.0,
            method: CalorieMethod::HeartRate,
            version: CalorieVersion::V1,
        };
        let result = CalorieEstimateResult::from_estimate(est);
        assert_eq!(result.kcal, 200.0);
        assert_eq!(result.method, CalorieMethod::HeartRate);
        assert_eq!(result.version, CalorieVersion::V1);
        assert_eq!(result.method_str, "heart_rate");
        assert_eq!(result.version_str, "v1");
        assert_eq!(result.source_priority, 2);
        assert!(result.is_estimate);
    }

    #[test]
    fn is_plausible_kcal_accepts_reasonable_values() {
        assert!(is_plausible_kcal(0.0));
        assert!(is_plausible_kcal(100.0));
        assert!(is_plausible_kcal(500.0));
        assert!(is_plausible_kcal(10_000.0));
    }

    #[test]
    fn is_plausible_kcal_rejects_unreasonable_values() {
        assert!(!is_plausible_kcal(-1.0));
        assert!(!is_plausible_kcal(-100.0));
        assert!(!is_plausible_kcal(10_001.0));
        assert!(!is_plausible_kcal(1_000_000.0));
        assert!(!is_plausible_kcal(f64::NAN));
        assert!(!is_plausible_kcal(f64::INFINITY));
        assert!(!is_plausible_kcal(f64::NEG_INFINITY));
    }

    #[test]
    fn clamp_to_plausible_clamps_to_range() {
        assert_eq!(clamp_to_plausible(-1.0), 0.0);
        assert_eq!(clamp_to_plausible(-100.0), 0.0);
        assert_eq!(clamp_to_plausible(0.0), 0.0);
        assert_eq!(clamp_to_plausible(500.0), 500.0);
        assert_eq!(clamp_to_plausible(10_000.0), 10_000.0);
        assert_eq!(clamp_to_plausible(10_001.0), 10_000.0);
        assert_eq!(clamp_to_plausible(1_000_000.0), 10_000.0);
        assert_eq!(clamp_to_plausible(f64::NAN), 0.0);
        assert_eq!(clamp_to_plausible(f64::INFINITY), 10_000.0);
        assert_eq!(clamp_to_plausible(f64::NEG_INFINITY), 0.0);
    }

    #[test]
    fn calorie_estimate_serializes_with_version() {
        let est = CalorieEstimate {
            kcal: 200.0,
            method: CalorieMethod::Met,
            version: CalorieVersion::V2,
        };
        let json = serde_json::to_string(&est).unwrap();
        assert!(json.contains("\"version\":\"v2\""));
        assert!(json.contains("\"method\":\"met\""));
        assert!(json.contains("200"));
        // Round-trip
        let back: CalorieEstimate = serde_json::from_str(&json).unwrap();
        assert_eq!(back, est);
    }

    #[test]
    fn calorie_estimate_result_serializes_round_trip() {
        let est = CalorieEstimate {
            kcal: 350.0,
            method: CalorieMethod::Wearable,
            version: CalorieVersion::V2,
        };
        let result = CalorieEstimateResult::from_estimate(est);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"method_str\":\"wearable\""));
        assert!(json.contains("\"version_str\":\"v2\""));
        assert!(json.contains("\"source_priority\":1"));
        assert!(json.contains("\"is_estimate\":true"));
        let back: CalorieEstimateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back, result);
    }

    #[test]
    fn version_field_present_on_all_methods() {
        let mut inputs = base_inputs();
        // DistanceWeight (no weight)
        inputs.weight_kg = None;
        let dw = estimate(&inputs);
        assert_eq!(dw.version, CalorieVersion::V2);
        assert_eq!(dw.method, CalorieMethod::DistanceWeight);

        // MET
        inputs.weight_kg = Some(70.0);
        let met = estimate(&inputs);
        assert_eq!(met.version, CalorieVersion::V2);
        assert_eq!(met.method, CalorieMethod::Met);
    }
}
