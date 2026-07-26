//! Distance calculation engine (spec section 5).
//!
//! Distance is accumulated only from points that have already passed the
//! [`crate::engine::gps_filter::GpsFilter`] — never from raw points, and
//! never merely from what's drawn on the map. This module also tracks
//! distance-before-pause / distance-after-resume and mile/km splits.

use crate::models::haversine_distance_meters;

#[derive(Debug, Clone, Copy, Default)]
pub struct DistanceTotals {
    pub total_meters: f64,
    /// Distance accumulated while genuinely moving (excludes any distance
    /// that would have been added by drift while paused — since we never
    /// feed points into this engine while paused, this generally equals
    /// `total_meters`, but is tracked separately for clarity/telemetry).
    pub moving_meters: f64,
    pub distance_before_pause: f64,
    pub distance_after_resume: f64,
}

/// Accumulates distance from a stream of accepted (lat, lon) points.
/// One instance per active workout segment.
pub struct DistanceEngine {
    last_point: Option<(f64, f64)>,
    totals: DistanceTotals,
    is_paused: bool,
    /// Marks the total at the moment the most recent pause began, so we
    /// can compute `distance_after_resume` once movement continues.
    total_at_pause_start: Option<f64>,
}

impl DistanceEngine {
    pub fn new() -> Self {
        Self {
            last_point: None,
            totals: DistanceTotals::default(),
            is_paused: false,
            total_at_pause_start: None,
        }
    }

    pub fn totals(&self) -> DistanceTotals {
        self.totals
    }

    pub fn set_paused(&mut self, paused: bool) {
        if paused && !self.is_paused {
            self.total_at_pause_start = Some(self.totals.total_meters);
            self.totals.distance_before_pause = self.totals.total_meters;
        } else if !paused && self.is_paused {
            // Resuming: reset the "last point" anchor so the gap while
            // paused (during which the user may have physically moved a
            // little before pressing resume, e.g. walking to a bench and
            // back) does not get counted as workout distance.
            self.last_point = None;
        }
        self.is_paused = paused;
    }

    /// Feeds one accepted point. Returns the distance delta added (meters),
    /// which is 0.0 while paused.
    pub fn add_point(&mut self, lat: f64, lon: f64) -> f64 {
        if self.is_paused {
            // While paused we intentionally drop the anchor so no distance
            // accrues; still update nothing else.
            return 0.0;
        }

        let delta = match self.last_point {
            Some((plat, plon)) => haversine_distance_meters(plat, plon, lat, lon),
            None => 0.0,
        };

        self.last_point = Some((lat, lon));
        self.totals.total_meters += delta;
        self.totals.moving_meters += delta;

        if let Some(pause_total) = self.total_at_pause_start {
            self.totals.distance_after_resume = self.totals.total_meters - pause_total;
        }

        delta
    }

    pub fn distance_remaining_meters(&self, target_meters: f64) -> f64 {
        (target_meters - self.totals.total_meters).max(0.0)
    }

    /// Seeds the engine with a previously-accumulated total and last known
    /// point, used when restoring a workout from a crash-recovery
    /// checkpoint. This intentionally does *not* replay history — it just
    /// re-anchors so the next accepted point continues accumulating from
    /// the right baseline instead of starting back at zero or producing a
    /// huge spurious jump from `None`.
    pub fn seed(&mut self, last_point: Option<(f64, f64)>, total_meters: f64) {
        self.last_point = last_point;
        self.totals.total_meters = total_meters;
        self.totals.moving_meters = total_meters;
    }
}

impl Default for DistanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks automatic mile/km split boundaries as distance accumulates.
/// Returns how many new whole splits were crossed by the latest addition.
pub struct SplitBoundaryTracker {
    split_distance_meters: f64,
    last_split_index: u32,
}

impl SplitBoundaryTracker {
    pub fn new(split_distance_meters: f64) -> Self {
        Self {
            split_distance_meters,
            last_split_index: 0,
        }
    }

    /// Given the new cumulative total distance, returns the list of split
    /// indices (1-based) newly crossed, in order. Usually returns 0 or 1
    /// items, but could return more if a single GPS jump artificially
    /// crossed multiple splits (which validation should flag separately).
    pub fn check(&mut self, cumulative_meters: f64) -> Vec<u32> {
        if self.split_distance_meters <= 0.0 {
            return Vec::new();
        }
        let current_index = (cumulative_meters / self.split_distance_meters).floor() as u32;
        let mut crossed = Vec::new();
        while self.last_split_index < current_index {
            self.last_split_index += 1;
            crossed.push(self.last_split_index);
        }
        crossed
    }
}

pub const METERS_PER_KILOMETER: f64 = 1000.0;
pub const METERS_PER_MILE: f64 = 1609.344;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_distance_across_points() {
        let mut e = DistanceEngine::new();
        e.add_point(0.0, 0.0);
        let d = e.add_point(0.0, 0.001); // ~111m at equator
        assert!(d > 100.0 && d < 120.0);
        assert!((e.totals().total_meters - d).abs() < 1e-6);
    }

    #[test]
    fn no_distance_added_while_paused() {
        let mut e = DistanceEngine::new();
        e.add_point(0.0, 0.0);
        e.set_paused(true);
        let d = e.add_point(0.0, 0.01); // would be ~1.1km if counted
        assert_eq!(d, 0.0);
        assert!(e.totals().total_meters < 1.0);
    }

    #[test]
    fn resume_does_not_count_gap_as_distance() {
        let mut e = DistanceEngine::new();
        e.add_point(0.0, 0.0);
        e.set_paused(true);
        e.set_paused(false);
        // First point after resume just re-anchors; delta should be 0.
        let d = e.add_point(0.0, 0.01);
        assert_eq!(d, 0.0);
        // Next point after that does accumulate normally.
        let d2 = e.add_point(0.0, 0.011);
        assert!(d2 > 0.0);
    }

    #[test]
    fn split_boundary_tracker_fires_once_per_km() {
        let mut t = SplitBoundaryTracker::new(METERS_PER_KILOMETER);
        assert_eq!(t.check(500.0), Vec::<u32>::new());
        assert_eq!(t.check(999.0), Vec::<u32>::new());
        assert_eq!(t.check(1000.5), vec![1]);
        assert_eq!(t.check(1500.0), Vec::<u32>::new());
        assert_eq!(t.check(2200.0), vec![2]);
    }

    #[test]
    fn distance_remaining_never_negative() {
        let mut e = DistanceEngine::new();
        e.add_point(0.0, 0.0);
        e.add_point(0.0, 0.1); // way past target
        assert_eq!(e.distance_remaining_meters(100.0), 0.0);
    }
}
