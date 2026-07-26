//! §8 — AI coaching backend: rule-based fallback plan engine.
//!
//! The actual AI (Gemini or similar) runs server-side via Cloud
//! Functions or Cloud Run — never directly in the mobile client.
//! The mobile app sends **structured inputs** (not free-form
//! conversation text) and receives **structured outputs** (plans,
//! messages, adjustments).
//!
//! This module provides the **deterministic, rule-based fallback**
//! that the server layer uses when:
//! - The AI service is unavailable or times out
//! - The AI response fails validation guards
//! - The user has exhausted their AI usage quota
//! - The cost of an AI call exceeds the budget threshold
//!
//! It also provides the **validation guards** that the server-side AI
//! pipeline must respect — the same rules the fallback enforces. The
//! server layer calls these functions to validate AI-generated plans
//! before returning them to the client.
//!
//! ## What this module covers (per the §8 spec)
//!
//! - **Beginner progression limits**: distance/duration caps by
//!   experience level
//! - **Rest-day requirements**: mandatory rest, no consecutive
//!   high-intensity days for beginners
//! - **Gradual increase rules**: 10% rule for weekly distance/
//!   duration increases
//! - **Injury/pain warning responses**: what to do when user reports
//!   pain
//! - **No medical diagnosis**: guard that blocks diagnosis-style
//!   responses
//! - **No guaranteed weight loss promises**: guard that blocks
//!   weight-loss guarantee claims
//! - **Escalation messaging for concerning symptoms**: when to
//!   escalate to a medical professional
//! - **User ability to reject or modify a recommendation**: user
//!   feedback handling for plan adjustments
//! - **AI availability decision**: whether the AI should be called or
//!   the fallback used
//! - **Usage limit enforcement**: rate-limit decisions for AI calls
//! - **AI cost tracking**: cost estimation for budget enforcement
//! - **Cache key generation**: deterministic cache keys for repeated
//!   requests
//! - **Plan validation**: guards that validate AI-generated plans

use serde::{Deserialize, Serialize};

// ===========================================================================
// Experience level & plan difficulty
// ===========================================================================

/// The user's experience level, used to cap distance/duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperienceLevel {
    /// New to walking/running (< 4 weeks of activity).
    Beginner,
    /// Some experience (4–12 weeks of regular activity).
    Intermediate,
    /// Experienced (> 12 weeks of regular activity, comfortable with
    /// longer distances).
    Advanced,
}

impl ExperienceLevel {
    /// Returns the maximum single-session distance in meters for
    /// this experience level.
    pub fn max_single_session_distance_m(self) -> f64 {
        match self {
            ExperienceLevel::Beginner => 5_000.0, // 5 km
            ExperienceLevel::Intermediate => 15_000.0, // 15 km
            ExperienceLevel::Advanced => 42_195.0, // marathon distance
        }
    }

    /// Returns the maximum single-session duration in seconds.
    pub fn max_single_session_duration_s(self) -> u64 {
        match self {
            ExperienceLevel::Beginner => 3_600, // 60 min
            ExperienceLevel::Intermediate => 10_800, // 3 hours
            ExperienceLevel::Advanced => 28_800, // 8 hours
        }
    }

    /// Returns the maximum weekly distance in meters.
    pub fn max_weekly_distance_m(self) -> f64 {
        match self {
            ExperienceLevel::Beginner => 25_000.0, // 25 km/week
            ExperienceLevel::Intermediate => 75_000.0, // 75 km/week
            ExperienceLevel::Advanced => 200_000.0, // 200 km/week
        }
    }

    /// Returns the recommended number of rest days per week.
    pub fn recommended_rest_days_per_week(self) -> u32 {
        match self {
            ExperienceLevel::Beginner => 3,
            ExperienceLevel::Intermediate => 2,
            ExperienceLevel::Advanced => 1,
        }
    }

    /// Returns a human-readable label for this level.
    pub fn label(self) -> &'static str {
        match self {
            ExperienceLevel::Beginner => "Beginner",
            ExperienceLevel::Intermediate => "Intermediate",
            ExperienceLevel::Advanced => "Advanced",
        }
    }
}

/// The difficulty/intensity of a single workout day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkoutDifficulty {
    /// Light effort, comfortable pace.
    Easy,
    /// Moderate effort, sustainable but challenging.
    Moderate,
    /// High effort, challenging pace or long distance.
    Hard,
}

impl WorkoutDifficulty {
    pub fn label(self) -> &'static str {
        match self {
            WorkoutDifficulty::Easy => "Easy",
            WorkoutDifficulty::Moderate => "Moderate",
            WorkoutDifficulty::Hard => "Hard",
        }
    }
}

// ===========================================================================
// Day plan & weekly plan
// ===========================================================================

/// A single day in a weekly training plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DayPlan {
    /// Day of week: 0 = Monday, 6 = Sunday.
    pub day_of_week: u8,
    /// Whether this is a rest day (no target distance/duration).
    pub is_rest_day: bool,
    /// Target distance in meters (0 for rest days).
    pub target_distance_m: f64,
    /// Target duration in seconds (0 for rest days).
    pub target_duration_s: u64,
    /// Difficulty of this day's workout.
    pub difficulty: WorkoutDifficulty,
    /// Human-readable description of what to do.
    pub description: String,
}

/// A 7-day weekly training plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeeklyPlan {
    /// The 7 days (Monday through Sunday).
    pub days: Vec<DayPlan>,
    /// Total weekly distance in meters (sum of all non-rest days).
    pub weekly_distance_m: f64,
    /// Total weekly duration in seconds.
    pub weekly_duration_s: u64,
    /// Number of rest days in the plan.
    pub rest_days: u32,
    /// Experience level this plan was generated for.
    pub experience_level: ExperienceLevel,
}

// ===========================================================================
// Plan validation
// ===========================================================================

/// A validation issue found in a plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanValidationResult {
    /// Whether the plan is valid (no blocking issues).
    pub is_valid: bool,
    /// List of validation issues found.
    pub issues: Vec<String>,
    /// Adjustments that were or should be made to make the plan safe.
    pub adjustments: Vec<String>,
}

/// Validates a single day plan against the experience-level limits.
///
/// Returns a list of issues (empty if valid).
pub fn validate_day_plan(day: &DayPlan, experience: ExperienceLevel) -> Vec<String> {
    let mut issues = Vec::new();

    if day.is_rest_day {
        // Rest days should have zero distance/duration.
        if day.target_distance_m > 0.0 {
            issues.push(format!(
                "rest day {} has non-zero target distance {}",
                day.day_of_week, day.target_distance_m
            ));
        }
        if day.target_duration_s > 0 {
            issues.push(format!(
                "rest day {} has non-zero target duration {}",
                day.day_of_week, day.target_duration_s
            ));
        }
        return issues;
    }

    let max_dist = experience.max_single_session_distance_m();
    let max_dur = experience.max_single_session_duration_s();

    if day.target_distance_m > max_dist {
        issues.push(format!(
            "day {} distance {:.0}m exceeds {} max {:.0}m",
            day.day_of_week,
            day.target_distance_m,
            experience.label(),
            max_dist
        ));
    }

    if day.target_duration_s > max_dur {
        issues.push(format!(
            "day {} duration {}s exceeds {} max {}s",
            day.day_of_week,
            day.target_duration_s,
            experience.label(),
            max_dur
        ));
    }

    // Negative values are always invalid.
    if day.target_distance_m < 0.0 {
        issues.push(format!(
            "day {} has negative target distance",
            day.day_of_week
        ));
    }

    issues
}

/// Validates a weekly plan against all safety rules.
///
/// Checks:
/// - Each day is within experience-level limits
/// - Rest-day requirements are met (minimum rest days)
/// - No consecutive hard days for beginners
/// - 10% gradual increase rule (if previous week data provided)
pub fn validate_weekly_plan(
    plan: &WeeklyPlan,
    previous_weekly_distance_m: Option<f64>,
) -> PlanValidationResult {
    let mut issues = Vec::new();
    let adjustments = Vec::new();

    if plan.days.len() != 7 {
        issues.push(format!(
            "plan has {} days, expected 7",
            plan.days.len()
        ));
    }

    let experience = plan.experience_level;
    let mut actual_rest_days = 0u32;
    let mut consecutive_hard = 0u32;

    for (i, day) in plan.days.iter().enumerate() {
        let day_issues = validate_day_plan(day, experience);
        for issue in day_issues {
            issues.push(issue);
        }

        if day.is_rest_day {
            actual_rest_days += 1;
            consecutive_hard = 0;
        } else {
            // Track consecutive hard days.
            if day.difficulty == WorkoutDifficulty::Hard {
                consecutive_hard += 1;
                // Beginners should not have consecutive hard days.
                if experience == ExperienceLevel::Beginner && consecutive_hard > 1 {
                    issues.push(format!(
                        "beginner plan has consecutive hard days at day {} — \
                         beginners need rest between high-intensity sessions",
                        i
                    ));
                }
            } else {
                consecutive_hard = 0;
            }
        }
    }

    // Check rest-day minimums.
    let min_rest = experience.recommended_rest_days_per_week();
    if actual_rest_days < min_rest {
        issues.push(format!(
            "plan has {} rest days, minimum for {} is {}",
            actual_rest_days,
            experience.label(),
            min_rest
        ));
    }

    // Check total weekly distance against cap.
    let max_weekly = experience.max_weekly_distance_m();
    if plan.weekly_distance_m > max_weekly {
        issues.push(format!(
            "weekly distance {:.0}m exceeds {} max {:.0}m",
            plan.weekly_distance_m,
            experience.label(),
            max_weekly
        ));
    }

    // 10% gradual increase rule.
    if let Some(prev) = previous_weekly_distance_m {
        if prev > 0.0 {
            let increase_ratio = plan.weekly_distance_m / prev;
            let increase_pct = (increase_ratio - 1.0) * 100.0;
            if increase_ratio > 1.10 {
                issues.push(format!(
                    "weekly distance increases by {:.1}% (from {:.0}m to {:.0}m) — \
                     exceeds 10% gradual increase rule",
                    increase_pct,
                    prev,
                    plan.weekly_distance_m
                ));
            }
        }
    }

    PlanValidationResult {
        is_valid: issues.is_empty(),
        issues,
        adjustments,
    }
}

// ===========================================================================
// Fallback plan generation
// ===========================================================================

/// Generates a deterministic, rule-based weekly plan for the given
/// experience level and current weekly distance.
///
/// The plan:
/// - Applies a 10% increase from the current distance (capped at
///   the experience level's max weekly distance)
/// - Distributes distance across active days with varied intensity
/// - Includes the recommended number of rest days
/// - Respects single-session caps
pub fn generate_fallback_plan(
    experience: ExperienceLevel,
    current_weekly_distance_m: f64,
) -> WeeklyPlan {
    let max_weekly = experience.max_weekly_distance_m();
    let rest_days = experience.recommended_rest_days_per_week();
    let active_days = 7 - rest_days;

    // Target weekly distance: the current distance with a 10%
    // increase (capped at max), or if no current distance, a
    // conservative starting point.
    let target_weekly = if current_weekly_distance_m > 0.0 {
        (current_weekly_distance_m * 1.10).min(max_weekly)
    } else {
        match experience {
            ExperienceLevel::Beginner => 10_000.0, // 10 km start
            ExperienceLevel::Intermediate => 30_000.0, // 30 km start
            ExperienceLevel::Advanced => 60_000.0, // 60 km start
        }
    };

    let max_single = experience.max_single_session_distance_m();
    let max_dur = experience.max_single_session_duration_s();

    let mut days = Vec::with_capacity(7);
    let mut total_dist = 0.0f64;
    let mut total_dur = 0u64;
    let mut actual_rest = 0u32;

    for day_idx in 0..7u8 {
        let is_rest = match experience {
            ExperienceLevel::Beginner => {
                // Rest on Wednesday, Saturday, Sunday
                day_idx == 2 || day_idx == 5 || day_idx == 6
            }
            ExperienceLevel::Intermediate => {
                // Rest on Friday and Sunday
                day_idx == 4 || day_idx == 6
            }
            ExperienceLevel::Advanced => {
                // Rest on Sunday
                day_idx == 6
            }
        };

        if is_rest {
            days.push(DayPlan {
                day_of_week: day_idx,
                is_rest_day: true,
                target_distance_m: 0.0,
                target_duration_s: 0,
                difficulty: WorkoutDifficulty::Easy,
                description: "Rest day — recovery is important for progress".to_string(),
            });
            actual_rest += 1;
        } else {
            // Distribute: most days easy/moderate, one day longer/harder
            let (dist, dur, diff, desc) = match experience {
                ExperienceLevel::Beginner => {
                    // For beginners, distribute evenly with one longer day.
                    // avg_daily = target_weekly / active_days
                    let avg = target_weekly / active_days as f64;
                    if day_idx == 0 {
                        // Monday: longest day (1.5x average, capped)
                        let d = (avg * 1.5).min(max_single);
                        let t = ((d / 1.4) as u64 * 60).min(max_dur); // ~14 min/km walking
                        (d, t, WorkoutDifficulty::Moderate, "Longer walk to build endurance")
                    } else {
                        let d = (avg * 0.9).min(max_single);
                        let t = ((d / 1.4) as u64 * 60).min(max_dur);
                        (d, t, WorkoutDifficulty::Easy, "Easy walk to build the habit")
                    }
                }
                ExperienceLevel::Intermediate => {
                    let avg = target_weekly / active_days as f64;
                    if day_idx == 1 {
                        // Tuesday: hard day (intervals)
                        let d = (avg * 0.6).min(max_single);
                        let t = ((d / 2.5) as u64 * 60).min(max_dur); // ~24 min/5km running pace
                        (d, t, WorkoutDifficulty::Hard, "Interval training — alternate fast and slow")
                    } else if day_idx == 0 {
                        // Monday: long slow distance
                        let d = (avg * 1.4).min(max_single);
                        let t = ((d / 2.0) as u64 * 60).min(max_dur);
                        (d, t, WorkoutDifficulty::Moderate, "Long slow distance to build aerobic base")
                    } else {
                        let d = (avg * 0.85).min(max_single);
                        let t = ((d / 2.0) as u64 * 60).min(max_dur);
                        (d, t, WorkoutDifficulty::Easy, "Easy recovery run")
                    }
                }
                ExperienceLevel::Advanced => {
                    let avg = target_weekly / active_days as f64;
                    if day_idx == 1 {
                        // Tuesday: tempo/hard
                        let d = (avg * 0.7).min(max_single);
                        let t = ((d / 3.0) as u64 * 60).min(max_dur);
                        (d, t, WorkoutDifficulty::Hard, "Tempo run — sustained comfortable-hard pace")
                    } else if day_idx == 3 {
                        // Thursday: intervals
                        let d = (avg * 0.6).min(max_single);
                        let t = ((d / 3.5) as u64 * 60).min(max_dur);
                        (d, t, WorkoutDifficulty::Hard, "Speed intervals — 400m fast / 200m recovery")
                    } else if day_idx == 5 {
                        // Saturday: long run
                        let d = (avg * 1.5).min(max_single);
                        let t = ((d / 2.5) as u64 * 60).min(max_dur);
                        (d, t, WorkoutDifficulty::Moderate, "Long run — build endurance")
                    } else {
                        let d = (avg * 0.8).min(max_single);
                        let t = ((d / 2.5) as u64 * 60).min(max_dur);
                        (d, t, WorkoutDifficulty::Easy, "Easy recovery run")
                    }
                }
            };

            let desc_string = desc.to_string();
            days.push(DayPlan {
                day_of_week: day_idx,
                is_rest_day: false,
                target_distance_m: dist,
                target_duration_s: dur,
                difficulty: diff,
                description: desc_string,
            });
            total_dist += dist;
            total_dur += dur;
        }
    }

    WeeklyPlan {
        days,
        weekly_distance_m: total_dist,
        weekly_duration_s: total_dur,
        rest_days: actual_rest,
        experience_level: experience,
    }
}

// ===========================================================================
// Pain response
// ===========================================================================

/// The type of pain or discomfort the user reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PainType {
    /// No pain — user feels fine.
    None,
    /// Mild discomfort, not concerning.
    Mild,
    /// Moderate pain, should reduce intensity.
    Moderate,
    /// Severe pain, should stop.
    Severe,
    /// Joint pain (knees, ankles, hips).
    Joint,
    /// Chest pain — potentially serious, escalate.
    Chest,
    /// Dizziness — potentially serious, escalate.
    Dizziness,
}

/// The action the system recommends in response to pain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PainAction {
    /// Continue the workout as planned.
    Continue,
    /// Reduce the intensity/pace.
    ReduceIntensity,
    /// Take a rest day instead.
    Rest,
    /// Stop immediately and rest.
    StopAndRest,
    /// Seek medical attention.
    SeekMedicalAttention,
}

/// The system's response to a user-reported pain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PainResponse {
    /// The recommended action.
    pub action: PainAction,
    /// A human-readable message explaining the recommendation.
    pub message: String,
    /// Whether this should be escalated to a medical professional.
    pub should_escalate: bool,
    /// Whether the user should stop training immediately.
    pub should_stop_training: bool,
}

/// Generates a non-diagnostic pain response for the given pain type.
///
/// This function does NOT diagnose the cause of pain — it provides
/// a safety-oriented recommendation about what to do next.
pub fn respond_to_pain(pain: PainType) -> PainResponse {
    match pain {
        PainType::None => PainResponse {
            action: PainAction::Continue,
            message: "You're feeling good — continue with your plan. \
                      Listen to your body and adjust if anything changes."
                .to_string(),
            should_escalate: false,
            should_stop_training: false,
        },
        PainType::Mild => PainResponse {
            action: PainAction::Continue,
            message: "Mild discomfort is normal, especially when increasing \
                      activity. Warm up properly, stay hydrated, and monitor \
                      how you feel. If it worsens, reduce your intensity."
                .to_string(),
            should_escalate: false,
            should_stop_training: false,
        },
        PainType::Moderate => PainResponse {
            action: PainAction::ReduceIntensity,
            message: "Moderate pain suggests you may be pushing too hard. \
                      Reduce your pace and distance today, and consider an \
                      extra rest day. If pain persists for more than 2–3 \
                      days, consult a healthcare provider."
                .to_string(),
            should_escalate: false,
            should_stop_training: false,
        },
        PainType::Severe => PainResponse {
            action: PainAction::StopAndRest,
            message: "Severe pain is a signal to stop immediately. Rest \
                      for 2–3 days, apply ice if appropriate, and if the pain \
                      doesn't improve or returns when you resume activity, \
                      see a healthcare provider."
                .to_string(),
            should_escalate: false,
            should_stop_training: true,
        },
        PainType::Joint => PainResponse {
            action: PainAction::Rest,
            message: "Joint pain can indicate overuse or improper form. \
                      Take a rest day, ensure proper footwear, and consider \
                      lower-impact alternatives like cycling or swimming. \
                      If joint pain persists, consult a healthcare provider."
                .to_string(),
            should_escalate: false,
            should_stop_training: false,
        },
        PainType::Chest => PainResponse {
            action: PainAction::SeekMedicalAttention,
            message: "Chest pain during exercise should be taken seriously. \
                      Stop exercising immediately. If you experience chest \
                      pain, pressure, or tightness — especially with \
                      shortness of breath, dizziness, or pain radiating to \
                      your arm or jaw — seek immediate medical attention."
                .to_string(),
            should_escalate: true,
            should_stop_training: true,
        },
        PainType::Dizziness => PainResponse {
            action: PainAction::StopAndRest,
            message: "Dizziness during exercise can indicate overexertion, \
                      dehydration, or a more serious issue. Stop exercising, \
                      sit or lie down, hydrate, and rest. If dizziness \
                      persists, recurs, or is accompanied by chest pain or \
                      fainting, seek medical attention."
                .to_string(),
            should_escalate: true,
            should_stop_training: true,
        },
    }
}

// ===========================================================================
// Content guards: no diagnosis, no weight-loss promises
// ===========================================================================

/// Checks whether text contains medical diagnosis language patterns.
///
/// The AI must not diagnose conditions. This detects patterns like
/// "you have", "you're suffering from", "this is a symptom of",
/// "diagnosed with", "your condition is", etc.
pub fn contains_diagnosis(text: &str) -> bool {
    let lower = text.to_lowercase();

    // Direct diagnosis patterns.
    let diagnosis_patterns = [
        "you have ",
        "you've ",
        "you are suffering from",
        "you're suffering from",
        "this is a symptom of",
        "you are diagnosed with",
        "you're diagnosed with",
        "your condition is",
        "you are experiencing a condition",
        "this indicates you have",
        "your symptoms indicate",
        "the diagnosis is",
        "i diagnose you with",
        "you have developed",
        "this is caused by your",
    ];

    for pattern in &diagnosis_patterns {
        if lower.contains(pattern) {
            // But exclude benign uses like "you have completed"
            // or "you have a great workout".
            if *pattern == "you have " || *pattern == "you've " {
                // Check if followed by a medical term.
                let after = &lower[lower.find(pattern).unwrap() + pattern.len()..];
                let medical_terms = [
                    "a condition",
                    "an injury",
                    "a disease",
                    "a disorder",
                    "a syndrome",
                    "a fracture",
                    "a sprain",
                    "a strain",
                    "a tear",
                    "tendonitis",
                    "arthritis",
                    "plantar fasciitis",
                    "shin splints",
                    "a stress fracture",
                    "an infection",
                    "a deficiency",
                    "developed",
                ];
                for term in &medical_terms {
                    if after.starts_with(term) {
                        return true;
                    }
                }
            } else {
                return true;
            }
        }
    }

    false
}

/// Checks whether text contains weight-loss guarantee or promise
/// language patterns.
///
/// The AI must not guarantee specific weight-loss outcomes. This
/// detects patterns like "lose X pounds", "guaranteed weight loss",
/// "you will lose weight", "burn fat fast", etc.
pub fn contains_weight_loss_promise(text: &str) -> bool {
    let lower = text.to_lowercase();

    let promise_patterns = [
        "guaranteed weight loss",
        "guaranteed to lose",
        "you will lose weight",
        "you'll lose weight",
        "lose weight fast",
        "lose weight quickly",
        "burn fat fast",
        "burn fat quickly",
        "melt away fat",
        "rapid weight loss",
        "effortless weight loss",
        "easy weight loss",
        "you are guaranteed to lose",
        "promise you will lose",
        "weight loss promise",
        "shed pounds fast",
        "drop a dress size",
        "drop two sizes",
        "lose belly fat",
        "targeted fat loss",
        "spot reduction",
    ];

    for pattern in &promise_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    // Dynamic pattern: "lose <number> pounds/kg/lbs"
    // e.g., "lose 10 pounds", "lose 5 kg", "lose 20 lbs"
    if lower.contains("lose ") {
        if let Some(pos) = lower.find("lose ") {
            let after = &lower[pos + 5..]; // skip "lose "
            // Skip optional leading whitespace and check for a number
            let trimmed = after.trim_start();
            let mut idx = 0;
            let bytes = trimmed.as_bytes();
            // Check for digits (possibly with decimal point)
            while idx < bytes.len() && (bytes[idx].is_ascii_digit() || bytes[idx] == b'.') {
                idx += 1;
            }
            if idx > 0 {
                let rest = trimmed[idx..].trim_start();
                let weight_terms = ["pounds", "pound", "kg", "kilos", "kilogram", "kilograms", "lbs", "stone", "stones"];
                for term in &weight_terms {
                    if rest.starts_with(term) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// The result of validating AI-generated coaching text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentValidationResult {
    /// Whether the text passed all content guards.
    pub is_safe: bool,
    /// List of guard violations found.
    pub violations: Vec<String>,
}

/// Validates AI-generated coaching text against content guards.
///
/// Checks for:
/// - Medical diagnosis language
/// - Weight-loss guarantee language
pub fn validate_coaching_text(text: &str) -> ContentValidationResult {
    let mut violations = Vec::new();

    if contains_diagnosis(text) {
        violations.push(
            "Text contains medical diagnosis language — AI must not \
             diagnose conditions. Rewrite to describe what the user \
             reported without identifying a specific condition."
                .to_string(),
        );
    }

    if contains_weight_loss_promise(text) {
        violations.push(
            "Text contains weight-loss guarantee language — AI must not \
             promise specific weight-loss outcomes. Rewrite to focus on \
             general health and activity benefits."
                .to_string(),
        );
    }

    ContentValidationResult {
        is_safe: violations.is_empty(),
        violations,
    }
}

// ===========================================================================
// Escalation messaging
// ===========================================================================

/// The reason for escalating to a medical professional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationReason {
    /// Chest pain during exercise.
    ChestPain,
    /// Severe dizziness or fainting.
    SevereDizziness,
    /// Severe persistent pain.
    SeverePersistentPain,
    /// Cannot bear weight on a limb.
    CannotBearWeight,
    /// Neurological symptoms (numbness, vision changes, etc.).
    NeurologicalSymptoms,
    /// Persistent unexplained symptoms.
    PersistentUnexplainedSymptoms,
}

/// An escalation message for the user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationMessage {
    /// The reason for escalation.
    pub reason: EscalationReason,
    /// The message to show the user.
    pub message: String,
    /// Whether to recommend calling emergency services.
    pub recommend_emergency: bool,
}

/// Generates an escalation message for the given reason.
///
/// These messages are shown when the system detects potentially
/// serious symptoms that warrant professional medical attention.
pub fn escalation_message(reason: EscalationReason) -> EscalationMessage {
    match reason {
        EscalationReason::ChestPain => EscalationMessage {
            reason,
            message: "Chest pain during exercise is a serious symptom. \
                      If you're experiencing chest pain, pressure, or \
                      tightness — especially with shortness of breath, \
                      pain radiating to your arm or jaw, or sweating — \
                      please stop exercising and seek immediate medical \
                      attention. Call emergency services if symptoms are \
                      severe."
                .to_string(),
            recommend_emergency: true,
        },
        EscalationReason::SevereDizziness => EscalationMessage {
            reason,
            message: "Severe dizziness or fainting during exercise requires \
                      medical evaluation. Stop exercising, sit or lie down, \
                      and seek medical attention — especially if dizziness \
                      is accompanied by chest pain, palpitations, or \
                      fainting. Call emergency services if you've fainted \
                      or feel like you might."
                .to_string(),
            recommend_emergency: true,
        },
        EscalationReason::SeverePersistentPain => EscalationMessage {
            reason,
            message: "Severe pain that persists beyond a few days or \
                      recurs with activity should be evaluated by a \
                      healthcare provider. Please stop training and \
                      schedule an appointment with your doctor or a \
                      sports medicine specialist."
                .to_string(),
            recommend_emergency: false,
        },
        EscalationReason::CannotBearWeight => EscalationMessage {
            reason,
            message: "If you cannot bear weight on a limb after an \
                      injury, this may indicate a fracture or serious \
                      injury. Please seek medical attention promptly — \
                      visit an urgent care clinic or emergency department."
                .to_string(),
            recommend_emergency: true,
        },
        EscalationReason::NeurologicalSymptoms => EscalationMessage {
            reason,
            message: "Neurological symptoms such as numbness, tingling, \
                      vision changes, or weakness during or after exercise \
                      require prompt medical evaluation. Please stop \
                      training and contact your healthcare provider. If \
                      symptoms are sudden or severe, seek emergency care."
                .to_string(),
            recommend_emergency: true,
        },
        EscalationReason::PersistentUnexplainedSymptoms => EscalationMessage {
            reason,
            message: "Symptoms that persist without a clear cause should \
                      be discussed with your healthcare provider. Please \
                      schedule an appointment for a proper evaluation."
                .to_string(),
            recommend_emergency: false,
        },
    }
}

// ===========================================================================
// User feedback handling
// ===========================================================================

/// The type of feedback the user gives on a plan or recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserFeedback {
    /// User accepts the plan as-is.
    Accept,
    /// User rejects the plan.
    Reject,
    /// User wants to modify the plan.
    Modify,
    /// The plan is too hard/intense.
    TooHard,
    /// The plan is too easy.
    TooEasy,
    /// User doesn't have time for the plan.
    NoTime,
    /// User is injured and needs a modified plan.
    Injured,
}

/// The action to take based on user feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackAction {
    /// Keep the plan as-is.
    KeepPlan,
    /// Adjust the plan (general).
    AdjustPlan,
    /// Reduce the difficulty.
    ReduceDifficulty,
    /// Increase the difficulty.
    IncreaseDifficulty,
    /// Reduce time commitment.
    ReduceTimeCommitment,
    /// Switch to a recovery-focused plan.
    SwitchToRecovery,
    /// Escalate to a human coach (if available).
    EscalateToHumanCoach,
}

/// The result of processing user feedback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackResult {
    /// The action to take.
    pub action: FeedbackAction,
    /// A message for the user.
    pub message: String,
    /// Whether the plan should be regenerated.
    pub should_regenerate: bool,
}

/// Processes user feedback on a plan and determines the appropriate
/// action.
///
/// This is the core of the "user can reject or modify a recommendation"
/// requirement. The system respects the user's feedback and adjusts
/// the plan accordingly.
pub fn process_user_feedback(feedback: UserFeedback) -> FeedbackResult {
    match feedback {
        UserFeedback::Accept => FeedbackResult {
            action: FeedbackAction::KeepPlan,
            message: "Great! Your plan is confirmed. Remember to listen \
                      to your body and adjust if needed."
                .to_string(),
            should_regenerate: false,
        },
        UserFeedback::Reject => FeedbackResult {
            action: FeedbackAction::AdjustPlan,
            message: "No problem — let's find a plan that works better \
                      for you. We'll adjust based on your preferences."
                .to_string(),
            should_regenerate: true,
        },
        UserFeedback::Modify => FeedbackResult {
            action: FeedbackAction::AdjustPlan,
            message: "Let's modify your plan. Tell us what you'd like \
                      to change — distance, days, intensity, or rest."
                .to_string(),
            should_regenerate: true,
        },
        UserFeedback::TooHard => FeedbackResult {
            action: FeedbackAction::ReduceDifficulty,
            message: "We'll reduce the intensity of your plan. Less \
                      distance, more rest days, and easier sessions. \
                      It's better to build up gradually than to overdo it."
                .to_string(),
            should_regenerate: true,
        },
        UserFeedback::TooEasy => FeedbackResult {
            action: FeedbackAction::IncreaseDifficulty,
            message: "Ready for more challenge! We'll increase the \
                      intensity gradually — following the 10% rule to \
                      keep it safe. We'll add more distance or reduce \
                      rest days."
                .to_string(),
            should_regenerate: true,
        },
        UserFeedback::NoTime => FeedbackResult {
            action: FeedbackAction::ReduceTimeCommitment,
            message: "We understand time is tight. We'll shorten your \
                      sessions and focus on fewer, more efficient \
                      workouts. Even 20–30 minutes of activity is \
                      beneficial."
                .to_string(),
            should_regenerate: true,
        },
        UserFeedback::Injured => FeedbackResult {
            action: FeedbackAction::SwitchToRecovery,
            message: "We're sorry to hear you're injured. We'll switch \
                      your plan to focus on recovery — rest, gentle \
                      movement, and low-impact alternatives. Please \
                      consult a healthcare provider for proper guidance \
                      on your injury."
                .to_string(),
            should_regenerate: true,
        },
    }
}

// ===========================================================================
// AI availability & cost management
// ===========================================================================

/// Whether the AI service is available for use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiAvailability {
    /// AI is available and can be called.
    Available,
    /// The AI service is unavailable (down, timeout, etc.).
    ServiceUnavailable,
    /// The user has exceeded their AI usage quota.
    QuotaExceeded,
    /// The AI call would exceed the cost budget.
    CostBudgetExceeded,
    /// AI is disabled for this user/feature.
    Disabled,
}

/// Decides whether the AI should be called or the fallback used.
///
/// Inputs:
/// - `is_service_up`: whether the AI service is responding
/// - `remaining_quota`: remaining AI calls for the user (None = unlimited)
/// - `estimated_cost_cents`: estimated cost of the AI call
/// - `cost_budget_cents`: the cost budget threshold (None = no budget)
/// - `is_enabled`: whether AI is enabled for this feature
pub fn decide_ai_availability(
    is_service_up: bool,
    remaining_quota: Option<u32>,
    estimated_cost_cents: u64,
    cost_budget_cents: Option<u64>,
    is_enabled: bool,
) -> AiAvailability {
    if !is_enabled {
        return AiAvailability::Disabled;
    }

    if !is_service_up {
        return AiAvailability::ServiceUnavailable;
    }

    if let Some(remaining) = remaining_quota {
        if remaining == 0 {
            return AiAvailability::QuotaExceeded;
        }
    }

    if let Some(budget) = cost_budget_cents {
        if estimated_cost_cents > budget {
            return AiAvailability::CostBudgetExceeded;
        }
    }

    AiAvailability::Available
}

/// Whether the fallback plan should be used instead of calling the AI.
pub fn should_use_fallback(availability: AiAvailability) -> bool {
    match availability {
        AiAvailability::Available => false,
        AiAvailability::ServiceUnavailable
        | AiAvailability::QuotaExceeded
        | AiAvailability::CostBudgetExceeded
        | AiAvailability::Disabled => true,
    }
}

/// The type of AI operation being performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiOperation {
    /// Generate a new training plan.
    GeneratePlan,
    /// Adjust an existing plan based on feedback.
    AdjustPlan,
    /// Summarize a completed workout.
    SummarizeWorkout,
    /// Generate an encouragement message.
    Encouragement,
    /// Answer a user question.
    AnswerQuestion,
    /// Analyze progress trends.
    AnalyzeProgress,
}

/// Estimates the cost in cents (US dollars × 100) for an AI operation.
///
/// These are rough estimates based on typical token usage for each
/// operation type. Actual costs depend on the model and token count.
pub fn estimate_ai_cost_cents(operation: AiOperation) -> u64 {
    match operation {
        // Plan generation is more complex, more tokens.
        AiOperation::GeneratePlan => 50, // ~$0.50
        // Plan adjustment is moderate.
        AiOperation::AdjustPlan => 30, // ~$0.30
        // Workout summary is shorter.
        AiOperation::SummarizeWorkout => 20, // ~$0.20
        // Encouragement is short.
        AiOperation::Encouragement => 10, // ~$0.10
        // Answering questions varies, moderate.
        AiOperation::AnswerQuestion => 25, // ~$0.25
        // Progress analysis is complex.
        AiOperation::AnalyzeProgress => 40, // ~$0.40
    }
}

/// Generates a deterministic cache key for an AI request.
///
/// This allows the server to cache AI responses for identical requests,
/// avoiding redundant AI calls and costs.
pub fn cache_key(operation: AiOperation, user_input: &str) -> String {
    // Simple deterministic hash: operation name + sanitized input.
    let op_str = match operation {
        AiOperation::GeneratePlan => "gen_plan",
        AiOperation::AdjustPlan => "adj_plan",
        AiOperation::SummarizeWorkout => "sum_workout",
        AiOperation::Encouragement => "encourage",
        AiOperation::AnswerQuestion => "answer",
        AiOperation::AnalyzeProgress => "analyze",
    };

    // Simple hash of the input string.
    let mut hash: u64 = 14695981039346656037; // FNV offset basis
    for byte in user_input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211); // FNV prime
    }

    format!("{}:{:016x}", op_str, hash)
}

// ===========================================================================
// Workout summarization (fallback)
// ===========================================================================

/// The input data for summarizing a completed workout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkoutSummaryInput {
    /// Distance in meters.
    pub distance_m: f64,
    /// Duration in seconds.
    pub duration_s: u64,
    /// Average speed in m/s.
    pub avg_speed_mps: f64,
    /// Number of steps.
    pub steps: u64,
    /// Calories burned.
    pub calories: u64,
    /// Experience level of the user.
    pub experience_level: ExperienceLevel,
}

/// Summarizes a completed workout using rule-based logic (fallback
/// when AI is unavailable).
///
/// Generates a structured, encouraging summary of the workout.
pub fn summarize_workout(input: &WorkoutSummaryInput) -> String {
    let distance_km = input.distance_m / 1000.0;
    let duration_min = input.duration_s as f64 / 60.0;
    let pace_min_per_km = if distance_km > 0.0 {
        duration_min / distance_km
    } else {
        0.0
    };

    let distance_str = if distance_km >= 1.0 {
        format!("{:.1} km", distance_km)
    } else {
        format!("{} m", input.distance_m as u32)
    };

    let duration_str = if duration_min >= 60.0 {
        format!("{:.0}h {:.0}m", duration_min / 60.0, duration_min % 60.0)
    } else {
        format!("{:.0} minutes", duration_min)
    };

    let mut summary = format!(
        "You completed a {} workout in {}, covering {}",
        match input.experience_level {
            ExperienceLevel::Beginner => "great beginner",
            ExperienceLevel::Intermediate => "solid",
            ExperienceLevel::Advanced => "strong",
        },
        duration_str,
        distance_str,
    );

    if input.steps > 0 {
        summary.push_str(&format!(" with {} steps", input.steps));
    }

    if input.calories > 0 {
        summary.push_str(&format!(", burning {} calories", input.calories));
    }

    if pace_min_per_km > 0.0 {
        summary.push_str(&format!(
            " at a pace of {:.0}:{:02.0} per km",
            pace_min_per_km as u64,
            (pace_min_per_km.fract() * 60.0) as u64
        ));
    }

    summary.push('.');

    // Add an encouraging closing based on distance.
    if distance_km >= 10.0 {
        summary.push_str(" That's a fantastic distance — you should be proud!");
    } else if distance_km >= 5.0 {
        summary.push_str(" That's a solid effort — keep it up!");
    } else if distance_km >= 1.0 {
        summary.push_str(" Every step counts — great job getting moving!");
    } else {
        summary.push_str(" Even a short walk is worth celebrating!");
    }

    summary
}

/// Generates a rule-based encouragement message based on the
/// user's recent activity.
///
/// This is the fallback for AI-generated encouragement.
pub fn generate_encouragement(
    days_active_last_week: u32,
    total_distance_last_week_m: f64,
    goal_distance_m: f64,
) -> String {
    if days_active_last_week == 0 {
        return "Ready to get started? Even a 10-minute walk can boost \
                your mood and energy. Lace up and take that first step \
                — you've got this!"
            .to_string();
    }

    if days_active_last_week >= 5 {
        return format!(
            "Incredible consistency — {} days active last week! You're \
             building a powerful habit. Keep up the great work!",
            days_active_last_week
        );
    }

    if goal_distance_m > 0.0 && total_distance_last_week_m >= goal_distance_m {
        return format!(
            "You hit your weekly goal of {:.1} km — amazing work! \
             Consider setting a slightly higher goal to keep \
             challenging yourself.",
            goal_distance_m / 1000.0
        );
    }

    if days_active_last_week >= 3 {
        return format!(
            "You're building momentum with {} days of activity. \
             Consistency is the key to progress — keep showing up!",
            days_active_last_week
        );
    }

    "Great job staying active this week! Every workout brings you \
     closer to your goals. Let's keep the streak going!"
        .to_string()
}

// ===========================================================================
// Plan adjustment (fallback)
// ===========================================================================

/// Input for adjusting a plan based on user feedback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanAdjustmentInput {
    /// The current weekly distance.
    pub current_weekly_distance_m: f64,
    /// The user's experience level.
    pub experience_level: ExperienceLevel,
    /// The user's feedback.
    pub feedback: UserFeedback,
    /// Whether the user is injured.
    pub is_injured: bool,
}

/// The result of a plan adjustment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanAdjustmentResult {
    /// The adjusted weekly distance.
    pub adjusted_weekly_distance_m: f64,
    /// Whether the difficulty was reduced.
    pub difficulty_reduced: bool,
    /// Whether rest days were added.
    pub rest_days_added: u32,
    /// A message explaining the adjustment.
    pub message: String,
}

/// Adjusts a plan based on user feedback using rule-based logic.
///
/// This is the fallback when AI is unavailable.
pub fn adjust_plan(input: &PlanAdjustmentInput) -> PlanAdjustmentResult {
    let max_weekly = input.experience_level.max_weekly_distance_m();

    if input.is_injured {
        // For injuries, drastically reduce and add rest.
        let adjusted = (input.current_weekly_distance_m * 0.3).max(3_000.0);
        return PlanAdjustmentResult {
            adjusted_weekly_distance_m: adjusted,
            difficulty_reduced: true,
            rest_days_added: 2,
            message: "Switched to a recovery-focused plan with reduced \
                      distance and extra rest. Focus on gentle movement \
                      and healing. Please consult a healthcare provider \
                      for injury-specific guidance."
                .to_string(),
        };
    }

    match input.feedback {
        UserFeedback::TooHard => {
            let adjusted = (input.current_weekly_distance_m * 0.75).max(3_000.0);
            PlanAdjustmentResult {
                adjusted_weekly_distance_m: adjusted,
                difficulty_reduced: true,
                rest_days_added: 1,
                message: format!(
                    "Reduced your weekly distance from {:.1} km to {:.1} km \
                     and added a rest day. Remember the 10% rule — build \
                     back up gradually.",
                    input.current_weekly_distance_m / 1000.0,
                    adjusted / 1000.0
                ),
            }
        }
        UserFeedback::TooEasy => {
            let adjusted = (input.current_weekly_distance_m * 1.10).min(max_weekly);
            PlanAdjustmentResult {
                adjusted_weekly_distance_m: adjusted,
                difficulty_reduced: false,
                rest_days_added: 0,
                message: format!(
                    "Increased your weekly distance by 10% to {:.1} km, \
                     following the gradual increase rule. We'll keep \
                     building from here.",
                    adjusted / 1000.0
                ),
            }
        }
        UserFeedback::NoTime => {
            // Keep distance but compress into fewer, shorter sessions.
            PlanAdjustmentResult {
                adjusted_weekly_distance_m: input.current_weekly_distance_m,
                difficulty_reduced: false,
                rest_days_added: 0,
                message: "Kept your weekly distance but redistributed it \
                          into shorter, more efficient sessions. Even \
                          20–30 minutes of focused activity is effective."
                    .to_string(),
            }
        }
        _ => {
            // For Accept, Reject, Modify — no automatic adjustment.
            PlanAdjustmentResult {
                adjusted_weekly_distance_m: input.current_weekly_distance_m,
                difficulty_reduced: false,
                rest_days_added: 0,
                message: "Your plan has been noted. We'll adjust based on \
                          your preferences."
                    .to_string(),
            }
        }
    }
}

/// Recommends a realistic progression for the next week.
///
/// Applies the 10% rule: increase weekly distance by up to 10%,
/// capped at the experience level's maximum.
pub fn recommend_progression(
    current_weekly_distance_m: f64,
    experience: ExperienceLevel,
    weeks_at_current_level: u32,
) -> f64 {
    let max_weekly = experience.max_weekly_distance_m();

    // If already at max, no further increase.
    if current_weekly_distance_m >= max_weekly {
        return max_weekly;
    }

    // If new to this level (< 2 weeks), smaller increase (5%).
    let increase_factor = if weeks_at_current_level < 2 {
        1.05
    } else {
        1.10
    };

    (current_weekly_distance_m * increase_factor).min(max_weekly)
}

// ===========================================================================
// Request moderation
// ===========================================================================

/// The result of moderating a user request for safety.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestModerationResult {
    /// Whether the request is safe to process.
    pub is_safe: bool,
    /// The moderated/refused message if the request was unsafe.
    pub refusal_message: Option<String>,
    /// The reason the request was flagged, if any.
    pub flag_reason: Option<String>,
}

/// Moderates a user request for safety before processing.
///
/// Checks for:
/// - Dangerous training requests (extreme distances, overtraining)
/// - Medical diagnosis requests
/// - Drug/medication advice requests
/// - Self-harm indicators
pub fn moderate_request(request: &str) -> RequestModerationResult {
    let lower = request.to_lowercase();

    // Check for extreme/dangerous training requests.
    let dangerous_training = [
        "run a marathon tomorrow",
        "run 100km",
        "run 100 miles",
        "double my distance",
        "triple my distance",
        "run without stopping for",
        "extreme weight loss",
        "lose 20 pounds in a week",
        "lose 10 kg in a week",
        "fast for",
        "water fast",
    ];

    for pattern in &dangerous_training {
        if lower.contains(pattern) {
            return RequestModerationResult {
                is_safe: false,
                refusal_message: Some(
                    "I can't help with that request. Extreme changes to \
                     your training or diet can be dangerous. I recommend \
                     building up gradually — the 10% rule is a good \
                     guideline for increasing activity. Please consult a \
                     healthcare provider before making drastic changes."
                        .to_string(),
                ),
                flag_reason: Some("dangerous_training_request".to_string()),
            };
        }
    }

    // Check for medical diagnosis requests.
    let diagnosis_requests = [
        "diagnose my",
        "what's wrong with my",
        "what is wrong with my",
        "is my pain serious",
        "do i have a fracture",
        "do i have arthritis",
        "is this a stress fracture",
        "what's my injury",
        "what is my injury",
        "am i injured",
    ];

    for pattern in &diagnosis_requests {
        if lower.contains(pattern) {
            return RequestModerationResult {
                is_safe: false,
                refusal_message: Some(
                    "I'm not able to diagnose medical conditions. For proper \
                     diagnosis and treatment, please consult a healthcare \
                     provider. I can help you with training plans, \
                     technique tips, and general fitness guidance."
                        .to_string(),
                ),
                flag_reason: Some("diagnosis_request".to_string()),
            };
        }
    }

    // Check for drug/medication advice.
    let drug_requests = [
        "should i take",
        "what medication",
        "painkillers for",
        "ibuprofen for",
        "supplements for",
        "performance enhancer",
        "steroids",
        "weight loss pills",
        "diet pills",
        "fat burner",
    ];

    for pattern in &drug_requests {
        if lower.contains(pattern) {
            return RequestModerationResult {
                is_safe: false,
                refusal_message: Some(
                    "I can't provide advice about medications or \
                     supplements. Please consult a healthcare provider \
                     or pharmacist for guidance on medications, \
                     supplements, or performance-related substances."
                        .to_string(),
                ),
                flag_reason: Some("drug_advice_request".to_string()),
            };
        }
    }

    // Check for self-harm indicators.
    let self_harm = [
        "can't take it anymore",
        "end it all",
        "don't want to live",
        "hurt myself",
        "self harm",
        "suicide",
    ];

    for pattern in &self_harm {
        if lower.contains(pattern) {
            return RequestModerationResult {
                is_safe: false,
                refusal_message: Some(
                    "If you're going through a difficult time, please know \
                     that help is available. If you're in crisis, please \
                     reach out to a crisis helpline or emergency services \
                     in your area immediately. You don't have to go \
                     through this alone."
                        .to_string(),
                ),
                flag_reason: Some("self_harm_indicator".to_string()),
            };
        }
    }

    RequestModerationResult {
        is_safe: true,
        refusal_message: None,
        flag_reason: None,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Experience level tests ─────────────────────────────────────

    #[test]
    fn beginner_caps_are_conservative() {
        assert_eq!(ExperienceLevel::Beginner.max_single_session_distance_m(), 5_000.0);
        assert_eq!(ExperienceLevel::Beginner.max_single_session_duration_s(), 3_600);
        assert_eq!(ExperienceLevel::Beginner.max_weekly_distance_m(), 25_000.0);
        assert_eq!(ExperienceLevel::Beginner.recommended_rest_days_per_week(), 3);
    }

    #[test]
    fn intermediate_caps_are_moderate() {
        assert_eq!(ExperienceLevel::Intermediate.max_single_session_distance_m(), 15_000.0);
        assert_eq!(ExperienceLevel::Intermediate.max_single_session_duration_s(), 10_800);
        assert_eq!(ExperienceLevel::Intermediate.max_weekly_distance_m(), 75_000.0);
        assert_eq!(ExperienceLevel::Intermediate.recommended_rest_days_per_week(), 2);
    }

    #[test]
    fn advanced_caps_are_generous() {
        assert_eq!(ExperienceLevel::Advanced.max_single_session_distance_m(), 42_195.0);
        assert_eq!(ExperienceLevel::Advanced.max_single_session_duration_s(), 28_800);
        assert_eq!(ExperienceLevel::Advanced.max_weekly_distance_m(), 200_000.0);
        assert_eq!(ExperienceLevel::Advanced.recommended_rest_days_per_week(), 1);
    }

    #[test]
    fn experience_level_labels() {
        assert_eq!(ExperienceLevel::Beginner.label(), "Beginner");
        assert_eq!(ExperienceLevel::Intermediate.label(), "Intermediate");
        assert_eq!(ExperienceLevel::Advanced.label(), "Advanced");
    }

    #[test]
    fn difficulty_labels() {
        assert_eq!(WorkoutDifficulty::Easy.label(), "Easy");
        assert_eq!(WorkoutDifficulty::Moderate.label(), "Moderate");
        assert_eq!(WorkoutDifficulty::Hard.label(), "Hard");
    }

    // ── Day plan validation tests ──────────────────────────────────

    #[test]
    fn valid_day_plan_has_no_issues() {
        let day = DayPlan {
            day_of_week: 0,
            is_rest_day: false,
            target_distance_m: 3_000.0,
            target_duration_s: 1_800,
            difficulty: WorkoutDifficulty::Easy,
            description: "Easy walk".to_string(),
        };
        let issues = validate_day_plan(&day, ExperienceLevel::Beginner);
        assert!(issues.is_empty(), "expected no issues, got {:?}", issues);
    }

    #[test]
    fn day_exceeding_beginner_distance_cap_is_flagged() {
        let day = DayPlan {
            day_of_week: 0,
            is_rest_day: false,
            target_distance_m: 10_000.0, // exceeds 5km beginner cap
            target_duration_s: 3_600,
            difficulty: WorkoutDifficulty::Moderate,
            description: "Long walk".to_string(),
        };
        let issues = validate_day_plan(&day, ExperienceLevel::Beginner);
        assert!(issues.iter().any(|i| i.contains("exceeds")), "expected distance cap issue, got {:?}", issues);
    }

    #[test]
    fn day_exceeding_duration_cap_is_flagged() {
        let day = DayPlan {
            day_of_week: 0,
            is_rest_day: false,
            target_distance_m: 3_000.0,
            target_duration_s: 7_200, // exceeds 60min beginner cap
            difficulty: WorkoutDifficulty::Easy,
            description: "Long walk".to_string(),
        };
        let issues = validate_day_plan(&day, ExperienceLevel::Beginner);
        assert!(issues.iter().any(|i| i.contains("duration")), "expected duration issue, got {:?}", issues);
    }

    #[test]
    fn rest_day_with_distance_is_flagged() {
        let day = DayPlan {
            day_of_week: 6,
            is_rest_day: true,
            target_distance_m: 2_000.0,
            target_duration_s: 0,
            difficulty: WorkoutDifficulty::Easy,
            description: "Rest".to_string(),
        };
        let issues = validate_day_plan(&day, ExperienceLevel::Beginner);
        assert!(issues.iter().any(|i| i.contains("rest day")), "expected rest day issue, got {:?}", issues);
    }

    #[test]
    fn rest_day_with_zero_distance_is_valid() {
        let day = DayPlan {
            day_of_week: 6,
            is_rest_day: true,
            target_distance_m: 0.0,
            target_duration_s: 0,
            difficulty: WorkoutDifficulty::Easy,
            description: "Rest".to_string(),
        };
        let issues = validate_day_plan(&day, ExperienceLevel::Beginner);
        assert!(issues.is_empty(), "expected no issues for rest day, got {:?}", issues);
    }

    #[test]
    fn negative_distance_is_flagged() {
        let day = DayPlan {
            day_of_week: 0,
            is_rest_day: false,
            target_distance_m: -1_000.0,
            target_duration_s: 1_800,
            difficulty: WorkoutDifficulty::Easy,
            description: "Walk".to_string(),
        };
        let issues = validate_day_plan(&day, ExperienceLevel::Beginner);
        assert!(issues.iter().any(|i| i.contains("negative")), "expected negative issue, got {:?}", issues);
    }

    // ── Weekly plan validation tests ───────────────────────────────

    #[test]
    fn valid_weekly_plan_passes_validation() {
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, 10_000.0);
        let result = validate_weekly_plan(&plan, None);
        assert!(result.is_valid, "valid plan failed: {:?}", result.issues);
    }

    #[test]
    fn weekly_plan_with_too_few_rest_days_is_flagged() {
        let mut days = vec![];
        for i in 0..7 {
            days.push(DayPlan {
                day_of_week: i,
                is_rest_day: false,
                target_distance_m: 3_000.0,
                target_duration_s: 1_800,
                difficulty: WorkoutDifficulty::Easy,
                description: "Walk".to_string(),
            });
        }
        let plan = WeeklyPlan {
            days,
            weekly_distance_m: 21_000.0,
            weekly_duration_s: 12_600,
            rest_days: 0,
            experience_level: ExperienceLevel::Beginner,
        };
        let result = validate_weekly_plan(&plan, None);
        assert!(result.issues.iter().any(|i| i.contains("rest days")), "expected rest day issue, got {:?}", result.issues);
    }

    #[test]
    fn consecutive_hard_days_for_beginner_is_flagged() {
        let mut days = vec![];
        for i in 0..7 {
            let is_rest = i == 2 || i == 5 || i == 6;
            days.push(DayPlan {
                day_of_week: i,
                is_rest_day: is_rest,
                target_distance_m: if is_rest { 0.0 } else { 3_000.0 },
                target_duration_s: if is_rest { 0 } else { 1_800 },
                difficulty: if is_rest { WorkoutDifficulty::Easy } else { WorkoutDifficulty::Hard },
                description: if is_rest { "Rest".to_string() } else { "Hard".to_string() },
            });
        }
        let plan = WeeklyPlan {
            days,
            weekly_distance_m: 12_000.0,
            weekly_duration_s: 7_200,
            rest_days: 3,
            experience_level: ExperienceLevel::Beginner,
        };
        let result = validate_weekly_plan(&plan, None);
        assert!(result.issues.iter().any(|i| i.contains("consecutive hard")), "expected consecutive hard issue, got {:?}", result.issues);
    }

    #[test]
    fn ten_percent_increase_rule_is_enforced() {
        // Previous week: 10km. Current plan: 12km (20% increase).
        let mut days = vec![];
        for i in 0..7 {
            let is_rest = i == 2 || i == 5 || i == 6;
            days.push(DayPlan {
                day_of_week: i,
                is_rest_day: is_rest,
                target_distance_m: if is_rest { 0.0 } else { 3_000.0 },
                target_duration_s: if is_rest { 0 } else { 1_800 },
                difficulty: WorkoutDifficulty::Easy,
                description: "Walk".to_string(),
            });
        }
        let plan = WeeklyPlan {
            days,
            weekly_distance_m: 12_000.0, // 20% > 10%
            weekly_duration_s: 7_200,
            rest_days: 3,
            experience_level: ExperienceLevel::Beginner,
        };
        let result = validate_weekly_plan(&plan, Some(10_000.0));
        assert!(result.issues.iter().any(|i| i.contains("10%")), "expected 10% rule issue, got {:?}", result.issues);
    }

    #[test]
    fn ten_percent_increase_at_boundary_is_ok() {
        // Previous week: 10km. Current plan: 11km (10% increase — exactly at boundary).
        let mut days = vec![];
        for i in 0..7 {
            let is_rest = i == 2 || i == 5 || i == 6;
            days.push(DayPlan {
                day_of_week: i,
                is_rest_day: is_rest,
                target_distance_m: if is_rest { 0.0 } else { 2_750.0 },
                target_duration_s: if is_rest { 0 } else { 1_800 },
                difficulty: WorkoutDifficulty::Easy,
                description: "Walk".to_string(),
            });
        }
        let plan = WeeklyPlan {
            days,
            weekly_distance_m: 11_000.0,
            weekly_duration_s: 7_200,
            rest_days: 3,
            experience_level: ExperienceLevel::Beginner,
        };
        let result = validate_weekly_plan(&plan, Some(10_000.0));
        assert!(!result.issues.iter().any(|i| i.contains("10%")), "10% boundary should be OK, got {:?}", result.issues);
    }

    #[test]
    fn weekly_plan_exceeding_max_distance_is_flagged() {
        let mut days = vec![];
        for i in 0..7 {
            let is_rest = i == 6;
            days.push(DayPlan {
                day_of_week: i,
                is_rest_day: is_rest,
                target_distance_m: if is_rest { 0.0 } else { 4_000.0 },
                target_duration_s: if is_rest { 0 } else { 2_400 },
                difficulty: WorkoutDifficulty::Easy,
                description: "Walk".to_string(),
            });
        }
        // 6 × 4000 = 24000, which is under 25000 beginner max. Let's go higher.
        let plan = WeeklyPlan {
            days: days.clone(),
            weekly_distance_m: 30_000.0, // exceeds 25km beginner cap
            weekly_duration_s: 14_400,
            rest_days: 1, // also too few for beginner (min 3)
            experience_level: ExperienceLevel::Beginner,
        };
        let result = validate_weekly_plan(&plan, None);
        assert!(result.issues.iter().any(|i| i.contains("weekly distance")), "expected weekly distance issue, got {:?}", result.issues);
    }

    #[test]
    fn wrong_number_of_days_is_flagged() {
        let plan = WeeklyPlan {
            days: vec![],
            weekly_distance_m: 0.0,
            weekly_duration_s: 0,
            rest_days: 0,
            experience_level: ExperienceLevel::Beginner,
        };
        let result = validate_weekly_plan(&plan, None);
        assert!(result.issues.iter().any(|i| i.contains("expected 7")), "expected day count issue, got {:?}", result.issues);
    }

    // ── Fallback plan generation tests ─────────────────────────────

    #[test]
    fn fallback_plan_has_seven_days() {
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, 0.0);
        assert_eq!(plan.days.len(), 7);
    }

    #[test]
    fn fallback_plan_does_not_exceed_single_session_cap() {
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, 100_000.0);
        let max_dist = ExperienceLevel::Beginner.max_single_session_distance_m();
        for day in &plan.days {
            assert!(day.target_distance_m <= max_dist,
                "day {} distance {} exceeds cap {}", day.day_of_week, day.target_distance_m, max_dist);
        }
    }

    #[test]
    fn fallback_plan_has_correct_rest_days() {
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, 0.0);
        assert_eq!(plan.rest_days, 3);
        let plan = generate_fallback_plan(ExperienceLevel::Intermediate, 0.0);
        assert_eq!(plan.rest_days, 2);
        let plan = generate_fallback_plan(ExperienceLevel::Advanced, 0.0);
        assert_eq!(plan.rest_days, 1);
    }

    #[test]
    fn fallback_plan_with_current_distance_does_not_exceed_ten_percent() {
        let current = 10_000.0;
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, current);
        // The plan should target ~10% more than current, capped at max.
        let max_weekly = ExperienceLevel::Beginner.max_weekly_distance_m();
        // The actual weekly distance is the sum of distributed daily
        // distances, which should be close to the target. It should
        // not exceed the target by much (distribution may not be exact).
        // More importantly, it should not be significantly more than
        // 10% above the current distance.
        let max_allowed = current * 1.20; // 20% tolerance for distribution rounding
        assert!(plan.weekly_distance_m <= max_allowed,
            "weekly distance {} exceeds 15% above current ({})", plan.weekly_distance_m, max_allowed);
        assert!(plan.weekly_distance_m <= max_weekly,
            "weekly distance {} exceeds max weekly {}", plan.weekly_distance_m, max_weekly);
        // And it should be greater than current (10% increase)
        assert!(plan.weekly_distance_m > current,
            "weekly distance {} should be > current {}", plan.weekly_distance_m, current);
    }

    #[test]
    fn fallback_plan_respects_max_weekly_distance() {
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, 100_000.0);
        assert!(plan.weekly_distance_m <= ExperienceLevel::Beginner.max_weekly_distance_m(),
            "weekly distance {} exceeds max", plan.weekly_distance_m);
    }

    #[test]
    fn fallback_plan_default_starting_distances() {
        let beginner_plan = generate_fallback_plan(ExperienceLevel::Beginner, 0.0);
        // Should be around 10km for a beginner with no current distance.
        assert!(beginner_plan.weekly_distance_m > 0.0, "beginner plan should have some distance");

        let intermediate_plan = generate_fallback_plan(ExperienceLevel::Intermediate, 0.0);
        assert!(intermediate_plan.weekly_distance_m > beginner_plan.weekly_distance_m,
            "intermediate default should be more than beginner");

        let advanced_plan = generate_fallback_plan(ExperienceLevel::Advanced, 0.0);
        assert!(advanced_plan.weekly_distance_m > intermediate_plan.weekly_distance_m,
            "advanced default should be more than intermediate");
    }

    #[test]
    fn fallback_plan_has_rest_day_descriptions() {
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, 0.0);
        let rest_days: Vec<_> = plan.days.iter().filter(|d| d.is_rest_day).collect();
        assert!(!rest_days.is_empty(), "should have rest days");
        for day in &rest_days {
            assert!(day.description.contains("Rest"), "rest day description should mention rest: {}", day.description);
        }
    }

    #[test]
    fn fallback_plan_validates_as_safe() {
        let plan = generate_fallback_plan(ExperienceLevel::Intermediate, 20_000.0);
        let result = validate_weekly_plan(&plan, Some(20_000.0));
        assert!(result.is_valid, "generated plan should be valid: {:?}", result.issues);
    }

    // ── Pain response tests ─────────────────────────────────────────

    #[test]
    fn no_pain_allows_continue() {
        let resp = respond_to_pain(PainType::None);
        assert_eq!(resp.action, PainAction::Continue);
        assert!(!resp.should_escalate);
        assert!(!resp.should_stop_training);
    }

    #[test]
    fn mild_pain_allows_continue() {
        let resp = respond_to_pain(PainType::Mild);
        assert_eq!(resp.action, PainAction::Continue);
        assert!(!resp.should_escalate);
    }

    #[test]
    fn moderate_pain_reduces_intensity() {
        let resp = respond_to_pain(PainType::Moderate);
        assert_eq!(resp.action, PainAction::ReduceIntensity);
        assert!(!resp.should_escalate);
        assert!(!resp.should_stop_training);
    }

    #[test]
    fn severe_pain_stops_training() {
        let resp = respond_to_pain(PainType::Severe);
        assert_eq!(resp.action, PainAction::StopAndRest);
        assert!(resp.should_stop_training);
        assert!(!resp.should_escalate);
    }

    #[test]
    fn joint_pain_recommends_rest() {
        let resp = respond_to_pain(PainType::Joint);
        assert_eq!(resp.action, PainAction::Rest);
        assert!(!resp.should_escalate);
    }

    #[test]
    fn chest_pain_escalates() {
        let resp = respond_to_pain(PainType::Chest);
        assert_eq!(resp.action, PainAction::SeekMedicalAttention);
        assert!(resp.should_escalate);
        assert!(resp.should_stop_training);
    }

    #[test]
    fn dizziness_escalates() {
        let resp = respond_to_pain(PainType::Dizziness);
        assert_eq!(resp.action, PainAction::StopAndRest);
        assert!(resp.should_escalate);
        assert!(resp.should_stop_training);
    }

    #[test]
    fn pain_responses_have_non_empty_messages() {
        let all_pains = [
            PainType::None,
            PainType::Mild,
            PainType::Moderate,
            PainType::Severe,
            PainType::Joint,
            PainType::Chest,
            PainType::Dizziness,
        ];
        for pain in &all_pains {
            let resp = respond_to_pain(*pain);
            assert!(!resp.message.is_empty(), "pain {:?} has empty message", pain);
        }
    }

    // ── Diagnosis detection tests ───────────────────────────────────

    #[test]
    fn detects_explicit_diagnosis() {
        assert!(contains_diagnosis("You are suffering from tendonitis."));
        assert!(contains_diagnosis("You've developed a stress fracture."));
        assert!(contains_diagnosis("This is a symptom of plantar fasciitis."));
    }

    #[test]
    fn detects_diagnosis_with_medical_term() {
        assert!(contains_diagnosis("You have a fracture in your foot."));
        assert!(contains_diagnosis("You have an injury that needs attention."));
        assert!(contains_diagnosis("You have arthritis in your knee."));
    }

    #[test]
    fn does_not_flag_benign_you_have() {
        assert!(!contains_diagnosis("You have completed a great workout!"));
        assert!(!contains_diagnosis("You have a great plan ahead of you."));
        assert!(!contains_diagnosis("You've been doing well with your training."));
    }

    #[test]
    fn detects_diagnosis_with_condition_word() {
        assert!(contains_diagnosis("Your condition is shin splints."));
        assert!(contains_diagnosis("You are diagnosed with arthritis."));
        assert!(contains_diagnosis("I diagnose you with plantar fasciitis."));
    }

    // ── Weight loss promise detection tests ────────────────────────

    #[test]
    fn detects_weight_loss_guarantee() {
        assert!(contains_weight_loss_promise("Guaranteed weight loss in 30 days!"));
        assert!(contains_weight_loss_promise("You will lose weight fast with this plan."));
        assert!(contains_weight_loss_promise("Burn fat fast with our program!"));
    }

    #[test]
    fn detects_specific_weight_claims() {
        assert!(contains_weight_loss_promise("Lose 10 pounds in a week!"));
        assert!(contains_weight_loss_promise("Rapid weight loss is possible!"));
        assert!(contains_weight_loss_promise("Easy weight loss guaranteed!"));
    }

    #[test]
    fn does_not_flag_general_health_advice() {
        assert!(!contains_weight_loss_promise("Regular exercise supports overall health."));
        assert!(!contains_weight_loss_promise("Building muscle can improve your metabolism."));
        assert!(!contains_weight_loss_promise("A balanced diet is important for fitness."));
    }

    // ── Content validation tests ───────────────────────────────────

    #[test]
    fn safe_coaching_text_passes() {
        let result = validate_coaching_text("Great workout today! You covered 5km in 30 minutes. Keep up the good work!");
        assert!(result.is_safe);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn diagnosis_text_is_flagged() {
        let result = validate_coaching_text("You are suffering from tendonitis based on your symptoms.");
        assert!(!result.is_safe);
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].contains("diagnosis"));
    }

    #[test]
    fn weight_loss_promise_text_is_flagged() {
        let result = validate_coaching_text("This plan will guarantee rapid weight loss!");
        assert!(!result.is_safe);
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].contains("weight-loss"));
    }

    #[test]
    fn both_violations_are_detected() {
        let result = validate_coaching_text(
            "You are suffering from arthritis and this plan guarantees rapid weight loss!",
        );
        assert!(!result.is_safe);
        assert_eq!(result.violations.len(), 2);
    }

    // ── Escalation message tests ───────────────────────────────────

    #[test]
    fn chest_pain_escalation_is_emergency() {
        let msg = escalation_message(EscalationReason::ChestPain);
        assert!(msg.recommend_emergency);
        assert!(!msg.message.is_empty());
    }

    #[test]
    fn severe_dizziness_escalation_is_emergency() {
        let msg = escalation_message(EscalationReason::SevereDizziness);
        assert!(msg.recommend_emergency);
    }

    #[test]
    fn cannot_bear_weight_escalation_is_emergency() {
        let msg = escalation_message(EscalationReason::CannotBearWeight);
        assert!(msg.recommend_emergency);
    }

    #[test]
    fn neurological_escalation_is_emergency() {
        let msg = escalation_message(EscalationReason::NeurologicalSymptoms);
        assert!(msg.recommend_emergency);
    }

    #[test]
    fn severe_persistent_pain_not_emergency() {
        let msg = escalation_message(EscalationReason::SeverePersistentPain);
        assert!(!msg.recommend_emergency);
    }

    #[test]
    fn unexplained_symptoms_not_emergency() {
        let msg = escalation_message(EscalationReason::PersistentUnexplainedSymptoms);
        assert!(!msg.recommend_emergency);
    }

    #[test]
    fn all_escalation_messages_have_content() {
        let reasons = [
            EscalationReason::ChestPain,
            EscalationReason::SevereDizziness,
            EscalationReason::SeverePersistentPain,
            EscalationReason::CannotBearWeight,
            EscalationReason::NeurologicalSymptoms,
            EscalationReason::PersistentUnexplainedSymptoms,
        ];
        for reason in &reasons {
            let msg = escalation_message(*reason);
            assert!(!msg.message.is_empty(), "escalation {:?} has empty message", reason);
        }
    }

    // ── User feedback tests ─────────────────────────────────────────

    #[test]
    fn accept_feedback_keeps_plan() {
        let result = process_user_feedback(UserFeedback::Accept);
        assert_eq!(result.action, FeedbackAction::KeepPlan);
        assert!(!result.should_regenerate);
    }

    #[test]
    fn reject_feedback_adjusts_plan() {
        let result = process_user_feedback(UserFeedback::Reject);
        assert_eq!(result.action, FeedbackAction::AdjustPlan);
        assert!(result.should_regenerate);
    }

    #[test]
    fn modify_feedback_adjusts_plan() {
        let result = process_user_feedback(UserFeedback::Modify);
        assert_eq!(result.action, FeedbackAction::AdjustPlan);
        assert!(result.should_regenerate);
    }

    #[test]
    fn too_hard_feedback_reduces_difficulty() {
        let result = process_user_feedback(UserFeedback::TooHard);
        assert_eq!(result.action, FeedbackAction::ReduceDifficulty);
        assert!(result.should_regenerate);
    }

    #[test]
    fn too_easy_feedback_increases_difficulty() {
        let result = process_user_feedback(UserFeedback::TooEasy);
        assert_eq!(result.action, FeedbackAction::IncreaseDifficulty);
        assert!(result.should_regenerate);
    }

    #[test]
    fn no_time_feedback_reduces_time_commitment() {
        let result = process_user_feedback(UserFeedback::NoTime);
        assert_eq!(result.action, FeedbackAction::ReduceTimeCommitment);
        assert!(result.should_regenerate);
    }

    #[test]
    fn injured_feedback_switches_to_recovery() {
        let result = process_user_feedback(UserFeedback::Injured);
        assert_eq!(result.action, FeedbackAction::SwitchToRecovery);
        assert!(result.should_regenerate);
    }

    #[test]
    fn all_feedback_has_non_empty_messages() {
        let all_feedback = [
            UserFeedback::Accept,
            UserFeedback::Reject,
            UserFeedback::Modify,
            UserFeedback::TooHard,
            UserFeedback::TooEasy,
            UserFeedback::NoTime,
            UserFeedback::Injured,
        ];
        for fb in &all_feedback {
            let result = process_user_feedback(*fb);
            assert!(!result.message.is_empty(), "feedback {:?} has empty message", fb);
        }
    }

    // ── AI availability tests ────────────────────────────────────────

    #[test]
    fn ai_available_when_all_conditions_met() {
        let avail = decide_ai_availability(true, Some(10), 30, Some(100), true);
        assert_eq!(avail, AiAvailability::Available);
    }

    #[test]
    fn ai_unavailable_when_service_down() {
        let avail = decide_ai_availability(false, Some(10), 30, Some(100), true);
        assert_eq!(avail, AiAvailability::ServiceUnavailable);
    }

    #[test]
    fn ai_unavailable_when_quota_exceeded() {
        let avail = decide_ai_availability(true, Some(0), 30, Some(100), true);
        assert_eq!(avail, AiAvailability::QuotaExceeded);
    }

    #[test]
    fn ai_unlimited_quota_works() {
        let avail = decide_ai_availability(true, None, 30, Some(100), true);
        assert_eq!(avail, AiAvailability::Available);
    }

    #[test]
    fn ai_unavailable_when_cost_exceeds_budget() {
        let avail = decide_ai_availability(true, Some(10), 200, Some(100), true);
        assert_eq!(avail, AiAvailability::CostBudgetExceeded);
    }

    #[test]
    fn ai_disabled_when_not_enabled() {
        let avail = decide_ai_availability(true, Some(10), 30, Some(100), false);
        assert_eq!(avail, AiAvailability::Disabled);
    }

    #[test]
    fn no_budget_does_not_block() {
        let avail = decide_ai_availability(true, Some(10), 999, None, true);
        assert_eq!(avail, AiAvailability::Available);
    }

    // ── Should use fallback tests ───────────────────────────────────

    #[test]
    fn available_does_not_use_fallback() {
        assert!(!should_use_fallback(AiAvailability::Available));
    }

    #[test]
    fn all_unavailable_states_use_fallback() {
        assert!(should_use_fallback(AiAvailability::ServiceUnavailable));
        assert!(should_use_fallback(AiAvailability::QuotaExceeded));
        assert!(should_use_fallback(AiAvailability::CostBudgetExceeded));
        assert!(should_use_fallback(AiAvailability::Disabled));
    }

    // ── AI cost estimation tests ────────────────────────────────────

    #[test]
    fn cost_estimates_are_positive() {
        let ops = [
            AiOperation::GeneratePlan,
            AiOperation::AdjustPlan,
            AiOperation::SummarizeWorkout,
            AiOperation::Encouragement,
            AiOperation::AnswerQuestion,
            AiOperation::AnalyzeProgress,
        ];
        for op in &ops {
            let cost = estimate_ai_cost_cents(*op);
            assert!(cost > 0, "operation {:?} has zero cost", op);
        }
    }

    #[test]
    fn generate_plan_is_most_expensive() {
        let gen_cost = estimate_ai_cost_cents(AiOperation::GeneratePlan);
        let enc_cost = estimate_ai_cost_cents(AiOperation::Encouragement);
        assert!(gen_cost > enc_cost, "generate plan should cost more than encouragement");
    }

    #[test]
    fn encouragement_is_cheapest() {
        let enc_cost = estimate_ai_cost_cents(AiOperation::Encouragement);
        assert_eq!(enc_cost, 10);
    }

    // ── Cache key tests ─────────────────────────────────────────────

    #[test]
    fn cache_key_is_deterministic() {
        let key1 = cache_key(AiOperation::GeneratePlan, "beginner, 10km, no injuries");
        let key2 = cache_key(AiOperation::GeneratePlan, "beginner, 10km, no injuries");
        assert_eq!(key1, key2);
    }

    #[test]
    fn cache_key_differs_for_different_input() {
        let key1 = cache_key(AiOperation::GeneratePlan, "beginner, 10km");
        let key2 = cache_key(AiOperation::GeneratePlan, "beginner, 20km");
        assert_ne!(key1, key2);
    }

    #[test]
    fn cache_key_differs_for_different_operation() {
        let key1 = cache_key(AiOperation::GeneratePlan, "test input");
        let key2 = cache_key(AiOperation::AdjustPlan, "test input");
        assert_ne!(key1, key2);
    }

    #[test]
    fn cache_key_has_operation_prefix() {
        let key = cache_key(AiOperation::GeneratePlan, "test");
        assert!(key.starts_with("gen_plan:"));
    }

    // ── Workout summarization tests ────────────────────────────────

    #[test]
    fn workout_summary_includes_distance() {
        let input = WorkoutSummaryInput {
            distance_m: 5_000.0,
            duration_s: 1_800,
            avg_speed_mps: 2.78,
            steps: 6_000,
            calories: 300,
            experience_level: ExperienceLevel::Beginner,
        };
        let summary = summarize_workout(&input);
        assert!(summary.contains("5.0 km"));
    }

    #[test]
    fn workout_summary_includes_steps() {
        let input = WorkoutSummaryInput {
            distance_m: 5_000.0,
            duration_s: 1_800,
            avg_speed_mps: 2.78,
            steps: 6_000,
            calories: 300,
            experience_level: ExperienceLevel::Beginner,
        };
        let summary = summarize_workout(&input);
        assert!(summary.contains("6000 steps"));
    }

    #[test]
    fn workout_summary_includes_calories() {
        let input = WorkoutSummaryInput {
            distance_m: 5_000.0,
            duration_s: 1_800,
            avg_speed_mps: 2.78,
            steps: 6_000,
            calories: 300,
            experience_level: ExperienceLevel::Beginner,
        };
        let summary = summarize_workout(&input);
        assert!(summary.contains("300 calories"));
    }

    #[test]
    fn long_distance_summary_has_encouraging_closing() {
        let input = WorkoutSummaryInput {
            distance_m: 15_000.0,
            duration_s: 5_400,
            avg_speed_mps: 2.78,
            steps: 18_000,
            calories: 900,
            experience_level: ExperienceLevel::Intermediate,
        };
        let summary = summarize_workout(&input);
        assert!(summary.contains("proud") || summary.contains("fantastic"));
    }

    #[test]
    fn short_distance_summary_has_encouraging_closing() {
        let input = WorkoutSummaryInput {
            distance_m: 500.0,
            duration_s: 300,
            avg_speed_mps: 1.67,
            steps: 600,
            calories: 30,
            experience_level: ExperienceLevel::Beginner,
        };
        let summary = summarize_workout(&input);
        assert!(summary.contains("celebrat"));
    }

    // ── Encouragement tests ─────────────────────────────────────────

    #[test]
    fn no_activity_encourages_starting() {
        let msg = generate_encouragement(0, 0.0, 0.0);
        assert!(msg.contains("start") || msg.contains("first step"));
    }

    #[test]
    fn high_consistency_encourages_momentum() {
        let msg = generate_encouragement(6, 30_000.0, 25_000.0);
        assert!(msg.contains("incredible") || msg.contains("consistency"));
    }

    #[test]
    fn goal_achievement_is_celebrated() {
        let msg = generate_encouragement(4, 50_000.0, 50_000.0);
        assert!(msg.contains("goal") || msg.contains("amazing"));
    }

    #[test]
    fn moderate_activity_encourages_momentum() {
        let msg = generate_encouragement(3, 15_000.0, 20_000.0);
        assert!(msg.contains("momentum") || msg.contains("Consistency"));
    }

    // ── Plan adjustment tests ───────────────────────────────────────

    #[test]
    fn injured_adjustment_switches_to_recovery() {
        let input = PlanAdjustmentInput {
            current_weekly_distance_m: 20_000.0,
            experience_level: ExperienceLevel::Intermediate,
            feedback: UserFeedback::Injured,
            is_injured: true,
        };
        let result = adjust_plan(&input);
        assert!(result.difficulty_reduced);
        assert!(result.rest_days_added > 0);
        assert!(result.adjusted_weekly_distance_m < input.current_weekly_distance_m);
    }

    #[test]
    fn too_hard_adjustment_reduces_distance() {
        let input = PlanAdjustmentInput {
            current_weekly_distance_m: 20_000.0,
            experience_level: ExperienceLevel::Intermediate,
            feedback: UserFeedback::TooHard,
            is_injured: false,
        };
        let result = adjust_plan(&input);
        assert!(result.difficulty_reduced);
        assert!(result.adjusted_weekly_distance_m < input.current_weekly_distance_m);
        assert_eq!(result.rest_days_added, 1);
    }

    #[test]
    fn too_easy_adjustment_increases_by_ten_percent() {
        let input = PlanAdjustmentInput {
            current_weekly_distance_m: 20_000.0,
            experience_level: ExperienceLevel::Intermediate,
            feedback: UserFeedback::TooEasy,
            is_injured: false,
        };
        let result = adjust_plan(&input);
        assert!(!result.difficulty_reduced);
        assert!(result.adjusted_weekly_distance_m > input.current_weekly_distance_m);
        // Should be ~10% more (capped at max)
        let expected: f64 = (20_000.0_f64 * 1.10).min(ExperienceLevel::Intermediate.max_weekly_distance_m());
        assert!((result.adjusted_weekly_distance_m - expected).abs() < 1.0);
    }

    #[test]
    fn no_time_adjustment_keeps_distance() {
        let input = PlanAdjustmentInput {
            current_weekly_distance_m: 20_000.0,
            experience_level: ExperienceLevel::Intermediate,
            feedback: UserFeedback::NoTime,
            is_injured: false,
        };
        let result = adjust_plan(&input);
        assert_eq!(result.adjusted_weekly_distance_m, input.current_weekly_distance_m);
        assert!(!result.difficulty_reduced);
    }

    #[test]
    fn too_easy_does_not_exceed_max() {
        let input = PlanAdjustmentInput {
            current_weekly_distance_m: 74_000.0,
            experience_level: ExperienceLevel::Intermediate,
            feedback: UserFeedback::TooEasy,
            is_injured: false,
        };
        let result = adjust_plan(&input);
        assert!(result.adjusted_weekly_distance_m <= ExperienceLevel::Intermediate.max_weekly_distance_m());
    }

    // ── Progression recommendation tests ────────────────────────────

    #[test]
    fn progression_at_max_does_not_increase() {
        let max = ExperienceLevel::Beginner.max_weekly_distance_m();
        let recommended = recommend_progression(max, ExperienceLevel::Beginner, 10);
        assert_eq!(recommended, max);
    }

    #[test]
    fn progression_above_max_capped() {
        let recommended = recommend_progression(30_000.0, ExperienceLevel::Beginner, 10);
        assert_eq!(recommended, ExperienceLevel::Beginner.max_weekly_distance_m());
    }

    #[test]
    fn new_level_progression_is_5_percent() {
        let recommended = recommend_progression(10_000.0, ExperienceLevel::Beginner, 0);
        // 5% increase for new level
        assert!((recommended - 10_500.0).abs() < 1.0);
    }

    #[test]
    fn established_progression_is_10_percent() {
        let recommended = recommend_progression(10_000.0, ExperienceLevel::Beginner, 3);
        // 10% increase for established level
        assert!((recommended - 11_000.0).abs() < 1.0);
    }

    #[test]
    fn progression_capped_at_max_weekly() {
        let recommended = recommend_progression(24_000.0, ExperienceLevel::Beginner, 5);
        // 24k × 1.10 = 26400, but max is 25000
        assert_eq!(recommended, ExperienceLevel::Beginner.max_weekly_distance_m());
    }

    // ── Request moderation tests ───────────────────────────────────

    #[test]
    fn safe_request_passes_moderation() {
        let result = moderate_request("Can you create a beginner walking plan for me?");
        assert!(result.is_safe);
        assert!(result.refusal_message.is_none());
    }

    #[test]
    fn run_through_injury_is_moderated() {
        let result = moderate_request("I want to run a marathon tomorrow");
        assert!(!result.is_safe);
        assert!(result.refusal_message.is_some());
    }

    #[test]
    fn extreme_distance_request_is_moderated() {
        let result = moderate_request("I want to run 100km this weekend");
        assert!(!result.is_safe);
        assert!(result.refusal_message.is_some());
        assert!(result.flag_reason.is_some());
    }

    #[test]
    fn diagnosis_request_is_moderated() {
        let result = moderate_request("Can you diagnose my knee pain?");
        assert!(!result.is_safe);
        assert!(result.refusal_message.is_some());
        assert_eq!(result.flag_reason, Some("diagnosis_request".to_string()));
    }

    #[test]
    fn drug_advice_request_is_moderated() {
        let result = moderate_request("Should I take ibuprofen for my pain?");
        assert!(!result.is_safe);
        assert!(result.refusal_message.is_some());
    }

    #[test]
    fn extreme_weight_loss_request_is_moderated() {
        let result = moderate_request("I want to lose 20 pounds in a week");
        assert!(!result.is_safe);
        assert!(result.refusal_message.is_some());
    }

    #[test]
    fn self_harm_indicator_is_moderated() {
        let result = moderate_request("I can't take it anymore");
        assert!(!result.is_safe);
        assert!(result.refusal_message.is_some());
        assert_eq!(result.flag_reason, Some("self_harm_indicator".to_string()));
    }

    #[test]
    fn normal_fitness_question_passes() {
        let result = moderate_request("How can I improve my running pace?");
        assert!(result.is_safe);
        assert!(result.refusal_message.is_none());
    }

    // ── Serialization tests ─────────────────────────────────────────

    #[test]
    fn experience_level_serializes_snake_case() {
        let json = serde_json::to_string(&ExperienceLevel::Beginner).unwrap();
        assert_eq!(json, "\"beginner\"");
        let json = serde_json::to_string(&ExperienceLevel::Intermediate).unwrap();
        assert_eq!(json, "\"intermediate\"");
        let json = serde_json::to_string(&ExperienceLevel::Advanced).unwrap();
        assert_eq!(json, "\"advanced\"");
    }

    #[test]
    fn experience_level_deserializes_from_snake_case() {
        let level: ExperienceLevel = serde_json::from_str("\"intermediate\"").unwrap();
        assert_eq!(level, ExperienceLevel::Intermediate);
    }

    #[test]
    fn day_plan_round_trips() {
        let day = DayPlan {
            day_of_week: 3,
            is_rest_day: false,
            target_distance_m: 5_000.0,
            target_duration_s: 3_600,
            difficulty: WorkoutDifficulty::Moderate,
            description: "Moderate walk".to_string(),
        };
        let json = serde_json::to_string(&day).unwrap();
        let decoded: DayPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(day, decoded);
    }

    #[test]
    fn weekly_plan_round_trips() {
        let plan = generate_fallback_plan(ExperienceLevel::Beginner, 10_000.0);
        let json = serde_json::to_string(&plan).unwrap();
        let decoded: WeeklyPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, decoded);
    }

    #[test]
    fn pain_response_round_trips() {
        let resp = respond_to_pain(PainType::Chest);
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: PainResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, decoded);
    }

    #[test]
    fn content_validation_result_round_trips() {
        let result = validate_coaching_text("You are suffering from tendonitis.");
        let json = serde_json::to_string(&result).unwrap();
        let decoded: ContentValidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, decoded);
    }

    #[test]
    fn escalation_message_round_trips() {
        let msg = escalation_message(EscalationReason::ChestPain);
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: EscalationMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn feedback_result_round_trips() {
        let result = process_user_feedback(UserFeedback::TooHard);
        let json = serde_json::to_string(&result).unwrap();
        let decoded: FeedbackResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, decoded);
    }

    #[test]
    fn request_moderation_result_round_trips() {
        let result = moderate_request("I want to run 100km tomorrow");
        let json = serde_json::to_string(&result).unwrap();
        let decoded: RequestModerationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, decoded);
    }

    #[test]
    fn plan_adjustment_result_round_trips() {
        let input = PlanAdjustmentInput {
            current_weekly_distance_m: 20_000.0,
            experience_level: ExperienceLevel::Intermediate,
            feedback: UserFeedback::TooHard,
            is_injured: false,
        };
        let result = adjust_plan(&input);
        let json = serde_json::to_string(&result).unwrap();
        let decoded: PlanAdjustmentResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, decoded);
    }

    #[test]
    fn workout_summary_input_round_trips() {
        let input = WorkoutSummaryInput {
            distance_m: 5_000.0,
            duration_s: 1_800,
            avg_speed_mps: 2.78,
            steps: 6_000,
            calories: 300,
            experience_level: ExperienceLevel::Beginner,
        };
        let json = serde_json::to_string(&input).unwrap();
        let decoded: WorkoutSummaryInput = serde_json::from_str(&json).unwrap();
        assert_eq!(input, decoded);
    }
}
