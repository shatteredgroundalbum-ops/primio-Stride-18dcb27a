//! GPS point validation and filtering (spec section 3).
//!
//! Raw GPS data is noisy: it can jump across streets, drift while the user
//! stands still, arrive out of order, or come from a mock-location provider.
//! This module is the single gatekeeper deciding whether a raw
//! [`WorkoutPoint`] gets promoted into the accepted route, and if not, why.

use serde::{Deserialize, Serialize};

use crate::models::{haversine_distance_meters, RejectionReason, WorkoutPoint};

/// Tunable thresholds for point validation. Different environments (dense
/// city, open field, tree cover) may want different limits, so this is a
/// plain config struct rather than hard-coded constants.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Reject points whose reported horizontal accuracy is worse
    /// (numerically larger) than this, in meters.
    pub max_accuracy_meters: f64,
    /// Reject a point if implied speed since the previous accepted point
    /// exceeds this, in meters/second (defaults comfortably above elite
    /// running speed to avoid rejecting legitimate fast splits, but below
    /// typical vehicle speed).
    pub max_plausible_speed_mps: f64,
    /// Reject a point if the straight-line jump from the previous accepted
    /// point exceeds this many meters, regardless of elapsed time (guards
    /// against single-sample teleports even when speed math looks sane
    /// due to a large time gap).
    pub max_jump_meters: f64,
    /// Points closer than this to the previous accepted point (in meters)
    /// combined with a tiny elapsed time are treated as duplicates/noise.
    pub duplicate_distance_meters: f64,
    pub duplicate_time_ms: i64,
    /// A cached/stale fix reported with a timestamp older than the previous
    /// accepted point by more than this (ms) is rejected outright.
    pub max_timestamp_regression_ms: i64,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            max_accuracy_meters: 25.0,
            max_plausible_speed_mps: 12.0, // ~43 km/h, above elite sprint pace
            max_jump_meters: 200.0,
            duplicate_distance_meters: 0.5,
            duplicate_time_ms: 1_000,
            max_timestamp_regression_ms: 2_000,
        }
    }
}

impl FilterConfig {
    /// A looser profile for open/rural areas with typically excellent
    /// satellite visibility.
    pub fn open_area() -> Self {
        Self {
            max_accuracy_meters: 15.0,
            ..Default::default()
        }
    }

    /// A tighter-on-accuracy-but-more-jump-tolerant profile for dense urban
    /// canyons / tree cover where accuracy circles are naturally larger but
    /// legitimate short jumps (multipath) are common.
    pub fn dense_urban_or_tree_cover() -> Self {
        Self {
            max_accuracy_meters: 35.0,
            max_jump_meters: 120.0,
            ..Default::default()
        }
    }
}

/// Stateful validator: holds the last *accepted* point so each new sample
/// can be checked for continuity. One instance per active workout.
pub struct GpsFilter {
    config: FilterConfig,
    last_accepted: Option<WorkoutPoint>,
}

impl GpsFilter {
    pub fn new(config: FilterConfig) -> Self {
        Self {
            config,
            last_accepted: None,
        }
    }

    pub fn last_accepted(&self) -> Option<&WorkoutPoint> {
        self.last_accepted.as_ref()
    }

    /// Evaluates a raw point, returning it annotated with `accepted` /
    /// `rejection_reason` set appropriately. If accepted, it becomes the
    /// new `last_accepted` reference point for subsequent calls.
    pub fn evaluate(&mut self, mut point: WorkoutPoint) -> WorkoutPoint {
        if let Some(reason) = self.reject_reason(&point) {
            point.accepted = false;
            point.rejection_reason = Some(reason);
            return point;
        }

        point.accepted = true;
        point.rejection_reason = None;
        self.last_accepted = Some(point.clone());
        point
    }

    fn reject_reason(&self, point: &WorkoutPoint) -> Option<RejectionReason> {
        // Mock-location guard first — cheap and unambiguous.
        if point.is_mock_location {
            return Some(RejectionReason::MockLocation);
        }

        // Missing/invalid timestamp.
        if point.recorded_at <= 0 {
            return Some(RejectionReason::MissingTimestamp);
        }

        // Accuracy guard.
        if let Some(acc) = point.accuracy_meters {
            if acc > self.config.max_accuracy_meters {
                return Some(RejectionReason::PoorAccuracy);
            }
        }

        let Some(prev) = &self.last_accepted else {
            // First point of the workout: nothing to compare against.
            return None;
        };

        // Timestamp ordering: reject stale/out-of-sequence fixes.
        let dt_ms = point.recorded_at - prev.recorded_at;
        if dt_ms < -self.config.max_timestamp_regression_ms {
            return Some(RejectionReason::OutOfSequence);
        }
        if dt_ms <= 0 {
            // Same or regressed timestamp within tolerance: treat as stale
            // duplicate rather than a hard sequence error.
            return Some(RejectionReason::StaleTimestamp);
        }

        let distance = haversine_distance_meters(
            prev.latitude,
            prev.longitude,
            point.latitude,
            point.longitude,
        );

        // Duplicate: negligible movement in a very short time window.
        if distance < self.config.duplicate_distance_meters
            && dt_ms < self.config.duplicate_time_ms
        {
            return Some(RejectionReason::Duplicate);
        }

        // Absolute jump guard, independent of elapsed time.
        if distance > self.config.max_jump_meters {
            return Some(RejectionReason::TooFarFromPrevious);
        }

        // Implied-speed guard: catches jumps that pass the absolute check
        // only because a lot of time elapsed, but are still physically
        // implausible for the elapsed window (e.g. car-speed travel).
        let implied_speed_mps = distance / (dt_ms as f64 / 1000.0);
        if implied_speed_mps > self.config.max_plausible_speed_mps {
            return Some(RejectionReason::ImpossibleSpeed);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LocationSource;

    fn pt(lat: f64, lon: f64, t: i64) -> WorkoutPoint {
        let mut p = WorkoutPoint::new("w1", lat, lon, t, LocationSource::PhoneGps);
        p.accuracy_meters = Some(8.0);
        p
    }

    #[test]
    fn first_point_always_accepted_if_otherwise_valid() {
        let mut f = GpsFilter::new(FilterConfig::default());
        let p = f.evaluate(pt(40.0, -73.0, 1_000));
        assert!(p.accepted);
    }

    #[test]
    fn rejects_poor_accuracy() {
        let mut f = GpsFilter::new(FilterConfig::default());
        let mut p = pt(40.0, -73.0, 1_000);
        p.accuracy_meters = Some(999.0);
        let out = f.evaluate(p);
        assert!(!out.accepted);
        assert_eq!(out.rejection_reason, Some(RejectionReason::PoorAccuracy));
    }

    #[test]
    fn rejects_mock_location() {
        let mut f = GpsFilter::new(FilterConfig::default());
        let mut p = pt(40.0, -73.0, 1_000);
        p.is_mock_location = true;
        let out = f.evaluate(p);
        assert_eq!(out.rejection_reason, Some(RejectionReason::MockLocation));
    }

    #[test]
    fn rejects_duplicate_point_in_short_window() {
        let mut f = GpsFilter::new(FilterConfig::default());
        f.evaluate(pt(40.0, -73.0, 1_000));
        let out = f.evaluate(pt(40.000001, -73.0, 1_200));
        assert_eq!(out.rejection_reason, Some(RejectionReason::Duplicate));
    }

    #[test]
    fn rejects_impossible_jump_over_short_time() {
        let mut f = GpsFilter::new(FilterConfig::default());
        f.evaluate(pt(40.0, -73.0, 1_000));
        // ~1.1km away one second later => ~1100 m/s, impossible.
        let out = f.evaluate(pt(40.01, -73.0, 2_000));
        assert!(!out.accepted);
        assert!(matches!(
            out.rejection_reason,
            Some(RejectionReason::ImpossibleSpeed) | Some(RejectionReason::TooFarFromPrevious)
        ));
    }

    #[test]
    fn rejects_out_of_sequence_timestamp() {
        let mut f = GpsFilter::new(FilterConfig::default());
        f.evaluate(pt(40.0, -73.0, 10_000));
        let out = f.evaluate(pt(40.0001, -73.0, 1_000)); // 9s in the past
        assert_eq!(out.rejection_reason, Some(RejectionReason::OutOfSequence));
    }

    #[test]
    fn accepts_reasonable_walking_sequence() {
        let mut f = GpsFilter::new(FilterConfig::default());
        f.evaluate(pt(40.0, -73.0, 1_000));
        // ~1.4 m/s walking pace over 5 seconds => ~7m
        let out = f.evaluate(pt(40.0000630, -73.0, 6_000));
        assert!(out.accepted, "{:?}", out.rejection_reason);
    }

    #[test]
    fn last_accepted_tracks_only_accepted_points() {
        let mut f = GpsFilter::new(FilterConfig::default());
        f.evaluate(pt(40.0, -73.0, 1_000));
        let mut bad = pt(40.0, -73.0, 1_100);
        bad.accuracy_meters = Some(999.0);
        f.evaluate(bad);
        assert_eq!(f.last_accepted().unwrap().recorded_at, 1_000);
    }
}
