//! Unit system (spec section 34).
//!
//! All internal/canonical values are metric (meters, meters/second,
//! kilograms, seconds-per-km). This module holds the *only* conversions
//! to/from display units (miles, feet, mph, pace-per-mile, pounds) so
//! there is a single source of truth and no risk of inconsistent
//! conversions scattered across the codebase.

pub const METERS_PER_MILE: f64 = 1609.344;
pub const METERS_PER_FOOT: f64 = 0.3048;
pub const KG_PER_POUND: f64 = 0.45359237;

pub fn meters_to_miles(m: f64) -> f64 {
    m / METERS_PER_MILE
}

pub fn miles_to_meters(mi: f64) -> f64 {
    mi * METERS_PER_MILE
}

pub fn meters_to_feet(m: f64) -> f64 {
    m / METERS_PER_FOOT
}

pub fn feet_to_meters(ft: f64) -> f64 {
    ft * METERS_PER_FOOT
}

pub fn kg_to_lb(kg: f64) -> f64 {
    kg / KG_PER_POUND
}

pub fn lb_to_kg(lb: f64) -> f64 {
    lb * KG_PER_POUND
}

pub fn mps_to_kmh(mps: f64) -> f64 {
    mps * 3.6
}

pub fn kmh_to_mps(kmh: f64) -> f64 {
    kmh / 3.6
}

pub fn mps_to_mph(mps: f64) -> f64 {
    mps * 2.23694
}

pub fn mph_to_mps(mph: f64) -> f64 {
    mph / 2.23694
}

pub fn pace_sec_per_km_to_sec_per_mile(sec_per_km: f64) -> f64 {
    if sec_per_km <= 0.0 {
        return 0.0;
    }
    sec_per_km * (METERS_PER_MILE / 1000.0)
}

pub fn pace_sec_per_mile_to_sec_per_km(sec_per_mile: f64) -> f64 {
    if sec_per_mile <= 0.0 {
        return 0.0;
    }
    sec_per_mile / (METERS_PER_MILE / 1000.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceUnit {
    Kilometers,
    Miles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightUnit {
    Kilograms,
    Pounds,
}

/// A single named conversion the FFI boundary can invoke without Dart
/// needing to hand-maintain its own copy of these constants/formulas.
/// Kept as a flat enum (rather than one FFI function per conversion) so
/// adding a new conversion never requires touching `ffi.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionKind {
    MetersToMiles,
    MilesToMeters,
    MetersToFeet,
    FeetToMeters,
    KgToLb,
    LbToKg,
    MpsToKmh,
    KmhToMps,
    MpsToMph,
    MphToMps,
    PaceSecPerKmToSecPerMile,
    PaceSecPerMileToSecPerKm,
}

/// Applies a [`ConversionKind`] to `value`. This is the function the FFI
/// boundary calls so the whole unit system (spec section 34) is reachable
/// from Dart through one stable entry point.
pub fn convert(kind: ConversionKind, value: f64) -> f64 {
    match kind {
        ConversionKind::MetersToMiles => meters_to_miles(value),
        ConversionKind::MilesToMeters => miles_to_meters(value),
        ConversionKind::MetersToFeet => meters_to_feet(value),
        ConversionKind::FeetToMeters => feet_to_meters(value),
        ConversionKind::KgToLb => kg_to_lb(value),
        ConversionKind::LbToKg => lb_to_kg(value),
        ConversionKind::MpsToKmh => mps_to_kmh(value),
        ConversionKind::KmhToMps => kmh_to_mps(value),
        ConversionKind::MpsToMph => mps_to_mph(value),
        ConversionKind::MphToMps => mph_to_mps(value),
        ConversionKind::PaceSecPerKmToSecPerMile => pace_sec_per_km_to_sec_per_mile(value),
        ConversionKind::PaceSecPerMileToSecPerKm => pace_sec_per_mile_to_sec_per_km(value),
    }
}

/// Formats a pace (seconds per canonical km) as "M:SS" for the given
/// display unit, returning `"--:--"` for a zero/invalid pace rather than
/// something misleading like "0:00" or "NaN:NaN".
pub fn format_pace(sec_per_km: f64, unit: DistanceUnit) -> String {
    if sec_per_km <= 0.0 || !sec_per_km.is_finite() {
        return "--:--".to_string();
    }
    let sec_per_unit = match unit {
        DistanceUnit::Kilometers => sec_per_km,
        DistanceUnit::Miles => pace_sec_per_km_to_sec_per_mile(sec_per_km),
    };
    let total_seconds = sec_per_unit.round() as i64;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_roundtrip_meters_miles() {
        let m = 5000.0;
        let mi = meters_to_miles(m);
        let back = miles_to_meters(mi);
        assert!((back - m).abs() < 1e-6);
    }

    #[test]
    fn weight_roundtrip_kg_lb() {
        let kg = 70.0;
        let lb = kg_to_lb(kg);
        assert!((lb - 154.324).abs() < 0.01);
        let back = lb_to_kg(lb);
        assert!((back - kg).abs() < 1e-6);
    }

    #[test]
    fn speed_conversions() {
        assert!((mps_to_kmh(1.0) - 3.6).abs() < 1e-9);
        assert!((kmh_to_mps(3.6) - 1.0).abs() < 1e-9);
        assert!((mps_to_mph(1.0) - 2.23694).abs() < 1e-6);
    }

    #[test]
    fn pace_conversion_km_to_mile() {
        let sec_per_km = 300.0; // 5:00/km
        let sec_per_mile = pace_sec_per_km_to_sec_per_mile(sec_per_km);
        assert!((sec_per_mile - 482.8).abs() < 0.5);
    }

    #[test]
    fn format_pace_invalid_shows_placeholder() {
        assert_eq!(format_pace(0.0, DistanceUnit::Kilometers), "--:--");
        assert_eq!(format_pace(-5.0, DistanceUnit::Miles), "--:--");
    }

    #[test]
    fn convert_dispatches_to_correct_formula() {
        assert!((convert(ConversionKind::MpsToKmh, 1.0) - 3.6).abs() < 1e-9);
        assert!((convert(ConversionKind::KgToLb, 70.0) - kg_to_lb(70.0)).abs() < 1e-9);
    }

    #[test]
    fn format_pace_known_value() {
        // 5:30/km exactly
        assert_eq!(format_pace(330.0, DistanceUnit::Kilometers), "5:30");
    }
}
