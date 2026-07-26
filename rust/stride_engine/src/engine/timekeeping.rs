//! Timekeeping engine (spec section 6).
//!
//! Every duration is derived from system timestamps (epoch milliseconds)
//! supplied by the caller, never from a local incrementing counter or
//! `Instant::now()` inside this struct. This is what makes durations
//! correct across screen-lock, app-background, and process-pause events:
//! the Dart side always passes "now" in, and the engine does pure math.

use crate::models::EpochMillis;

#[derive(Debug, Clone, Copy, Default)]
pub struct TimeTotals {
    /// Time from workout start to now/end, including all pauses.
    pub elapsed_ms: i64,
    /// Elapsed minus manually+automatically paused time.
    pub active_ms: i64,
    /// Subset of active time during which real movement was detected.
    pub moving_ms: i64,
    /// Total time the workout has spent paused (manual + auto).
    pub paused_ms: i64,
}

/// Accumulates elapsed/active/paused/moving time given discrete update
/// ticks. Call [`Self::tick`] whenever the engine processes a new instant
/// (typically once per second, or once per accepted GPS point).
#[derive(Debug, Clone)]
pub struct TimeKeeper {
    started_at: EpochMillis,
    last_tick_at: EpochMillis,
    totals: TimeTotals,
    is_paused: bool,
    /// Wall-clock time of the last confirmed *movement* sample, used to
    /// compute `time_since_last_movement`.
    last_movement_at: Option<EpochMillis>,
    last_point_at: Option<EpochMillis>,
}

impl TimeKeeper {
    pub fn new(started_at: EpochMillis) -> Self {
        Self {
            started_at,
            last_tick_at: started_at,
            totals: TimeTotals::default(),
            is_paused: false,
            last_movement_at: None,
            last_point_at: None,
        }
    }

    pub fn totals(&self) -> TimeTotals {
        self.totals
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Advances the clock to `now`, crediting the elapsed delta to the
    /// correct bucket(s) based on current pause/movement state.
    ///
    /// `is_moving` should reflect the movement-detection result for the
    /// interval just elapsed (see `movement.rs`).
    pub fn tick(&mut self, now: EpochMillis, is_moving: bool) {
        if now <= self.last_tick_at {
            // Clock went backwards or didn't advance (e.g. duplicate tick);
            // ignore rather than corrupt totals.
            return;
        }
        let delta = now - self.last_tick_at;
        self.totals.elapsed_ms += delta;

        if self.is_paused {
            self.totals.paused_ms += delta;
        } else {
            self.totals.active_ms += delta;
            if is_moving {
                self.totals.moving_ms += delta;
                self.last_movement_at = Some(now);
            }
        }

        self.last_tick_at = now;
    }

    pub fn record_point_received(&mut self, at: EpochMillis) {
        self.last_point_at = Some(at);
    }

    pub fn time_since_last_point_ms(&self, now: EpochMillis) -> Option<i64> {
        self.last_point_at.map(|t| now - t)
    }

    pub fn time_since_last_movement_ms(&self, now: EpochMillis) -> Option<i64> {
        self.last_movement_at.map(|t| now - t)
    }

    pub fn elapsed_since_start_ms(&self, now: EpochMillis) -> i64 {
        now - self.started_at
    }

    /// Restores accumulated totals from a checkpoint after a crash/restart
    /// recovery. `checkpoint_at` becomes the new tick anchor so time that
    /// passed while the app was dead is not silently counted as active —
    /// the caller (recovery flow) decides separately whether to fold any
    /// of that gap into `paused_ms`.
    pub fn seed(&mut self, totals: TimeTotals, checkpoint_at: EpochMillis) {
        self.totals = totals;
        self.last_tick_at = checkpoint_at;
    }
}

/// Computes remaining time toward a duration-based goal, safely handling
/// already-elapsed-past-target cases.
pub fn countdown_remaining_ms(target_duration_ms: i64, active_ms: i64) -> i64 {
    (target_duration_ms - active_ms).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_active_time_when_not_paused() {
        let mut tk = TimeKeeper::new(0);
        tk.tick(1_000, true);
        tk.tick(2_000, true);
        let t = tk.totals();
        assert_eq!(t.elapsed_ms, 2_000);
        assert_eq!(t.active_ms, 2_000);
        assert_eq!(t.moving_ms, 2_000);
        assert_eq!(t.paused_ms, 0);
    }

    #[test]
    fn pauses_stop_active_and_moving_accumulation() {
        let mut tk = TimeKeeper::new(0);
        tk.tick(1_000, true);
        tk.set_paused(true);
        tk.tick(3_000, true); // moving flag ignored while paused
        let t = tk.totals();
        assert_eq!(t.elapsed_ms, 3_000);
        assert_eq!(t.active_ms, 1_000);
        assert_eq!(t.paused_ms, 2_000);
        assert_eq!(t.moving_ms, 1_000);
    }

    #[test]
    fn moving_time_excludes_stationary_active_periods() {
        let mut tk = TimeKeeper::new(0);
        tk.tick(1_000, false); // active but stationary (e.g. at a crossing)
        tk.tick(2_000, true);
        let t = tk.totals();
        assert_eq!(t.active_ms, 2_000);
        assert_eq!(t.moving_ms, 1_000);
    }

    #[test]
    fn ignores_non_advancing_ticks() {
        let mut tk = TimeKeeper::new(0);
        tk.tick(1_000, true);
        tk.tick(1_000, true); // duplicate timestamp, should be a no-op
        tk.tick(500, true); // clock went backwards, should be a no-op
        assert_eq!(tk.totals().elapsed_ms, 1_000);
    }

    #[test]
    fn countdown_never_goes_negative() {
        assert_eq!(countdown_remaining_ms(60_000, 90_000), 0);
        assert_eq!(countdown_remaining_ms(60_000, 30_000), 30_000);
    }
}
