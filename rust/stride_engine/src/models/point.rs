use serde::{Deserialize, Serialize};

use super::EpochMillis;

/// Where a location fix came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationSource {
    PhoneGps,
    WearOs,
    HealthConnect,
    Manual,
    ServerCorrected,
    Estimated,
}

/// Why a raw GPS reading was rejected by the filtering pipeline.
/// `None` (represented as absence in JSON) means the point was accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionReason {
    PoorAccuracy,
    ImpossibleJump,
    Duplicate,
    StaleTimestamp,
    MissingTimestamp,
    ImpossibleSpeed,
    OutOfSequence,
    MockLocation,
    TooFarFromPrevious,
}

/// A single incoming raw location/sensor reading, before or after
/// validation. Mirrors the `WorkoutPoint` model from the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutPoint {
    pub point_id: String,
    pub workout_id: String,
    pub latitude: f64,
    pub longitude: f64,
    /// Meters above sea level, if reported.
    pub altitude_meters: Option<f64>,
    /// Horizontal accuracy radius in meters (smaller = better).
    pub accuracy_meters: Option<f64>,
    /// Vertical/altitude accuracy in meters, if reported.
    pub altitude_accuracy_meters: Option<f64>,
    /// Device-reported instantaneous speed, m/s.
    pub speed_meters_per_second: Option<f64>,
    /// Compass bearing in degrees [0, 360), if reported.
    pub bearing_degrees: Option<f64>,
    pub recorded_at: EpochMillis,
    pub source: LocationSource,
    /// Whether the OS flagged this as a mock/simulated location
    /// (Android `Location.isMock()` / iOS equivalent), when available.
    pub is_mock_location: bool,
    /// Filled in by the validator; `true` once accepted into the route.
    pub accepted: bool,
    pub rejection_reason: Option<RejectionReason>,
}

impl WorkoutPoint {
    pub fn new(
        workout_id: impl Into<String>,
        latitude: f64,
        longitude: f64,
        recorded_at: EpochMillis,
        source: LocationSource,
    ) -> Self {
        Self {
            point_id: super::new_id(),
            workout_id: workout_id.into(),
            latitude,
            longitude,
            altitude_meters: None,
            accuracy_meters: None,
            altitude_accuracy_meters: None,
            speed_meters_per_second: None,
            bearing_degrees: None,
            recorded_at,
            source,
            is_mock_location: false,
            accepted: false,
            rejection_reason: None,
        }
    }
}

/// Haversine great-circle distance between two coordinates, in meters.
pub fn haversine_distance_meters(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let (lat1_r, lat2_r) = (lat1.to_radians(), lat2.to_radians());
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + lat1_r.cos() * lat2_r.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_M * c
}

/// Initial compass bearing (degrees, 0-360) from point 1 to point 2.
pub fn bearing_degrees(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (lat1_r, lat2_r) = (lat1.to_radians(), lat2.to_radians());
    let d_lon = (lon2 - lon1).to_radians();
    let y = d_lon.sin() * lat2_r.cos();
    let x = lat1_r.cos() * lat2_r.sin() - lat1_r.sin() * lat2_r.cos() * d_lon.cos();
    let deg = y.atan2(x).to_degrees();
    (deg + 360.0) % 360.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_zero_distance_for_identical_points() {
        let d = haversine_distance_meters(40.0, -73.0, 40.0, -73.0);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn haversine_known_distance_one_degree_latitude() {
        // Roughly 111.32 km per degree of latitude.
        let d = haversine_distance_meters(0.0, 0.0, 1.0, 0.0);
        assert!((d - 111_195.0).abs() < 500.0, "got {d}");
    }

    #[test]
    fn bearing_due_north_is_zero() {
        let b = bearing_degrees(0.0, 0.0, 1.0, 0.0);
        assert!(b.abs() < 1e-6 || (b - 360.0).abs() < 1e-6);
    }

    #[test]
    fn bearing_due_east_is_ninety() {
        let b = bearing_degrees(0.0, 0.0, 0.0, 1.0);
        assert!((b - 90.0).abs() < 1e-6);
    }
}
