//! Auto-pause / auto-resume (spec section 11).
//!
//! Sits on top of [`crate::engine::movement::MovementDetector`]. Requires
//! *sustained* stillness/movement before flipping state, and enforces a
//! cooldown so a single noisy sample can't cause pause/resume flicker.

use serde::{Deserialize, Serialize};

use crate::models::EpochMillis;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPauseState {
    Running,
    AutoPaused,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AutoPauseConfig {
    /// How long the user must be stationary before auto-pause triggers.
    pub stop_duration_threshold_ms: i64,
    /// How long sustained movement must be observed before auto-resuming.
    pub resume_duration_threshold_ms: i64,
    /// Minimum time that must pass after any state flip before another
    /// flip is allowed, to prevent flicker.
    pub cooldown_ms: i64,
    pub enabled: bool,
}

impl Default for AutoPauseConfig {
    fn default() -> Self {
        Self {
            stop_duration_threshold_ms: 8_000,
            resume_duration_threshold_ms: 3_000,
            cooldown_ms: 5_000,
            enabled: true,
        }
    }
}

pub struct AutoPauseEngine {
    config: AutoPauseConfig,
    state: AutoPauseState,
    /// When the current stationary/moving streak began.
    streak_started_at: Option<EpochMillis>,
    streak_is_stationary: bool,
    last_flip_at: Option<EpochMillis>,
}

/// Result of feeding one movement observation into the auto-pause engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoPauseTransition {
    None,
    TriggeredAutoPause,
    TriggeredAutoResume,
}

impl AutoPauseEngine {
    pub fn new(config: AutoPauseConfig) -> Self {
        Self {
            config,
            state: AutoPauseState::Running,
            streak_started_at: None,
            streak_is_stationary: false,
            last_flip_at: None,
        }
    }

    pub fn state(&self) -> AutoPauseState {
        self.state
    }

    /// `is_stationary` should come from the movement detector's current
    /// signal (treat `Indeterminate` as "not stationary" to be safe / avoid
    /// spurious pausing before enough data has accumulated).
    pub fn observe(&mut self, now: EpochMillis, is_stationary: bool) -> AutoPauseTransition {
        if !self.config.enabled {
            return AutoPauseTransition::None;
        }

        match self.streak_started_at {
            Some(_) if self.streak_is_stationary == is_stationary => {
                // Streak continues; fall through to threshold checks below.
            }
            _ => {
                // Streak just started or changed direction.
                self.streak_started_at = Some(now);
                self.streak_is_stationary = is_stationary;
            }
        }

        let streak_duration = now - self.streak_started_at.unwrap_or(now);
        let in_cooldown = self
            .last_flip_at
            .map(|t| now - t < self.config.cooldown_ms)
            .unwrap_or(false);

        if in_cooldown {
            return AutoPauseTransition::None;
        }

        match self.state {
            AutoPauseState::Running
                if is_stationary && streak_duration >= self.config.stop_duration_threshold_ms =>
            {
                self.state = AutoPauseState::AutoPaused;
                self.last_flip_at = Some(now);
                AutoPauseTransition::TriggeredAutoPause
            }
            AutoPauseState::AutoPaused
                if !is_stationary
                    && streak_duration >= self.config.resume_duration_threshold_ms =>
            {
                self.state = AutoPauseState::Running;
                self.last_flip_at = Some(now);
                AutoPauseTransition::TriggeredAutoResume
            }
            _ => AutoPauseTransition::None,
        }
    }

    /// Allows the user to manually override (e.g. dismiss an auto-pause and
    /// force the workout back to running) without waiting for the movement
    /// streak logic.
    pub fn force_state(&mut self, now: EpochMillis, state: AutoPauseState) {
        self.state = state;
        self.last_flip_at = Some(now);
        self.streak_started_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> AutoPauseConfig {
        AutoPauseConfig {
            stop_duration_threshold_ms: 5_000,
            resume_duration_threshold_ms: 2_000,
            cooldown_ms: 1_000,
            enabled: true,
        }
    }

    #[test]
    fn does_not_pause_on_single_brief_stop() {
        let mut e = AutoPauseEngine::new(cfg());
        let t = e.observe(0, true);
        assert_eq!(t, AutoPauseTransition::None);
        assert_eq!(e.state(), AutoPauseState::Running);
    }

    #[test]
    fn pauses_after_sustained_stop() {
        let mut e = AutoPauseEngine::new(cfg());
        e.observe(0, true);
        e.observe(2_000, true);
        let t = e.observe(5_500, true); // streak of 5.5s >= 5s threshold
        assert_eq!(t, AutoPauseTransition::TriggeredAutoPause);
        assert_eq!(e.state(), AutoPauseState::AutoPaused);
    }

    #[test]
    fn resumes_after_sustained_movement() {
        let mut e = AutoPauseEngine::new(cfg());
        e.observe(0, true);
        e.observe(6_000, true); // triggers pause
        assert_eq!(e.state(), AutoPauseState::AutoPaused);

        // Movement streak begins after cooldown passes.
        e.observe(8_000, false);
        let t = e.observe(10_500, false); // 2.5s of movement >= 2s threshold, cooldown elapsed
        assert_eq!(t, AutoPauseTransition::TriggeredAutoResume);
    }

    #[test]
    fn cooldown_prevents_immediate_flicker() {
        let mut e = AutoPauseEngine::new(cfg());
        e.observe(0, true);
        e.observe(6_000, true);
        assert_eq!(e.state(), AutoPauseState::AutoPaused);
        // Movement observed immediately within cooldown window should not flip yet.
        let t = e.observe(6_500, false);
        assert_eq!(t, AutoPauseTransition::None);
        assert_eq!(e.state(), AutoPauseState::AutoPaused);
    }

    #[test]
    fn disabled_config_never_triggers() {
        let mut cfg = cfg();
        cfg.enabled = false;
        let mut e = AutoPauseEngine::new(cfg);
        e.observe(0, true);
        let t = e.observe(10_000, true);
        assert_eq!(t, AutoPauseTransition::None);
        assert_eq!(e.state(), AutoPauseState::Running);
    }

    #[test]
    fn manual_force_overrides_state_immediately() {
        let mut e = AutoPauseEngine::new(cfg());
        e.force_state(0, AutoPauseState::Running);
        assert_eq!(e.state(), AutoPauseState::Running);
    }
}
