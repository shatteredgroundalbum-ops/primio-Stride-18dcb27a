//! Step and cadence engine (spec section 13).
//!
//! Steps may arrive from multiple sources (phone step sensor, wearable,
//! Health Connect). This engine tracks a single authoritative step total
//! using source priority to avoid double counting, and derives cadence
//! (steps per minute) from a rolling window of step deltas.

use std::collections::VecDeque;

use crate::models::{EpochMillis, SensorSource};

#[derive(Debug, Clone, Copy)]
struct CadenceWindowEntry {
    at_ms: EpochMillis,
    steps: u32,
}

pub struct StepEngine {
    total_steps: u32,
    /// The source currently considered authoritative for step counting.
    /// Once a higher-priority source appears, it takes over; we do not
    /// sum across sources simultaneously (spec: "prevent duplicate step
    /// counting").
    active_source: Option<SensorSource>,
    cadence_window: VecDeque<CadenceWindowEntry>,
    cadence_window_span_ms: i64,
    max_cadence_spm: f64,
    cadence_sum_for_average: f64,
    cadence_sample_count: u64,
}

impl StepEngine {
    pub fn new(cadence_window_span_ms: i64) -> Self {
        Self {
            total_steps: 0,
            active_source: None,
            cadence_window: VecDeque::new(),
            cadence_window_span_ms,
            max_cadence_spm: 0.0,
            cadence_sum_for_average: 0.0,
            cadence_sample_count: 0,
        }
    }

    pub fn default_config() -> Self {
        Self::new(20_000) // 20s rolling window for cadence smoothing
    }

    pub fn total_steps(&self) -> u32 {
        self.total_steps
    }

    /// Feeds a step-count delta from a given source at a given time. If a
    /// lower-priority source tries to report after a higher-priority
    /// source is already active, it is ignored to prevent double counting
    /// (per spec: "The engine must identify the source and prevent
    /// duplicate step counting").
    pub fn add_step_delta(&mut self, at_ms: EpochMillis, delta: u32, source: SensorSource) {
        if delta == 0 {
            return;
        }

        match self.active_source {
            Some(active) if source.priority() < active.priority() => {
                // Lower priority than the currently active source: ignore
                // to avoid double-counting the same physical steps.
                return;
            }
            _ => self.active_source = Some(source),
        }

        self.total_steps += delta;

        if self.cadence_window.len() == self.cadence_window.capacity().max(64) {
            self.cadence_window.pop_front();
        }
        self.cadence_window.push_back(CadenceWindowEntry {
            at_ms,
            steps: delta,
        });
        self.trim_cadence_window(at_ms);

        let cadence = self.current_cadence_spm();
        if cadence > self.max_cadence_spm {
            self.max_cadence_spm = cadence;
        }
        self.cadence_sum_for_average += cadence;
        self.cadence_sample_count += 1;
    }

    fn trim_cadence_window(&mut self, now_ms: EpochMillis) {
        while let Some(front) = self.cadence_window.front() {
            if now_ms - front.at_ms > self.cadence_window_span_ms {
                self.cadence_window.pop_front();
            } else {
                break;
            }
        }
    }

    /// Steps per minute, extrapolated from the current rolling window.
    pub fn current_cadence_spm(&self) -> f64 {
        if self.cadence_window.len() < 2 {
            return 0.0;
        }
        let steps_in_window: u32 = self.cadence_window.iter().map(|e| e.steps).sum();
        let raw_span_ms = self.cadence_window.back().unwrap().at_ms - self.cadence_window.front().unwrap().at_ms;
        if raw_span_ms <= 0 {
            return 0.0;
        }
        // `raw_span_ms` only covers the gaps *between* samples (n-1 gaps for
        // n samples), which underestimates the true elapsed time the steps
        // occurred over. Scale up by n/(n-1) so a steady cadence reports
        // its true rate rather than being biased low.
        let n = self.cadence_window.len() as f64;
        let effective_span_ms = raw_span_ms as f64 * n / (n - 1.0);
        (steps_in_window as f64) / (effective_span_ms / 1000.0) * 60.0
    }

    pub fn average_cadence_spm(&self) -> f64 {
        if self.cadence_sample_count == 0 {
            return 0.0;
        }
        self.cadence_sum_for_average / self.cadence_sample_count as f64
    }

    pub fn max_cadence_spm(&self) -> f64 {
        self.max_cadence_spm
    }

    /// Rough stride length estimate (meters) given distance and total
    /// steps, safely handling zero-step cases.
    pub fn estimated_stride_length_m(&self, distance_meters: f64) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        distance_meters / self.total_steps as f64
    }
}

impl Default for StepEngine {
    fn default() -> Self {
        Self::default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_steps_from_single_source() {
        let mut e = StepEngine::default_config();
        e.add_step_delta(0, 10, SensorSource::PhoneStepSensor);
        e.add_step_delta(1000, 10, SensorSource::PhoneStepSensor);
        assert_eq!(e.total_steps(), 20);
    }

    #[test]
    fn higher_priority_source_takes_over() {
        let mut e = StepEngine::default_config();
        e.add_step_delta(0, 10, SensorSource::PhoneStepSensor);
        e.add_step_delta(1000, 15, SensorSource::WearOs); // higher priority, takes over
        assert_eq!(e.total_steps(), 25);
    }

    #[test]
    fn lower_priority_source_ignored_once_higher_active() {
        let mut e = StepEngine::default_config();
        e.add_step_delta(0, 10, SensorSource::WearOs);
        e.add_step_delta(1000, 999, SensorSource::PhoneAccelerometer); // ignored
        assert_eq!(e.total_steps(), 10);
    }

    #[test]
    fn cadence_computed_from_window() {
        let mut e = StepEngine::default_config();
        // 2 steps per second sustained => 120 spm
        for i in 0..10 {
            e.add_step_delta(i * 500, 1, SensorSource::PhoneStepSensor);
        }
        let cadence = e.current_cadence_spm();
        assert!((cadence - 120.0).abs() < 5.0, "cadence={cadence}");
    }

    #[test]
    fn zero_delta_is_noop() {
        let mut e = StepEngine::default_config();
        e.add_step_delta(0, 0, SensorSource::PhoneStepSensor);
        assert_eq!(e.total_steps(), 0);
    }

    #[test]
    fn stride_length_safe_with_zero_steps() {
        let e = StepEngine::default_config();
        assert_eq!(e.estimated_stride_length_m(1000.0), 0.0);
    }
}
