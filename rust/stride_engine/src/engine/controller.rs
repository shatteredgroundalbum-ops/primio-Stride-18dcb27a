//! Workout Session Controller (spec section 1).
//!
//! This is the top-level orchestrator that owns one workout's lifecycle
//! end-to-end, wiring together every other engine submodule according to
//! the processing flow described in the spec:
//!
//! ```text
//! User taps Start
//!         v
//! Validate permissions and GPS
//!         v
//! Create local workout session
//!         v
//! Start foreground tracking service      (Dart/native side)
//!         v
//! Receive GPS and sensor data             --> add_location_sample() / add_step_delta() / add_heart_rate_sample()
//!         v
//! Validate and filter each reading        --> GpsFilter
//!         v
//! Detect walking, running, stopping, or vehicle movement --> ActivityClassifier, MovementDetector, VehicleMotionDetector
//!         v
//! Update distance, time, pace, speed, route, steps, calories --> DistanceEngine, TimeKeeper, SpeedEngine, PaceEngine, RouteBuilder, StepEngine, calories::estimate
//!         v
//! Save checkpoints locally                --> recovery::build_checkpoint (Dart persists it)
//!         v
//! Draw route and update live workout screen --> tick() return value
//!         v
//! Generate splits, coaching events, and goal alerts --> SplitEngine, CoachingEventBuilder, WorkoutGoal
//!         v
//! User ends workout
//!         v
//! Validate and finalize workout            --> validation::validate
//!         v
//! Generate workout summary                 --> finish()
//!         v
//! Save locally as sync pending             (Dart persists it)
//!         v
//! Upload summary and route when online     (Dart/SyncService)
//! ```
//!
//! One `WorkoutSessionController` instance exists per in-progress workout.
//! It never performs I/O itself (no file/network access) — every method
//! is pure state mutation driven by inputs the Dart side supplies
//! (timestamps, sensor readings). This makes the whole engine trivially
//! unit-testable and free of platform dependencies.

use crate::engine::autopause::{AutoPauseConfig, AutoPauseEngine, AutoPauseTransition};
use crate::engine::calories::{self, CalorieInputs};
use crate::engine::classification::{
    ActivityClassifier, ClassificationSample, ClassificationThresholds, VehicleMotionDetector,
};
use crate::engine::coaching::CoachingEventBuilder;
use crate::engine::distance::{DistanceEngine, METERS_PER_KILOMETER, METERS_PER_MILE};
use crate::engine::elevation::ElevationEngine;
use crate::engine::gps_filter::{FilterConfig, GpsFilter};
use crate::engine::gps_quality::{GpsQualityMonitor, GpsQualitySample};
use crate::engine::heart_rate::HeartRateEngine;
use crate::engine::movement::{MovementDetector, MovementSample, MovementSignal};
use crate::engine::pace::{self, FastestPaceTracker};
use crate::engine::personal_records::PriorBests;
use crate::engine::route::RouteBuilder;
use crate::engine::speed::SpeedEngine;
use crate::engine::splits::SplitEngine;
use crate::engine::steps::StepEngine;
use crate::engine::timekeeping::TimeKeeper;
use crate::engine::validation::{self, ValidationInput};
use crate::models::{
    ActivityType, CoachingEvent, EpochMillis, GoalProgress, GpsQualityState,
    RejectionReason, SensorSource, WorkoutCheckpoint, WorkoutGoal, WorkoutPoint, WorkoutSession,
    WorkoutSplit, WorkoutState, WorkoutSummary,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionConfig {
    #[serde(default)]
    pub filter_config: FilterConfig,
    #[serde(default)]
    pub autopause_config: AutoPauseConfig,
    #[serde(default)]
    pub classification_thresholds: ClassificationThresholds,
    #[serde(default = "default_split_distance")]
    pub split_distance_meters: f64,
    #[serde(default = "default_max_hr")]
    pub max_hr_estimate: u16,
}

fn default_split_distance() -> f64 {
    METERS_PER_KILOMETER
}
fn default_max_hr() -> u16 {
    190
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            filter_config: FilterConfig::default(),
            autopause_config: AutoPauseConfig::default(),
            classification_thresholds: ClassificationThresholds::default(),
            split_distance_meters: METERS_PER_KILOMETER,
            max_hr_estimate: 190,
        }
    }
}

/// Everything the Dart layer needs to redraw the live workout screen after
/// processing a tick or a new sample.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LiveUpdate {
    pub session: WorkoutSession,
    pub newly_completed_splits: Vec<WorkoutSplit>,
    pub coaching_events: Vec<CoachingEvent>,
    pub goal_progress: Option<GoalProgress>,
    pub gps_quality: GpsQualityState,
    pub last_point_accepted: Option<bool>,
    pub last_rejection_reason: Option<RejectionReason>,
}

pub struct WorkoutSessionController {
    session: WorkoutSession,
    config: SessionConfig,
    goal: Option<WorkoutGoal>,

    gps_filter: GpsFilter,
    time_keeper: TimeKeeper,
    distance_engine: DistanceEngine,
    speed_engine: SpeedEngine,
    fastest_pace_tracker: FastestPaceTracker,
    movement_detector: MovementDetector,
    autopause_engine: AutoPauseEngine,
    classifier: ActivityClassifier,
    vehicle_detector: VehicleMotionDetector,
    step_engine: StepEngine,
    elevation_engine: ElevationEngine,
    heart_rate_engine: HeartRateEngine,
    split_engine: SplitEngine,
    route_builder: RouteBuilder,
    gps_quality_monitor: GpsQualityMonitor,
    coaching: CoachingEventBuilder,

    last_gps_quality: GpsQualityState,
    crossed_first_km: bool,
    crossed_first_mile: bool,
    crossed_halfway: bool,
    vehicle_warning_emitted: bool,
    /// Prior personal bests supplied by Dart at session-creation time (from
    /// local/cloud history). Used only for the *live* "personal record
    /// possible" coaching nudge (spec section 19) — the authoritative,
    /// final record detection still happens post-finish via
    /// `stride_detect_personal_records`, since only then are all totals
    /// final. `None` means the caller didn't supply history (e.g. first
    /// workout ever, or Dart chose not to look it up); in that case no
    /// live nudge is emitted, but final detection still runs normally.
    prior_bests: Option<PriorBests>,
    personal_record_nudge_emitted: bool,
    /// Wall-clock time of the last hydration reminder, so it can repeat
    /// periodically during long workouts without spamming every tick.
    last_hydration_reminder_at: Option<EpochMillis>,
    /// Set once the "workout ending" nudge has fired for the current goal
    /// (distance/duration goals only), so it doesn't repeat every sample.
    workout_ending_nudge_emitted: bool,
}

/// Errors surfaced to the Dart layer for illegal operations. Kept as a
/// simple string-friendly enum so it serializes trivially across FFI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ControllerError {
    IllegalStateTransition { from: WorkoutState, to: WorkoutState },
    AlreadyRecording,
    NotRecording,
    NoActiveWorkout,
}

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControllerError::IllegalStateTransition { from, to } => {
                write!(f, "Illegal transition from {from:?} to {to:?}")
            }
            ControllerError::AlreadyRecording => write!(f, "A workout is already recording"),
            ControllerError::NotRecording => write!(f, "No workout is currently recording"),
            ControllerError::NoActiveWorkout => write!(f, "No active workout exists"),
        }
    }
}

impl WorkoutSessionController {
    pub fn new(
        user_id: impl Into<String>,
        activity_type: ActivityType,
        weight_kg: f64,
        started_at: EpochMillis,
        config: SessionConfig,
    ) -> Self {
        let session = WorkoutSession::new(user_id, activity_type, started_at, weight_kg);
        let workout_id = session.workout_id.clone();

        Self {
            gps_filter: GpsFilter::new(config.filter_config),
            time_keeper: TimeKeeper::new(started_at),
            distance_engine: DistanceEngine::new(),
            speed_engine: SpeedEngine::default_config(),
            fastest_pace_tracker: FastestPaceTracker::new(50.0),
            movement_detector: MovementDetector::default_config(),
            autopause_engine: AutoPauseEngine::new(config.autopause_config),
            classifier: ActivityClassifier::new(config.classification_thresholds),
            vehicle_detector: VehicleMotionDetector::default_config(),
            step_engine: StepEngine::default_config(),
            elevation_engine: ElevationEngine::default_config(),
            heart_rate_engine: HeartRateEngine::new(config.max_hr_estimate),
            split_engine: SplitEngine::new(config.split_distance_meters, started_at),
            route_builder: RouteBuilder::default_config(),
            gps_quality_monitor: GpsQualityMonitor::default_config(),
            coaching: CoachingEventBuilder::new(workout_id),
            last_gps_quality: GpsQualityState::Unavailable,
            crossed_first_km: false,
            crossed_first_mile: false,
            crossed_halfway: false,
            vehicle_warning_emitted: false,
            prior_bests: None,
            personal_record_nudge_emitted: false,
            last_hydration_reminder_at: None,
            workout_ending_nudge_emitted: false,
            session,
            config,
            goal: None,
        }
    }

    /// Supplies the user's prior personal bests (loaded by Dart from local
    /// or cloud history) so the controller can emit a live "personal
    /// record possible" coaching nudge (spec section 19) once the current
    /// pace/distance trajectory looks likely to beat a known best. This is
    /// advisory only — the authoritative check happens post-finish via
    /// `stride_detect_personal_records`.
    pub fn set_prior_bests(&mut self, prior: PriorBests) {
        self.prior_bests = Some(prior);
    }

    /// Rebuilds a controller from a previously-persisted [`WorkoutCheckpoint`]
    /// plus the small amount of extra context the checkpoint itself doesn't
    /// carry (user id, activity type, weight — these live in the Dart-side
    /// `active_workout` SQLite row alongside the checkpoint JSON).
    ///
    /// This is the crash/kill/restart recovery path (spec section 23): after
    /// the app relaunches and finds a leftover `active_workout` row plus its
    /// most recent checkpoint, the Dart `RecoveryService` calls this to get
    /// a fresh, live `WorkoutSessionController` continuing from where the
    /// checkpoint left off, rather than losing the in-progress workout.
    ///
    /// The rebuilt controller resumes in the `Paused` state regardless of
    /// what state the checkpoint captured — the user must explicitly tap
    /// "Resume" to continue live GPS tracking, since we can't safely assume
    /// the phone was still moving through the entire gap while the process
    /// was dead.
    pub fn restore_from_checkpoint(
        checkpoint: &WorkoutCheckpoint,
        user_id: impl Into<String>,
        activity_type: ActivityType,
        weight_kg: f64,
        config: SessionConfig,
    ) -> Self {
        let mut controller = Self::new(user_id, activity_type, weight_kg, checkpoint.started_at, config);

        controller.session.workout_id = checkpoint.workout_id.clone();
        controller.session.total_distance_meters = checkpoint.total_distance_meters;
        controller.session.active_ms = checkpoint.active_ms;
        controller.session.paused_ms = checkpoint.paused_ms;
        controller.session.elapsed_ms = checkpoint.active_ms + checkpoint.paused_ms;
        controller.session.last_point_at = checkpoint.last_point_at;
        // Always land in Paused: NotStarted -> Starting -> Active -> Paused
        // are all legal forward transitions, so this is safe regardless of
        // which state the checkpoint captured.
        controller.session.state = WorkoutState::Paused;

        let last_point = match (checkpoint.last_point_lat, checkpoint.last_point_lon) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        };
        controller
            .distance_engine
            .seed(last_point, checkpoint.total_distance_meters);
        controller.distance_engine.set_paused(true);

        controller.time_keeper.seed(
            crate::engine::timekeeping::TimeTotals {
                elapsed_ms: checkpoint.active_ms + checkpoint.paused_ms,
                active_ms: checkpoint.active_ms,
                moving_ms: 0,
                paused_ms: checkpoint.paused_ms,
            },
            checkpoint.checkpoint_at,
        );
        controller.time_keeper.set_paused(true);
        controller.autopause_engine.force_state(
            checkpoint.checkpoint_at,
            crate::engine::autopause::AutoPauseState::AutoPaused,
        );

        controller
    }

    pub fn workout_id(&self) -> &str {
        &self.session.workout_id
    }

    pub fn state(&self) -> WorkoutState {
        self.session.state
    }

    pub fn session(&self) -> &WorkoutSession {
        &self.session
    }

    pub fn set_goal(&mut self, goal: WorkoutGoal) {
        self.goal = Some(goal);
    }

    // ─── State transitions ──────────────────────────────────────────────

    fn transition(&mut self, to: WorkoutState) -> Result<(), ControllerError> {
        if !self.session.state.can_transition_to(to) {
            return Err(ControllerError::IllegalStateTransition {
                from: self.session.state,
                to,
            });
        }
        self.session.state = to;
        Ok(())
    }

    pub fn start(&mut self) -> Result<CoachingEvent, ControllerError> {
        self.transition(WorkoutState::Starting)?;
        self.transition(WorkoutState::Active)?;
        Ok(self.coaching.workout_started(self.session.started_at))
    }

    pub fn pause(&mut self, now_ms: EpochMillis) -> Result<(), ControllerError> {
        self.transition(WorkoutState::Paused)?;
        self.time_keeper.set_paused(true);
        self.distance_engine.set_paused(true);
        let _ = now_ms;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), ControllerError> {
        self.transition(WorkoutState::Active)?;
        self.time_keeper.set_paused(false);
        self.distance_engine.set_paused(false);
        Ok(())
    }

    /// Discards the workout without saving. Valid from `Paused` (per the
    /// state machine) — Dart should route "discard while active" through
    /// pause-then-discard, or callers may relax this by extending
    /// `WorkoutState::can_transition_to` if a direct Active->Discarded
    /// path is desired later.
    pub fn discard(&mut self) -> Result<(), ControllerError> {
        self.transition(WorkoutState::Discarded)
    }

    // ─── Live sensor ingestion ──────────────────────────────────────────

    /// Processes one raw GPS reading: filters it, and if accepted, feeds
    /// every downstream engine (distance, speed, pace, route, elevation,
    /// classification, splits, goal progress) and returns a full
    /// [`LiveUpdate`] snapshot for the UI.
    pub fn add_location_sample(&mut self, mut point: WorkoutPoint, now_ms: EpochMillis) -> LiveUpdate {
        point.workout_id = self.session.workout_id.clone();
        let evaluated = self.gps_filter.evaluate(point);

        self.gps_quality_monitor.add_sample(GpsQualitySample {
            at_ms: evaluated.recorded_at,
            accuracy_meters: evaluated.accuracy_meters,
            accepted: evaluated.accepted,
        });

        let mut coaching_events = Vec::new();
        let last_accepted = evaluated.accepted;
        let last_rejection = evaluated.rejection_reason;

        if evaluated.accepted {
            self.time_keeper.record_point_received(evaluated.recorded_at);

            let delta_m = self
                .distance_engine
                .add_point(evaluated.latitude, evaluated.longitude);

            self.route_builder.add_accepted_point(evaluated.clone());

            if let Some(alt) = evaluated.altitude_meters {
                self.elevation_engine.add_sample(alt);
            }

            // Instantaneous speed: prefer distance/time derivation, which
            // is more robust than device-reported speed for slow walking.
            let instantaneous_speed = if delta_m > 0.0 {
                // Use time since previous accepted point if available via
                // the filter's internal state indirectly - approximate
                // using device speed as a fallback when we can't derive.
                evaluated.speed_meters_per_second.unwrap_or(0.0)
            } else {
                0.0
            };
            self.speed_engine.add_sample(instantaneous_speed.max(0.0));

            let movement_sample = MovementSample {
                at_ms: evaluated.recorded_at,
                distance_since_last_m: delta_m,
                speed_mps: self.speed_engine.smoothed_mps(),
                accuracy_m: evaluated.accuracy_meters,
                step_detected: None,
            };
            self.movement_detector.add_sample(movement_sample);
            let movement_signal = self.movement_detector.current_signal();
            let is_stationary = movement_signal == MovementSignal::Stationary;

            let autopause_transition = self.autopause_engine.observe(now_ms, is_stationary);
            match autopause_transition {
                AutoPauseTransition::TriggeredAutoPause => {
                    self.session.is_auto_paused = true;
                    self.time_keeper.set_paused(true);
                    self.distance_engine.set_paused(true);
                    coaching_events.push(self.coaching.auto_pause_activated(now_ms));
                }
                AutoPauseTransition::TriggeredAutoResume => {
                    self.session.is_auto_paused = false;
                    self.time_keeper.set_paused(false);
                    self.distance_engine.set_paused(false);
                    coaching_events.push(self.coaching.auto_resume_activated(now_ms));
                }
                AutoPauseTransition::None => {}
            }

            let classified = self.classifier.add_sample(ClassificationSample {
                at_ms: evaluated.recorded_at,
                speed_mps: self.speed_engine.smoothed_mps(),
                cadence_spm: None,
            });
            self.session.movement_state = classified;

            let vehicle_suspected = self.vehicle_detector.observe(
                evaluated.recorded_at,
                self.speed_engine.smoothed_mps(),
                None,
            );
            if vehicle_suspected && !self.vehicle_warning_emitted {
                self.vehicle_warning_emitted = true;
                coaching_events.push(self.coaching.vehicle_motion_suspected(now_ms));
            } else if !vehicle_suspected {
                self.vehicle_warning_emitted = false;
            }

            self.time_keeper.tick(now_ms, movement_signal == MovementSignal::Moving);

            let totals = self.distance_engine.totals();
            self.session.total_distance_meters = totals.total_meters;

            let time_totals = self.time_keeper.totals();
            self.session.elapsed_ms = time_totals.elapsed_ms;
            self.session.active_ms = time_totals.active_ms;
            self.session.moving_ms = time_totals.moving_ms;
            self.session.paused_ms = time_totals.paused_ms;

            let speed_snapshot = self.speed_engine.snapshot(
                totals.moving_meters,
                time_totals.moving_ms,
                totals.total_meters,
                time_totals.elapsed_ms,
            );
            self.session.current_speed_mps = speed_snapshot.current_mps;
            self.session.average_speed_mps = speed_snapshot.average_overall_mps;
            self.session.max_speed_mps = speed_snapshot.max_valid_mps;

            self.session.current_pace_sec_per_km = pace::speed_to_pace_sec_per_km(speed_snapshot.smoothed_mps);
            self.session.average_pace_sec_per_km =
                pace::pace_sec_per_km(time_totals.active_ms, totals.total_meters);

            self.fastest_pace_tracker
                .observe(self.session.current_pace_sec_per_km, delta_m.max(0.0) * 20.0);
            self.session.fastest_pace_sec_per_km = self.fastest_pace_tracker.fastest_sec_per_km();

            let elevation_totals = self.elevation_engine.totals();
            self.session.elevation_gain_meters = elevation_totals.gain_meters;
            self.session.elevation_loss_meters = elevation_totals.loss_meters;
            self.session.min_altitude_meters = elevation_totals.min_altitude_meters;
            self.session.max_altitude_meters = elevation_totals.max_altitude_meters;

            let calorie_result = calories::estimate(&CalorieInputs {
                wearable_kcal: None,
                weight_kg: Some(self.session.weight_kg),
                duration_ms: time_totals.active_ms,
                distance_meters: totals.total_meters,
                average_speed_mps: speed_snapshot.average_overall_mps,
                average_heart_rate_bpm: self.heart_rate_engine.average_bpm(),
                age_years: None,
                is_male: None,
                activity_type: self.session.resolved_activity_type,
                elevation_gain_meters: elevation_totals.gain_meters,
            });
            self.session.calories_estimated = calorie_result.kcal;
            self.session.calorie_estimate_method = calorie_result.method.as_str().to_string();

            self.split_engine.observe_live_metrics(
                self.heart_rate_engine.current_bpm().map(|b| b as f64),
                Some(self.step_engine.current_cadence_spm()),
            );
            let newly_completed_splits = self.split_engine.update(
                now_ms,
                totals.total_meters,
                elevation_totals.gain_meters,
                elevation_totals.loss_meters,
                self.session.weight_kg,
                self.session.resolved_activity_type,
            );
            for split in &newly_completed_splits {
                coaching_events.push(self.coaching.split_completed(
                    now_ms,
                    split.split_number,
                    split.average_pace_sec_per_km,
                ));
            }

            if !self.crossed_first_km && totals.total_meters >= METERS_PER_KILOMETER {
                self.crossed_first_km = true;
                coaching_events.push(
                    self.coaching
                        .first_kilometer_completed(now_ms, self.session.average_pace_sec_per_km),
                );
            }
            if !self.crossed_first_mile && totals.total_meters >= METERS_PER_MILE {
                self.crossed_first_mile = true;
                coaching_events.push(
                    self.coaching
                        .first_mile_completed(now_ms, self.session.average_pace_sec_per_km),
                );
            }

            let goal_progress = self.goal.as_ref().map(|g| {
                let current_value = match g.goal_type {
                    crate::models::GoalType::Distance => totals.total_meters,
                    crate::models::GoalType::Duration => time_totals.active_ms as f64,
                    crate::models::GoalType::Calories => self.session.calories_estimated,
                    crate::models::GoalType::Steps => self.session.step_count as f64,
                    crate::models::GoalType::Pace => self.session.average_pace_sec_per_km,
                };
                let progress = g.progress(current_value, time_totals.active_ms, totals.total_meters);

                if !self.crossed_halfway && progress.percent_complete >= 50.0 {
                    self.crossed_halfway = true;
                }
                progress
            });

            if let Some(ref progress) = goal_progress {
                if progress.is_complete {
                    coaching_events.push(self.coaching.goal_completed(now_ms));
                } else if !self.workout_ending_nudge_emitted && progress.percent_complete >= 90.0 {
                    // Distance/duration goal is nearly done — nudge the
                    // user that the workout is wrapping up (spec section
                    // 19's "workout ending" event).
                    self.workout_ending_nudge_emitted = true;
                    coaching_events.push(self.coaching.workout_ending(now_ms));
                }
            }

            // Hydration reminder: fires every 30 minutes of active time
            // while a workout is in progress, purely local/rule-based (no
            // AI/network dependency), per spec section 19's note that core
            // coaching must work fully offline.
            const HYDRATION_REMINDER_INTERVAL_MS: i64 = 30 * 60 * 1000;
            let should_remind_hydration = match self.last_hydration_reminder_at {
                None => time_totals.active_ms >= HYDRATION_REMINDER_INTERVAL_MS,
                Some(last) => now_ms - last >= HYDRATION_REMINDER_INTERVAL_MS,
            };
            if should_remind_hydration && time_totals.active_ms > 0 {
                self.last_hydration_reminder_at = Some(now_ms);
                coaching_events.push(self.coaching.hydration_reminder(now_ms));
            }

            // Live "personal record possible" nudge: only fires once per
            // workout, and only once the workout has covered enough ground
            // that pace/distance projections are meaningful (avoids a
            // premature nudge off a single fast opening burst).
            if !self.personal_record_nudge_emitted && totals.total_meters >= METERS_PER_KILOMETER {
                if let Some(prior) = &self.prior_bests {
                    let mut candidate: Option<&str> = None;
                    if let Some(best_dist) = prior.longest_distance_meters {
                        if totals.total_meters > best_dist * 0.9 {
                            candidate = Some("longest_distance");
                        }
                    }
                    if candidate.is_none() {
                        if let Some(best_pace) = prior.highest_average_pace_sec_per_km {
                            if self.session.average_pace_sec_per_km > 0.0
                                && self.session.average_pace_sec_per_km < best_pace * 1.02
                            {
                                candidate = Some("highest_average_pace");
                            }
                        }
                    }
                    if let Some(record_type) = candidate {
                        self.personal_record_nudge_emitted = true;
                        coaching_events.push(self.coaching.personal_record_possible(now_ms, record_type));
                    }
                }
            }

            let quality = self.gps_quality_monitor.evaluate(now_ms);
            if quality != self.last_gps_quality {
                if matches!(quality, GpsQualityState::Poor | GpsQualityState::Unavailable) {
                    coaching_events.push(self.coaching.gps_quality_poor(now_ms));
                } else if matches!(self.last_gps_quality, GpsQualityState::Poor | GpsQualityState::Unavailable) {
                    coaching_events.push(self.coaching.gps_quality_restored(now_ms));
                }
                self.last_gps_quality = quality;
            }

            return LiveUpdate {
                session: self.session.clone(),
                newly_completed_splits,
                coaching_events,
                goal_progress,
                gps_quality: quality,
                last_point_accepted: Some(last_accepted),
                last_rejection_reason: last_rejection,
            };
        }

        // Rejected point: still advance the clock/quality monitor so the
        // UI reflects "stale GPS" appropriately, but no stats change.
        self.time_keeper.tick(now_ms, false);
        let quality = self.gps_quality_monitor.evaluate(now_ms);

        LiveUpdate {
            session: self.session.clone(),
            newly_completed_splits: Vec::new(),
            coaching_events,
            goal_progress: None,
            gps_quality: quality,
            last_point_accepted: Some(last_accepted),
            last_rejection_reason: last_rejection,
        }
    }

    pub fn add_heart_rate_sample(&mut self, at_ms: EpochMillis, bpm: u16) {
        self.heart_rate_engine.add_sample(at_ms, bpm);
        self.session.average_heart_rate = self.heart_rate_engine.average_bpm().map(|v| v.round() as i32);
    }

    pub fn add_step_delta(&mut self, at_ms: EpochMillis, delta: u32, source: SensorSource) {
        self.step_engine.add_step_delta(at_ms, delta, source);
        self.session.step_count = self.step_engine.total_steps();
        self.session.current_cadence_spm = self.step_engine.current_cadence_spm();
        self.session.average_cadence_spm = self.step_engine.average_cadence_spm();
    }

    /// A periodic tick with no new GPS point (e.g. once per second),
    /// used to keep elapsed/paused time and GPS-staleness detection
    /// advancing even when no fix has arrived recently.
    pub fn tick(&mut self, now_ms: EpochMillis) -> LiveUpdate {
        let is_moving = self.movement_detector.current_signal() == MovementSignal::Moving;
        self.time_keeper.tick(now_ms, is_moving && !self.session.is_auto_paused);

        let time_totals = self.time_keeper.totals();
        self.session.elapsed_ms = time_totals.elapsed_ms;
        self.session.active_ms = time_totals.active_ms;
        self.session.moving_ms = time_totals.moving_ms;
        self.session.paused_ms = time_totals.paused_ms;

        let quality = self.gps_quality_monitor.evaluate(now_ms);
        let mut coaching_events = Vec::new();
        if quality != self.last_gps_quality {
            if matches!(quality, GpsQualityState::Poor | GpsQualityState::Unavailable) {
                coaching_events.push(self.coaching.gps_quality_poor(now_ms));
            }
            self.last_gps_quality = quality;
        }

        if self.heart_rate_engine.is_stale(now_ms, 30_000) {
            coaching_events.push(self.coaching.heart_rate_unavailable(now_ms));
        }

        LiveUpdate {
            session: self.session.clone(),
            newly_completed_splits: Vec::new(),
            coaching_events,
            goal_progress: None,
            gps_quality: quality,
            last_point_accepted: None,
            last_rejection_reason: None,
        }
    }

    /// Manual lap button.
    pub fn manual_lap(&mut self, now_ms: EpochMillis) -> WorkoutSplit {
        let totals = self.distance_engine.totals();
        let elevation_totals = self.elevation_engine.totals();
        self.split_engine.manual_lap(
            now_ms,
            totals.total_meters,
            elevation_totals.gain_meters,
            elevation_totals.loss_meters,
            self.session.weight_kg,
            self.session.resolved_activity_type,
        )
    }

    pub fn build_checkpoint(&self, now_ms: EpochMillis) -> WorkoutCheckpoint {
        let last_point = self
            .gps_filter
            .last_accepted()
            .map(|p| (p.latitude, p.longitude));
        crate::engine::recovery::build_checkpoint(
            self.session.workout_id.clone(),
            self.session.state,
            self.session.started_at,
            last_point,
            self.session.last_point_at,
            self.session.total_distance_meters,
            self.session.active_ms,
            self.session.paused_ms,
            self.split_engine.completed_splits().len() as u32 + 1,
            self.route_builder.full_resolution_points().len() as u64,
            now_ms,
        )
    }

    /// Ends the workout, computes final totals, and produces the
    /// finalized [`WorkoutSummary`] ready for local persistence as
    /// `sync_pending`. Does not itself write anything to disk.
    pub fn finish(&mut self, now_ms: EpochMillis) -> Result<WorkoutSummary, ControllerError> {
        self.transition(WorkoutState::Finishing)?;

        self.session.ended_at = Some(now_ms);
        let time_totals = self.time_keeper.totals();
        let distance_totals = self.distance_engine.totals();
        let elevation_totals = self.elevation_engine.totals();

        let start_loc = self.route_builder.start_location();
        let end_loc = self.route_builder.end_location();
        let polyline = self.route_builder.simplified_polyline();

        let goal_result = self.goal.as_ref().map(|g| {
            let current_value = match g.goal_type {
                crate::models::GoalType::Distance => distance_totals.total_meters,
                crate::models::GoalType::Duration => time_totals.active_ms as f64,
                crate::models::GoalType::Calories => self.session.calories_estimated,
                crate::models::GoalType::Steps => self.session.step_count as f64,
                crate::models::GoalType::Pace => self.session.average_pace_sec_per_km,
            };
            g.progress(current_value, time_totals.active_ms, distance_totals.total_meters)
        });

        let mut summary = WorkoutSummary {
            workout_id: self.session.workout_id.clone(),
            user_id: self.session.user_id.clone(),
            activity_type: self.session.resolved_activity_type,
            started_at: self.session.started_at,
            ended_at: now_ms,
            total_duration_ms: time_totals.elapsed_ms,
            moving_ms: time_totals.moving_ms,
            paused_ms: time_totals.paused_ms,
            distance_meters: distance_totals.total_meters,
            average_pace_sec_per_km: self.session.average_pace_sec_per_km,
            average_speed_mps: self.session.average_speed_mps,
            max_speed_mps: self.session.max_speed_mps,
            calories_estimated: self.session.calories_estimated,
            calorie_estimate_method: self.session.calorie_estimate_method.clone(),
            step_count: self.session.step_count,
            average_cadence_spm: self.session.average_cadence_spm,
            average_heart_rate_bpm: self.heart_rate_engine.average_bpm(),
            max_heart_rate_bpm: self.heart_rate_engine.max_bpm(),
            min_heart_rate_bpm: self.heart_rate_engine.min_bpm(),
            elevation_gain_meters: elevation_totals.gain_meters,
            elevation_loss_meters: elevation_totals.loss_meters,
            splits: self.split_engine.completed_splits().to_vec(),
            start_latitude: start_loc.map(|l| l.0),
            start_longitude: start_loc.map(|l| l.1),
            end_latitude: end_loc.map(|l| l.0),
            end_longitude: end_loc.map(|l| l.1),
            encoded_polyline: if polyline.is_empty() { None } else { Some(polyline) },
            goal_result,
            has_flagged_segments: self.vehicle_warning_emitted,
            validation_warnings: Vec::new(),
            is_blocked: false,
            block_reasons: Vec::new(),
        };

        // Run validation (spec section 26) now that all totals are final.
        // `is_duplicate_of_existing_workout` is always `false` here because
        // the Rust engine performs no I/O and has no access to workout
        // history; the Dart side is expected to re-run validation via the
        // standalone `stride_validate` FFI entry point (passing the real
        // duplicate-check result from local storage) before persisting,
        // OR-ing that result with this one. `gps_gap_count` is approximated
        // as the number of route-segment breaks, since a new segment is
        // only started after a significant pause/gap in accepted points.
        let route_point_count = self.route_builder.full_resolution_points().len() as u32;
        let gps_gap_count = self.route_builder.segments().len().saturating_sub(1) as u32;
        let validation_input = ValidationInput {
            started_at: summary.started_at,
            ended_at: summary.ended_at,
            distance_meters: summary.distance_meters,
            duration_ms: summary.total_duration_ms,
            average_speed_mps: summary.average_speed_mps,
            max_speed_mps: summary.max_speed_mps,
            calories_estimated: summary.calories_estimated,
            route_point_count,
            has_vehicle_flagged_segment: summary.has_flagged_segments,
            gps_gap_count,
            is_duplicate_of_existing_workout: false,
        };
        let result = validation::validate(&validation_input);
        summary.is_blocked = result.is_blocked;
        summary.block_reasons = result.block_reasons;
        summary.validation_warnings = result.warnings;

        self.transition(WorkoutState::Completed)?;
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LocationSource;

    fn point(lat: f64, lon: f64, t: i64, speed: f64) -> WorkoutPoint {
        let mut p = WorkoutPoint::new("temp", lat, lon, t, LocationSource::PhoneGps);
        p.accuracy_meters = Some(6.0);
        p.speed_meters_per_second = Some(speed);
        p
    }

    fn new_controller() -> WorkoutSessionController {
        WorkoutSessionController::new(
            "user1",
            ActivityType::Walk,
            70.0,
            0,
            SessionConfig::default(),
        )
    }

    #[test]
    fn cannot_pause_before_starting() {
        let mut c = new_controller();
        assert!(c.pause(0).is_err());
    }

    #[test]
    fn start_transitions_to_active() {
        let mut c = new_controller();
        c.start().unwrap();
        assert_eq!(c.state(), WorkoutState::Active);
    }

    #[test]
    fn double_start_is_illegal() {
        let mut c = new_controller();
        c.start().unwrap();
        // Starting state machine directly again should fail since we're
        // already Active (NotStarted->Starting is the only legal start).
        let result = c.transition(WorkoutState::Starting);
        assert!(result.is_err());
    }

    #[test]
    fn pause_then_resume_round_trip() {
        let mut c = new_controller();
        c.start().unwrap();
        c.pause(1000).unwrap();
        assert_eq!(c.state(), WorkoutState::Paused);
        c.resume().unwrap();
        assert_eq!(c.state(), WorkoutState::Active);
    }

    #[test]
    fn accepted_gps_point_updates_distance() {
        let mut c = new_controller();
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        let update = c.add_location_sample(point(40.0000630, -73.0, 6_000, 1.4), 6_000);
        assert!(update.session.total_distance_meters > 0.0);
        assert_eq!(update.last_point_accepted, Some(true));
    }

    #[test]
    fn rejected_gps_point_does_not_update_distance() {
        let mut c = new_controller();
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.0), 1_000);
        // huge implausible jump
        let update = c.add_location_sample(point(41.0, -73.0, 2_000, 1.0), 2_000);
        assert_eq!(update.last_point_accepted, Some(false));
        assert_eq!(update.session.total_distance_meters, 0.0);
    }

    #[test]
    fn finish_produces_summary_and_completes_state() {
        let mut c = new_controller();
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        c.add_location_sample(point(40.0000630, -73.0, 6_000, 1.4), 6_000);
        let summary = c.finish(10_000).unwrap();
        assert_eq!(c.state(), WorkoutState::Completed);
        assert_eq!(summary.workout_id, c.workout_id());
        assert!(summary.total_duration_ms >= 0);
    }

    #[test]
    fn goal_progress_reported_when_goal_set() {
        let mut c = new_controller();
        c.set_goal(WorkoutGoal {
            goal_type: crate::models::GoalType::Distance,
            target_value: 5.0, // tiny target so it's easy to trigger completion
        });
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        let update = c.add_location_sample(point(40.0000630, -73.0, 6_000, 1.4), 6_000);
        assert!(update.goal_progress.is_some());
    }

    #[test]
    fn heart_rate_and_steps_flow_into_session() {
        let mut c = new_controller();
        c.start().unwrap();
        c.add_heart_rate_sample(1000, 130);
        c.add_step_delta(1000, 20, SensorSource::PhoneStepSensor);
        assert_eq!(c.session().step_count, 20);
        assert!(c.session().average_heart_rate.is_some());
    }

    #[test]
    fn manual_lap_creates_a_split() {
        let mut c = new_controller();
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        c.add_location_sample(point(40.0000630, -73.0, 6_000, 1.4), 6_000);
        let split = c.manual_lap(10_000);
        assert_eq!(split.split_number, 1);
    }

    #[test]
    fn checkpoint_reflects_current_progress() {
        let mut c = new_controller();
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        let cp = c.build_checkpoint(2_000);
        assert_eq!(cp.workout_id, c.workout_id());
        assert_eq!(cp.state, WorkoutState::Active);
    }

    #[test]
    fn restore_from_checkpoint_continues_distance_and_can_resume() {
        let mut c = new_controller();
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        c.add_location_sample(point(40.0000630, -73.0, 6_000, 1.4), 6_000);
        let distance_before = c.session().total_distance_meters;
        assert!(distance_before > 0.0);

        let checkpoint = c.build_checkpoint(7_000);

        // Simulate process death + relaunch: a brand new controller is
        // rebuilt purely from the checkpoint (+ small bit of side-channel
        // context normally read back from the SQLite active_workout row).
        let restored = WorkoutSessionController::restore_from_checkpoint(
            &checkpoint,
            "user-1",
            ActivityType::Walk,
            70.0,
            SessionConfig::default(),
        );

        assert_eq!(restored.workout_id(), c.workout_id());
        assert_eq!(restored.state(), WorkoutState::Paused);
        assert_eq!(
            restored.session().total_distance_meters,
            distance_before
        );

        // User taps Resume: must be legal per the state machine.
        let mut restored = restored;
        restored.resume().unwrap();
        assert_eq!(restored.state(), WorkoutState::Active);

        // And distance keeps accumulating from the restored baseline
        // rather than resetting to zero. Per the standard resume
        // semantics (see distance.rs `resume_does_not_count_gap_as_distance`),
        // the first point after resume just re-anchors; the second point
        // is where accumulation resumes.
        restored.add_location_sample(point(40.0001260, -73.0, 12_000, 1.4), 12_000);
        let update = restored.add_location_sample(point(40.0001890, -73.0, 17_000, 1.4), 17_000);
        assert!(update.session.total_distance_meters > distance_before);
    }

    fn new_controller_started_at(started_at: i64) -> WorkoutSessionController {
        WorkoutSessionController::new(
            "user1",
            ActivityType::Walk,
            70.0,
            started_at,
            SessionConfig::default(),
        )
    }

    #[test]
    fn finish_runs_validation_and_populates_warnings() {
        let mut c = new_controller_started_at(1_000);
        c.start().unwrap();
        // At least one tick so duration_ms > 0, but still zero distance
        // (no location samples) -> should warn, not block.
        c.tick(5_000);
        let summary = c.finish(6_000).unwrap();
        assert!(!summary.is_blocked, "zero-distance short workout should warn, not block");
        assert!(summary
            .validation_warnings
            .iter()
            .any(|w| w.contains("Zero-distance") || w.contains("short")));
    }

    #[test]
    fn finish_does_not_block_a_normal_workout() {
        let mut c = new_controller_started_at(1_000);
        c.start().unwrap();
        c.add_location_sample(point(40.0, -73.0, 2_000, 1.4), 2_000);
        c.add_location_sample(point(40.0000630, -73.0, 7_000, 1.4), 7_000);
        let summary = c.finish(40_000).unwrap();
        assert!(!summary.is_blocked);
    }

    #[test]
    fn finish_blocks_when_missing_start_time() {
        // started_at = 0 is treated as "missing" per validation rules.
        let mut c = new_controller();
        c.start().unwrap();
        let summary = c.finish(5_000).unwrap();
        assert!(summary.is_blocked);
        assert!(!summary.block_reasons.is_empty());
    }

    #[test]
    fn hydration_reminder_fires_after_interval() {
        let mut c = new_controller_started_at(1_000);
        c.start().unwrap();
        // Two accepted points 31 minutes apart -> active_ms comfortably
        // past the 30-minute hydration interval.
        c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        let update = c.add_location_sample(
            point(40.0000630, -73.0, 1_000 + 31 * 60 * 1000, 1.4),
            1_000 + 31 * 60 * 1000,
        );
        assert!(update
            .coaching_events
            .iter()
            .any(|e| matches!(e.event_type, crate::models::CoachingEventType::HydrationReminder)));
    }

    #[test]
    fn personal_record_nudge_fires_when_near_prior_best_pace() {
        let mut c = new_controller_started_at(1_000);
        c.set_prior_bests(crate::engine::personal_records::PriorBests {
            highest_average_pace_sec_per_km: Some(1_000_000.0), // trivially easy to approach
            ..Default::default()
        });
        c.start().unwrap();
        // Accumulate >1km across several small, plausible steps (each
        // within the GPS filter's max-jump/max-speed limits) so the
        // 1km live-nudge threshold is crossed via accepted points.
        let mut all_events = Vec::new();
        let mut last_distance = 0.0;
        let update = c.add_location_sample(point(40.0, -73.0, 1_000, 1.4), 1_000);
        all_events.extend(update.coaching_events);
        for i in 1..=20 {
            let t = 1_000 + i * 10_000; // 10s between points
            let lat = 40.0 + (i as f64) * 0.00063; // ~70m per step -> 7 m/s, within limits
            let update = c.add_location_sample(point(lat, -73.0, t, 1.4), t);
            last_distance = update.session.total_distance_meters;
            all_events.extend(update.coaching_events);
        }
        assert!(last_distance > METERS_PER_KILOMETER);
        assert!(all_events.iter().any(|e| matches!(
            e.event_type,
            crate::models::CoachingEventType::PersonalRecordPossible
        )));
    }

    #[test]
    fn discard_requires_paused_state_per_state_machine() {
        let mut c = new_controller();
        c.start().unwrap();
        // Directly from Active, discard should fail per state machine
        // (Active -> Paused -> Discarded is the valid path).
        assert!(c.discard().is_err());
        c.pause(1000).unwrap();
        assert!(c.discard().is_ok());
        assert_eq!(c.state(), WorkoutState::Discarded);
    }
}
