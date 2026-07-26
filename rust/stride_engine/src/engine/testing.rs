//! Testing framework (spec section 15).
//!
//! Comprehensive test configuration, execution, and reporting system that
//! spans all engine subsystems. This module defines the *structure* of the
//! test suite — what tests exist, what layers they belong to, what
//! environments they require, and how their results are aggregated —
//! rather than performing the actual side effects (running Flutter
//! widget tests, driving physical GPS, etc.).
//!
//! Key responsibilities:
//!   - Classify tests by layer (unit, database, repository, sync,
//!     security-rules, AI-schema, widget, navigation, integration)
//!   - Classify tests by category (distance, pace, speed, calories,
//!     timing, plan progression, etc.)
//!   - Track test execution status (pending, running, passed, failed,
//!     skipped, flaky)
//!   - Define test environments for real-device testing (device
//!     profiles, OS versions, screen sizes, GPS quality, network
//!     state, etc.)
//!   - Build test suites for automated and real-device testing
//!   - Run test scenarios and produce results
//!   - Aggregate results into a test report with pass rate and
//!     coverage metrics
//!
//! The actual test execution (spawning Flutter, driving sensors,
//! measuring battery) happens on the Dart/Kotlin side. This module is
//! purely test-structure definitions and result aggregation, and is
//! side-effect free.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Test layer
// ---------------------------------------------------------------------------

/// The layer of the testing pyramid a test belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestLayer {
    /// Unit tests for pure functions (distance, pace, speed, calories,
    /// timing, plan progression).
    Unit,
    /// Database tests (SQLite schema, migrations, CRUD operations).
    Database,
    /// Repository tests (cloud repository interface conformance).
    Repository,
    /// Sync tests (local-to-cloud, conflict resolution, retry logic).
    Sync,
    /// Security-rules tests (Firestore rules validation).
    SecurityRules,
    /// AI schema tests (AI input/output validation, plan rules).
    AiSchema,
    /// Widget tests (Flutter widget rendering, interaction).
    Widget,
    /// Navigation tests (GoRouter routing, auth gate).
    Navigation,
    /// Integration tests (end-to-end engine flow).
    Integration,
}

impl TestLayer {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            TestLayer::Unit => "Unit",
            TestLayer::Database => "Database",
            TestLayer::Repository => "Repository",
            TestLayer::Sync => "Sync",
            TestLayer::SecurityRules => "Security rules",
            TestLayer::AiSchema => "AI schema",
            TestLayer::Widget => "Widget",
            TestLayer::Navigation => "Navigation",
            TestLayer::Integration => "Integration",
        }
    }

    /// Returns `true` if this layer requires a real device (vs. a
    /// simulator/emulator or CI runner).
    pub fn requires_device(self) -> bool {
        matches!(self, TestLayer::Integration)
    }

    /// Returns `true` if this layer can run in CI without any device.
    pub fn is_ci_friendly(self) -> bool {
        matches!(
            self,
            TestLayer::Unit
                | TestLayer::Database
                | TestLayer::Repository
                | TestLayer::Sync
                | TestLayer::SecurityRules
                | TestLayer::AiSchema
        )
    }

    /// Returns `true` if this layer requires the Flutter framework (vs.
    /// pure Dart or Rust-only tests).
    pub fn requires_flutter(self) -> bool {
        matches!(
            self,
            TestLayer::Widget | TestLayer::Navigation | TestLayer::Integration
        )
    }
}

// ---------------------------------------------------------------------------
// Test category
// ---------------------------------------------------------------------------

/// The functional category a test covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCategory {
    /// Distance calculation tests.
    Distance,
    /// Pace calculation tests.
    Pace,
    /// Speed calculation tests.
    Speed,
    /// Calorie estimation tests.
    Calories,
    /// Timing / elapsed-time tests.
    Timing,
    /// Training plan progression tests.
    PlanProgression,
    /// SQLite database operation tests.
    Database,
    /// Cloud repository interface tests.
    Repository,
    /// Sync engine tests.
    Sync,
    /// Security rules validation tests.
    Security,
    /// AI schema and plan-rule tests.
    AiSchema,
    /// Widget rendering tests.
    Widget,
    /// Navigation / routing tests.
    Navigation,
    /// End-to-end integration tests.
    Integration,
    /// GPS quality / accuracy tests.
    Gps,
    /// Background execution / battery tests.
    Background,
    /// Wearable / Health Connect tests.
    Wearable,
    /// Music / audio tests.
    Music,
    /// Notification tests.
    Notifications,
    /// Error / recovery tests.
    ErrorRecovery,
}

impl TestCategory {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            TestCategory::Distance => "Distance",
            TestCategory::Pace => "Pace",
            TestCategory::Speed => "Speed",
            TestCategory::Calories => "Calories",
            TestCategory::Timing => "Timing",
            TestCategory::PlanProgression => "Plan progression",
            TestCategory::Database => "Database",
            TestCategory::Repository => "Repository",
            TestCategory::Sync => "Sync",
            TestCategory::Security => "Security",
            TestCategory::AiSchema => "AI schema",
            TestCategory::Widget => "Widget",
            TestCategory::Navigation => "Navigation",
            TestCategory::Integration => "Integration",
            TestCategory::Gps => "GPS",
            TestCategory::Background => "Background",
            TestCategory::Wearable => "Wearable",
            TestCategory::Music => "Music",
            TestCategory::Notifications => "Notifications",
            TestCategory::ErrorRecovery => "Error recovery",
        }
    }

    /// The default test layer for this category.
    pub fn default_layer(self) -> TestLayer {
        match self {
            TestCategory::Distance
            | TestCategory::Pace
            | TestCategory::Speed
            | TestCategory::Calories
            | TestCategory::Timing
            | TestCategory::PlanProgression => TestLayer::Unit,
            TestCategory::Database => TestLayer::Database,
            TestCategory::Repository => TestLayer::Repository,
            TestCategory::Sync => TestLayer::Sync,
            TestCategory::Security => TestLayer::SecurityRules,
            TestCategory::AiSchema => TestLayer::AiSchema,
            TestCategory::Widget => TestLayer::Widget,
            TestCategory::Navigation => TestLayer::Navigation,
            TestCategory::Integration => TestLayer::Integration,
            TestCategory::Gps => TestLayer::Integration,
            TestCategory::Background => TestLayer::Integration,
            TestCategory::Wearable => TestLayer::Integration,
            TestCategory::Music => TestLayer::Widget,
            TestCategory::Notifications => TestLayer::Unit,
            TestCategory::ErrorRecovery => TestLayer::Unit,
        }
    }
}

// ---------------------------------------------------------------------------
// Test status
// ---------------------------------------------------------------------------

/// The execution status of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    /// The test has not been run yet.
    Pending,
    /// The test is currently running.
    Running,
    /// The test passed.
    Passed,
    /// The test failed.
    Failed,
    /// The test was skipped (e.g., a prerequisite failed).
    Skipped,
    /// The test passed intermittently — it failed on some runs but
    /// passed on others (flaky).
    Flaky,
}

impl Default for TestStatus {
    fn default() -> Self {
        TestStatus::Pending
    }
}

impl TestStatus {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            TestStatus::Pending => "Pending",
            TestStatus::Running => "Running",
            TestStatus::Passed => "Passed",
            TestStatus::Failed => "Failed",
            TestStatus::Skipped => "Skipped",
            TestStatus::Flaky => "Flaky",
        }
    }

    /// Returns `true` if the test passed (including flaky).
    pub fn is_passing(self) -> bool {
        matches!(self, TestStatus::Passed | TestStatus::Flaky)
    }

    /// Returns `true` if the test is done (not pending or running).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            TestStatus::Passed | TestStatus::Failed | TestStatus::Skipped | TestStatus::Flaky
        )
    }

    /// Returns `true` if this status counts as a failure for reporting.
    pub fn is_failure(self) -> bool {
        matches!(self, TestStatus::Failed)
    }
}

// ---------------------------------------------------------------------------
// Test severity
// ---------------------------------------------------------------------------

/// How critical a test is — critical tests block release, low tests are
/// informational.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestSeverity {
    /// Low — informational, does not block release.
    Low,
    /// Medium — should pass but won't block a hotfix.
    Medium,
    /// High — blocks release if it fails.
    High,
    /// Critical — must pass for any deployment, including internal
    /// testing.
    Critical,
}

impl Default for TestSeverity {
    fn default() -> Self {
        TestSeverity::Medium
    }
}

impl TestSeverity {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            TestSeverity::Low => "low",
            TestSeverity::Medium => "medium",
            TestSeverity::High => "high",
            TestSeverity::Critical => "critical",
        }
    }

    /// Returns `true` if a failure of this test should block release.
    pub fn blocks_release(self) -> bool {
        matches!(self, TestSeverity::High | TestSeverity::Critical)
    }
}

// ---------------------------------------------------------------------------
// Test config
// ---------------------------------------------------------------------------

/// Configuration for a single test case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestConfig {
    /// A unique, stable identifier for the test (e.g.,
    /// "unit_distance_haversine_known").
    pub id: String,
    /// A human-readable name for the test.
    pub name: String,
    /// The test layer.
    pub layer: TestLayer,
    /// The test category.
    pub category: TestCategory,
    /// A description of what the test validates.
    pub description: String,
    /// Timeout in milliseconds (0 = no timeout).
    pub timeout_ms: u64,
    /// Number of times to retry on failure (0 = no retries).
    pub retry_count: u32,
    /// The severity of the test.
    pub severity: TestSeverity,
    /// Tags for grouping / filtering.
    pub tags: Vec<String>,
    /// Whether this test requires a real device.
    pub requires_device: bool,
}

impl TestConfig {
    /// Creates a new test config with the given id, name, layer, and
    /// category. Other fields default to sensible values.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        layer: TestLayer,
        category: TestCategory,
    ) -> Self {
        let id = id.into();
        let requires_device = layer.requires_device();
        Self {
            id,
            name: name.into(),
            layer,
            category,
            description: String::new(),
            timeout_ms: 30_000,
            retry_count: 0,
            severity: TestSeverity::Medium,
            tags: Vec::new(),
            requires_device,
        }
    }

    /// Sets the description and returns self for chaining.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Sets the timeout and returns self for chaining.
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Sets the retry count and returns self for chaining.
    pub fn with_retries(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// Sets the severity and returns self for chaining.
    pub fn with_severity(mut self, severity: TestSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Adds a tag and returns self for chaining.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Returns `true` if this test has the given tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

// ---------------------------------------------------------------------------
// Test result
// ---------------------------------------------------------------------------

/// The result of running a single test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestResult {
    /// The test config that was run.
    pub config: TestConfig,
    /// The final status of the test.
    pub status: TestStatus,
    /// How long the test took in milliseconds.
    pub duration_ms: u64,
    /// Error message if the test failed (None if passed).
    pub error_message: Option<String>,
    /// UTC timestamp (epoch milliseconds) when the test finished.
    pub finished_at_ms: i64,
    /// Number of assertions that passed.
    pub assertions_passed: u32,
    /// Number of assertions that failed.
    pub assertions_failed: u32,
    /// How many retries were used (0 = first attempt passed).
    pub retries_used: u32,
}

impl TestResult {
    /// Creates a passing test result.
    pub fn passed(config: TestConfig, duration_ms: u64, assertions: u32) -> Self {
        Self {
            config,
            status: TestStatus::Passed,
            duration_ms,
            error_message: None,
            finished_at_ms: 0,
            assertions_passed: assertions,
            assertions_failed: 0,
            retries_used: 0,
        }
    }

    /// Creates a failing test result.
    pub fn failed(
        config: TestConfig,
        duration_ms: u64,
        error: impl Into<String>,
        assertions_passed: u32,
        assertions_failed: u32,
        retries_used: u32,
    ) -> Self {
        Self {
            config,
            status: TestStatus::Failed,
            duration_ms,
            error_message: Some(error.into()),
            finished_at_ms: 0,
            assertions_passed,
            assertions_failed,
            retries_used,
        }
    }

    /// Creates a skipped test result.
    pub fn skipped(config: TestConfig, reason: impl Into<String>) -> Self {
        Self {
            config,
            status: TestStatus::Skipped,
            duration_ms: 0,
            error_message: Some(reason.into()),
            finished_at_ms: 0,
            assertions_passed: 0,
            assertions_failed: 0,
            retries_used: 0,
        }
    }

    /// Returns `true` if this test passed.
    pub fn is_passing(&self) -> bool {
        self.status.is_passing()
    }

    /// Returns `true` if this test is a blocking failure.
    pub fn is_blocking(&self) -> bool {
        self.status.is_failure() && self.config.severity.blocks_release()
    }
}

// ---------------------------------------------------------------------------
// Test suite
// ---------------------------------------------------------------------------

/// A collection of test configs and their results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestSuite {
    /// A unique name for the suite (e.g., "unit_tests", "real_device").
    pub name: String,
    /// The test layer this suite covers.
    pub layer: TestLayer,
    /// The test configs in this suite.
    pub configs: Vec<TestConfig>,
    /// The results of running the tests (empty if not yet run).
    pub results: Vec<TestResult>,
}

impl TestSuite {
    /// Creates a new test suite with the given name and layer.
    pub fn new(name: impl Into<String>, layer: TestLayer) -> Self {
        Self {
            name: name.into(),
            layer,
            configs: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Adds a test config to the suite.
    pub fn add(&mut self, config: TestConfig) {
        self.configs.push(config);
    }

    /// Adds multiple test configs to the suite.
    pub fn add_all(&mut self, configs: impl IntoIterator<Item = TestConfig>) {
        self.configs.extend(configs);
    }

    /// Returns the total number of tests in the suite.
    pub fn total(&self) -> usize {
        self.configs.len()
    }

    /// Returns the number of results that passed.
    pub fn passed(&self) -> usize {
        self.results.iter().filter(|r| r.is_passing()).count()
    }

    /// Returns the number of results that failed.
    pub fn failed(&self) -> usize {
        self.results.iter().filter(|r| r.status.is_failure()).count()
    }

    /// Returns the number of results that were skipped.
    pub fn skipped(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count()
    }

    /// Returns the number of results that are flaky.
    pub fn flaky(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Flaky)
            .count()
    }

    /// Returns the number of results that are still pending or running.
    pub fn pending(&self) -> usize {
        self.results
            .iter()
            .filter(|r| !r.status.is_terminal())
            .count()
    }

    /// Returns the pass rate as a fraction (0.0 to 1.0). Returns 0.0
    /// if there are no results.
    pub fn pass_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.passed() as f64 / self.results.len() as f64
    }

    /// Returns `true` if all tests in the suite have been run and
    /// passed.
    pub fn all_passed(&self) -> bool {
        !self.results.is_empty()
            && self.results.len() == self.configs.len()
            && self.results.iter().all(|r| r.is_passing())
    }

    /// Returns `true` if there are any blocking failures.
    pub fn has_blocking_failures(&self) -> bool {
        self.results.iter().any(|r| r.is_blocking())
    }
}

// ---------------------------------------------------------------------------
// Device profile (for real-device testing)
// ---------------------------------------------------------------------------

/// The type of physical device for real-device testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceProfile {
    /// A low-end Android phone (e.g., 2-3 GB RAM, older CPU).
    LowEndPhone,
    /// A new flagship phone (e.g., Galaxy S24, Pixel 8).
    FlagshipPhone,
    /// A tablet (larger screen, may have no GPS or cellular).
    Tablet,
    /// Samsung Galaxy Watch Ultra (Wear OS).
    WatchUltra,
}

impl DeviceProfile {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            DeviceProfile::LowEndPhone => "Low-end phone",
            DeviceProfile::FlagshipPhone => "Flagship phone",
            DeviceProfile::Tablet => "Tablet",
            DeviceProfile::WatchUltra => "Samsung Galaxy Watch Ultra",
        }
    }

    /// Returns `true` if this profile is a phone (not a tablet or watch).
    pub fn is_phone(self) -> bool {
        matches!(self, DeviceProfile::LowEndPhone | DeviceProfile::FlagshipPhone)
    }

    /// Returns `true` if this profile is a wearable.
    pub fn is_wearable(self) -> bool {
        matches!(self, DeviceProfile::WatchUltra)
    }
}

// ---------------------------------------------------------------------------
// Network state (for test environments)
// ---------------------------------------------------------------------------

/// The network connectivity state for a test environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkState {
    /// Full connectivity (Wi-Fi or strong cellular).
    Online,
 /// Weak but usable connectivity.
    Weak,
    /// No connectivity (dead zone / airplane mode).
    Offline,
    /// Intermittent connectivity (flapping on and off).
    Intermittent,
}

impl NetworkState {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            NetworkState::Online => "Online",
            NetworkState::Weak => "Weak",
            NetworkState::Offline => "Offline",
            NetworkState::Intermittent => "Intermittent",
        }
    }

    /// Returns `true` if the network is available at all.
    pub fn is_available(self) -> bool {
        matches!(self, NetworkState::Online | NetworkState::Weak | NetworkState::Intermittent)
    }
}

// ---------------------------------------------------------------------------
// GPS quality (for test environments)
// ---------------------------------------------------------------------------

/// The GPS signal quality for a test environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpsQuality {
    /// No GPS fix at all.
    NoFix,
    /// Poor GPS (wide accuracy, slow updates).
    Poor,
    /// Good GPS (acceptable accuracy for tracking).
    Good,
    /// Excellent GPS (high accuracy, fast updates).
    Excellent,
}

impl GpsQuality {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            GpsQuality::NoFix => "No fix",
            GpsQuality::Poor => "Poor",
            GpsQuality::Good => "Good",
            GpsQuality::Excellent => "Excellent",
        }
    }

    /// Returns `true` if GPS is usable for tracking at all.
    pub fn is_usable(self) -> bool {
        matches!(self, GpsQuality::Good | GpsQuality::Excellent)
    }
}

// ---------------------------------------------------------------------------
// Location type (for test environments)
// ---------------------------------------------------------------------------

/// The type of geographic location for a real-device test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationType {
    /// Indoor environment (poor GPS, possible Wi-Fi positioning).
    Indoor,
    /// Urban environment (tall buildings, multipath GPS).
    Urban,
    /// Rural area (open sky, good GPS).
    Rural,
    /// Dead cellular zone (no cell signal at all).
    DeadZone,
}

impl LocationType {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            LocationType::Indoor => "Indoor",
            LocationType::Urban => "Urban",
            LocationType::Rural => "Rural",
            LocationType::DeadZone => "Dead cellular zone",
        }
    }

    /// The expected GPS quality for this location type.
    pub fn default_gps_quality(self) -> GpsQuality {
        match self {
            LocationType::Indoor => GpsQuality::Poor,
            LocationType::Urban => GpsQuality::Good,
            LocationType::Rural => GpsQuality::Excellent,
            LocationType::DeadZone => GpsQuality::NoFix,
        }
    }
}

// ---------------------------------------------------------------------------
// Test environment
// ---------------------------------------------------------------------------

/// A complete test environment for real-device testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestEnvironment {
    /// The device profile being tested.
    pub device: DeviceProfile,
    /// The Android OS version (e.g., "13", "14").
    pub os_version: String,
    /// Screen size in inches (for tablet vs. phone testing).
    pub screen_size_inches: f64,
    /// Whether a Samsung Galaxy Watch Ultra is paired.
    pub has_watch: bool,
    /// The GPS quality for this environment.
    pub gps_quality: GpsQuality,
    /// The network state for this environment.
    pub network_state: NetworkState,
    /// Battery level (0-100, -1 = unknown).
    pub battery_level: i32,
    /// The location type (urban, rural, etc.).
    pub location_type: LocationType,
    /// Whether the screen is off during the test.
    pub screen_off: bool,
    /// Whether battery saver is enabled.
    pub battery_saver: bool,
    /// Whether the app was force-stopped during the test.
    pub force_stopped: bool,
    /// Whether the phone was restarted during the test.
    pub phone_restarted: bool,
}

impl TestEnvironment {
    /// Creates a default test environment for the given device profile.
    pub fn for_device(device: DeviceProfile) -> Self {
        let (os_version, screen_size, has_watch) = match device {
            DeviceProfile::LowEndPhone => ("12".to_string(), 5.5, false),
            DeviceProfile::FlagshipPhone => ("14".to_string(), 6.7, false),
            DeviceProfile::Tablet => ("13".to_string(), 11.0, false),
            DeviceProfile::WatchUltra => ("14".to_string(), 1.5, true),
        };
        Self {
            device,
            os_version,
            screen_size_inches: screen_size,
            has_watch,
            gps_quality: GpsQuality::Good,
            network_state: NetworkState::Online,
            battery_level: 80,
            location_type: LocationType::Urban,
            screen_off: false,
            battery_saver: false,
            force_stopped: false,
            phone_restarted: false,
        }
    }

    /// Sets the GPS quality and returns self for chaining.
    pub fn with_gps(mut self, quality: GpsQuality) -> Self {
        self.gps_quality = quality;
        self
    }

    /// Sets the network state and returns self for chaining.
    pub fn with_network(mut self, state: NetworkState) -> Self {
        self.network_state = state;
        self
    }

    /// Sets the battery level and returns self for chaining.
    pub fn with_battery(mut self, level: i32) -> Self {
        self.battery_level = level;
        self
    }

    /// Sets the location type and returns self for chaining.
    pub fn with_location(mut self, loc: LocationType) -> Self {
        self.location_type = loc;
        self
    }

    /// Sets screen off and returns self for chaining.
    pub fn with_screen_off(mut self, off: bool) -> Self {
        self.screen_off = off;
        self
    }

    /// Sets battery saver and returns self for chaining.
    pub fn with_battery_saver(mut self, saver: bool) -> Self {
        self.battery_saver = saver;
        self
    }

    /// Sets force-stopped and returns self for chaining.
    pub fn with_force_stopped(mut self, stopped: bool) -> Self {
        self.force_stopped = stopped;
        self
    }

    /// Sets phone-restarted and returns self for chaining.
    pub fn with_phone_restarted(mut self, restarted: bool) -> Self {
        self.phone_restarted = restarted;
        self
    }

    /// Returns a human-readable description of the environment.
    pub fn description(&self) -> String {
        format!(
            "{} (Android {}, {:.1}\", GPS: {}, Net: {}, Battery: {}%, Location: {}{}{}{}{})",
            self.device.label(),
            self.os_version,
            self.screen_size_inches,
            self.gps_quality.label(),
            self.network_state.label(),
            self.battery_level,
            self.location_type.label(),
            if self.screen_off { ", screen off" } else { "" },
            if self.battery_saver { ", battery saver" } else { "" },
            if self.force_stopped { ", force-stopped" } else { "" },
            if self.phone_restarted { ", phone restarted" } else { "" },
        )
    }
}

// ---------------------------------------------------------------------------
// Real-device scenario
// ---------------------------------------------------------------------------

/// A real-device test scenario — a specific combination of device,
/// environment, and expected behaviors to validate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealDeviceScenario {
    /// A unique identifier for the scenario.
    pub id: String,
    /// A human-readable name for the scenario.
    pub name: String,
    /// A description of what the scenario tests.
    pub description: String,
    /// The device profile for this scenario.
    pub device: DeviceProfile,
    /// The test environment for this scenario.
    pub environment: TestEnvironment,
    /// The expected behaviors to validate (human-readable checklist).
    pub expected_behaviors: Vec<String>,
    /// The expected duration in minutes.
    pub duration_minutes: u32,
    /// The severity of this scenario.
    pub severity: TestSeverity,
}

impl RealDeviceScenario {
    /// Creates a new real-device scenario.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        device: DeviceProfile,
        environment: TestEnvironment,
    ) -> Self {
        let id = id.into();
        Self {
            id,
            name: name.into(),
            description: String::new(),
            device,
            environment,
            expected_behaviors: Vec::new(),
            duration_minutes: 30,
            severity: TestSeverity::High,
        }
    }

    /// Sets the description and returns self for chaining.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Adds an expected behavior and returns self for chaining.
    pub fn with_behavior(mut self, behavior: impl Into<String>) -> Self {
        self.expected_behaviors.push(behavior.into());
        self
    }

    /// Sets the duration and returns self for chaining.
    pub fn with_duration(mut self, minutes: u32) -> Self {
        self.duration_minutes = minutes;
        self
    }

    /// Sets the severity and returns self for chaining.
    pub fn with_severity(mut self, severity: TestSeverity) -> Self {
        self.severity = severity;
        self
    }
}

// ---------------------------------------------------------------------------
// Scenario result
// ---------------------------------------------------------------------------

/// The result of running a real-device scenario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// The scenario that was run.
    pub scenario: RealDeviceScenario,
    /// The test status.
    pub status: TestStatus,
    /// How long the scenario took in milliseconds.
    pub duration_ms: u64,
    /// Error message if the scenario failed.
    pub error_message: Option<String>,
    /// Which expected behaviors passed.
    pub behaviors_passed: u32,
    /// Which expected behaviors failed.
    pub behaviors_failed: u32,
    /// UTC timestamp (epoch milliseconds) when the scenario finished.
    pub finished_at_ms: i64,
    /// Notes from the tester.
    pub notes: Option<String>,
}

impl ScenarioResult {
    /// Creates a passing scenario result.
    pub fn passed(scenario: RealDeviceScenario, duration_ms: u64, behaviors: u32) -> Self {
        Self {
            scenario,
            status: TestStatus::Passed,
            duration_ms,
            error_message: None,
            behaviors_passed: behaviors,
            behaviors_failed: 0,
            finished_at_ms: 0,
            notes: None,
        }
    }

    /// Creates a failing scenario result.
    pub fn failed(
        scenario: RealDeviceScenario,
        duration_ms: u64,
        error: impl Into<String>,
        behaviors_passed: u32,
        behaviors_failed: u32,
    ) -> Self {
        Self {
            scenario,
            status: TestStatus::Failed,
            duration_ms,
            error_message: Some(error.into()),
            behaviors_passed,
            behaviors_failed,
            finished_at_ms: 0,
            notes: None,
        }
    }

    /// Returns `true` if this scenario passed.
    pub fn is_passing(&self) -> bool {
        self.status.is_passing()
    }

    /// Returns the pass rate of expected behaviors (0.0 to 1.0).
    pub fn behavior_pass_rate(&self) -> f64 {
        let total = self.behaviors_passed + self.behaviors_failed;
        if total == 0 {
            return 1.0;
        }
        self.behaviors_passed as f64 / total as f64
    }
}

// ---------------------------------------------------------------------------
// Test report
// ---------------------------------------------------------------------------

/// An aggregate test report covering multiple suites and scenarios.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestReport {
    /// The name of the report (e.g., "Pre-release regression",
    /// "Sprint 42").
    pub name: String,
    /// The test suites in this report.
    pub suites: Vec<TestSuite>,
    /// The real-device scenario results in this report.
    pub scenario_results: Vec<ScenarioResult>,
    /// UTC timestamp (epoch milliseconds) when the report was generated.
    pub generated_at_ms: i64,
}

impl TestReport {
    /// Creates a new test report.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            suites: Vec::new(),
            scenario_results: Vec::new(),
            generated_at_ms: 0,
        }
    }

    /// Adds a test suite to the report.
    pub fn add_suite(&mut self, suite: TestSuite) {
        self.suites.push(suite);
    }

    /// Adds a scenario result to the report.
    pub fn add_scenario_result(&mut self, result: ScenarioResult) {
        self.scenario_results.push(result);
    }

    /// Returns the total number of tests across all suites.
    pub fn total_tests(&self) -> usize {
        self.suites.iter().map(|s| s.configs.len()).sum()
    }

    /// Returns the total number of results across all suites.
    pub fn total_results(&self) -> usize {
        self.suites.iter().map(|s| s.results.len()).sum()
    }

    /// Returns the number of passing test results.
    pub fn passed(&self) -> usize {
        self.suites.iter().map(|s| s.passed()).sum()
    }

    /// Returns the number of failing test results.
    pub fn failed(&self) -> usize {
        self.suites.iter().map(|s| s.failed()).sum()
    }

    /// Returns the number of skipped test results.
    pub fn skipped(&self) -> usize {
        self.suites.iter().map(|s| s.skipped()).sum()
    }

    /// Returns the number of flaky test results.
    pub fn flaky(&self) -> usize {
        self.suites.iter().map(|s| s.flaky()).sum()
    }

    /// Returns the number of passing scenario results.
    pub fn scenarios_passed(&self) -> usize {
        self.scenario_results.iter().filter(|r| r.is_passing()).count()
    }

    /// Returns the number of failing scenario results.
    pub fn scenarios_failed(&self) -> usize {
        self.scenario_results
            .iter()
            .filter(|r| r.status.is_failure())
            .count()
    }

    /// Returns the total number of scenario results.
    pub fn total_scenarios(&self) -> usize {
        self.scenario_results.len()
    }

    /// Returns the overall test pass rate as a fraction (0.0 to 1.0).
    /// Returns 0.0 if there are no results.
    pub fn test_pass_rate(&self) -> f64 {
        let total = self.total_results();
        if total == 0 {
            return 0.0;
        }
        self.passed() as f64 / total as f64
    }

    /// Returns the overall scenario pass rate as a fraction (0.0 to 1.0).
    /// Returns 0.0 if there are no scenario results.
    pub fn scenario_pass_rate(&self) -> f64 {
        let total = self.total_scenarios();
        if total == 0 {
            return 0.0;
        }
        self.scenarios_passed() as f64 / total as f64
    }

    /// Returns `true` if the report indicates a blocking failure.
    pub fn has_blocking_failures(&self) -> bool {
        self.suites.iter().any(|s| s.has_blocking_failures())
            || self
                .scenario_results
                .iter()
                .any(|r| r.status.is_failure() && r.scenario.severity.blocks_release())
    }

    /// Returns `true` if the report is release-ready (no blocking
    /// failures and all critical tests passed).
    pub fn is_release_ready(&self) -> bool {
        !self.has_blocking_failures() && self.test_pass_rate() > 0.0
    }

    /// Returns a human-readable summary of the report.
    pub fn summary(&self) -> String {
        format!(
            "Test report '{}': {}/{} tests passed ({:.1}%), {}/{} scenarios passed ({:.1}%){}",
            self.name,
            self.passed(),
            self.total_results(),
            self.test_pass_rate() * 100.0,
            self.scenarios_passed(),
            self.total_scenarios(),
            self.scenario_pass_rate() * 100.0,
            if self.has_blocking_failures() {
                " — BLOCKING FAILURES DETECTED"
            } else if self.is_release_ready() {
                " — release ready"
            } else {
                ""
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Coverage metrics
// ---------------------------------------------------------------------------

/// Test coverage metrics for a specific category or layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageMetrics {
    /// The layer or category name this coverage applies to.
    pub name: String,
    /// Total number of tests for this area.
    pub total: usize,
    /// Number of passing tests.
    pub passed: usize,
    /// Number of failing tests.
    pub failed: usize,
    /// Number of skipped tests.
    pub skipped: usize,
    /// Pass rate (0.0 to 1.0).
    pub pass_rate: f64,
}

/// Computes coverage metrics for a test report grouped by test layer.
pub fn compute_coverage_by_layer(report: &TestReport) -> Vec<CoverageMetrics> {
    let mut metrics: Vec<CoverageMetrics> = Vec::new();

    for layer in [
        TestLayer::Unit,
        TestLayer::Database,
        TestLayer::Repository,
        TestLayer::Sync,
        TestLayer::SecurityRules,
        TestLayer::AiSchema,
        TestLayer::Widget,
        TestLayer::Navigation,
        TestLayer::Integration,
    ] {
        let suite: Vec<&TestSuite> = report.suites.iter().filter(|s| s.layer == layer).collect();
        if suite.is_empty() {
            continue;
        }

        let total: usize = suite.iter().map(|s| s.configs.len()).sum();
        let passed: usize = suite.iter().map(|s| s.passed()).sum();
        let failed: usize = suite.iter().map(|s| s.failed()).sum();
        let skipped: usize = suite.iter().map(|s| s.skipped()).sum();
        let pass_rate = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64
        };

        metrics.push(CoverageMetrics {
            name: layer.label().to_string(),
            total,
            passed,
            failed,
            skipped,
            pass_rate,
        });
    }

    metrics
}

/// Computes coverage metrics for a test report grouped by test category.
pub fn compute_coverage_by_category(report: &TestReport) -> Vec<CoverageMetrics> {
    let mut metrics: Vec<CoverageMetrics> = Vec::new();

    for category in [
        TestCategory::Distance,
        TestCategory::Pace,
        TestCategory::Speed,
        TestCategory::Calories,
        TestCategory::Timing,
        TestCategory::PlanProgression,
        TestCategory::Database,
        TestCategory::Repository,
        TestCategory::Sync,
        TestCategory::Security,
        TestCategory::AiSchema,
        TestCategory::Widget,
        TestCategory::Navigation,
        TestCategory::Integration,
        TestCategory::Gps,
        TestCategory::Background,
        TestCategory::Wearable,
        TestCategory::Music,
        TestCategory::Notifications,
        TestCategory::ErrorRecovery,
    ] {
        let results: Vec<&TestResult> = report
            .suites
            .iter()
            .flat_map(|s| s.results.iter())
            .filter(|r| r.config.category == category)
            .collect();
        if results.is_empty() {
            continue;
        }

        let total = results.len();
        let passed = results.iter().filter(|r| r.is_passing()).count();
        let failed = results.iter().filter(|r| r.status.is_failure()).count();
        let skipped = results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count();
        let pass_rate = passed as f64 / total as f64;

        metrics.push(CoverageMetrics {
            name: category.label().to_string(),
            total,
            passed,
            failed,
            skipped,
            pass_rate,
        });
    }

    metrics
}

// ---------------------------------------------------------------------------
// Suite builders
// ---------------------------------------------------------------------------

/// Builds the default unit test suite for the S.T.R.I.D.E. engine.
/// These are the automated unit tests for distance, pace, speed,
/// calories, timing, and plan progression.
pub fn build_unit_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("unit_tests", TestLayer::Unit);

    suite.add(
        TestConfig::new(
            "unit_distance_haversine",
            "Haversine distance: known distances",
            TestLayer::Unit,
            TestCategory::Distance,
        )
        .with_description("Validates haversine formula against known distances")
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "unit_distance_zero_identical",
            "Haversine distance: zero for identical points",
            TestLayer::Unit,
            TestCategory::Distance,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "unit_pace_basic",
            "Pace calculation: basic pace from distance and time",
            TestLayer::Unit,
            TestCategory::Pace,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "unit_pace_zero_distance",
            "Pace calculation: handles zero distance gracefully",
            TestLayer::Unit,
            TestCategory::Pace,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite.add(
        TestConfig::new(
            "unit_speed_average",
            "Speed calculation: average speed from distance and time",
            TestLayer::Unit,
            TestCategory::Speed,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "unit_calories_basic",
            "Calorie estimation: basic MET-based calculation",
            TestLayer::Unit,
            TestCategory::Calories,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "unit_calories_clamp",
            "Calorie estimation: clamps impossible values",
            TestLayer::Unit,
            TestCategory::Calories,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite.add(
        TestConfig::new(
            "unit_timing_elapsed",
            "Timing: elapsed time with pause exclusion",
            TestLayer::Unit,
            TestCategory::Timing,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "unit_plan_beginner_limits",
            "Plan progression: beginner limits enforced",
            TestLayer::Unit,
            TestCategory::PlanProgression,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "unit_plan_rest_days",
            "Plan progression: rest-day requirements enforced",
            TestLayer::Unit,
            TestCategory::PlanProgression,
        )
        .with_severity(TestSeverity::High),
    );

    suite
}

/// Builds the database test suite for SQLite operations.
pub fn build_database_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("database_tests", TestLayer::Database);

    suite.add(
        TestConfig::new(
            "db_schema_create",
            "SQLite schema: creates all required tables",
            TestLayer::Database,
            TestCategory::Database,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "db_migration",
            "SQLite schema: migration applies correctly",
            TestLayer::Database,
            TestCategory::Database,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "db_crud_workout",
            "SQLite CRUD: workout insert/read/update/delete",
            TestLayer::Database,
            TestCategory::Database,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "db_crud_route_points",
            "SQLite CRUD: route point batch insert",
            TestLayer::Database,
            TestCategory::Database,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "db_checkpoint_recovery",
            "SQLite: checkpoint recovery after crash",
            TestLayer::Database,
            TestCategory::Database,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite
}

/// Builds the repository test suite for cloud repository interfaces.
pub fn build_repository_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("repository_tests", TestLayer::Repository);

    suite.add(
        TestConfig::new(
            "repo_workout_crud",
            "Cloud workout repository: CRUD operations",
            TestLayer::Repository,
            TestCategory::Repository,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "repo_route_crud",
            "Cloud route repository: CRUD operations",
            TestLayer::Repository,
            TestCategory::Repository,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "repo_summary_crud",
            "Cloud summary repository: CRUD operations",
            TestLayer::Repository,
            TestCategory::Repository,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite
}

/// Builds the sync test suite for the sync engine.
pub fn build_sync_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("sync_tests", TestLayer::Sync);

    suite.add(
        TestConfig::new(
            "sync_upload",
            "Sync engine: local-to-cloud upload",
            TestLayer::Sync,
            TestCategory::Sync,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "sync_download",
            "Sync engine: cloud-to-local download",
            TestLayer::Sync,
            TestCategory::Sync,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "sync_conflict_resolution",
            "Sync engine: conflict resolution",
            TestLayer::Sync,
            TestCategory::Sync,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "sync_retry_backoff",
            "Sync engine: retry with exponential backoff",
            TestLayer::Sync,
            TestCategory::Sync,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "sync_duplicate_prevention",
            "Sync engine: duplicate prevention",
            TestLayer::Sync,
            TestCategory::Sync,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "sync_deletion_propagation",
            "Sync engine: deletion propagation",
            TestLayer::Sync,
            TestCategory::Sync,
        )
        .with_severity(TestSeverity::High),
    );

    suite
}

/// Builds the security-rules test suite for Firestore rules.
pub fn build_security_rules_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("security_rules_tests", TestLayer::SecurityRules);

    suite.add(
        TestConfig::new(
            "sec_ownership_validation",
            "Firestore rules: ownership validation",
            TestLayer::SecurityRules,
            TestCategory::Security,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "sec_field_validation",
            "Firestore rules: field-level validation",
            TestLayer::SecurityRules,
            TestCategory::Security,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "sec_admin_only",
            "Firestore rules: admin-only collections",
            TestLayer::SecurityRules,
            TestCategory::Security,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "sec_rate_limiting",
            "Firestore rules: rate limiting",
            TestLayer::SecurityRules,
            TestCategory::Security,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite
}

/// Builds the AI schema test suite for AI input/output validation.
pub fn build_ai_schema_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("ai_schema_tests", TestLayer::AiSchema);

    suite.add(
        TestConfig::new(
            "ai_input_validation",
            "AI schema: input validation",
            TestLayer::AiSchema,
            TestCategory::AiSchema,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "ai_output_validation",
            "AI schema: output validation",
            TestLayer::AiSchema,
            TestCategory::AiSchema,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "ai_plan_rules",
            "AI schema: plan-rule enforcement (beginner limits, rest days)",
            TestLayer::AiSchema,
            TestCategory::AiSchema,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "ai_fallback",
            "AI schema: fallback to rule-based plan when AI unavailable",
            TestLayer::AiSchema,
            TestCategory::AiSchema,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "ai_cost_tracking",
            "AI schema: cost tracking and usage limits",
            TestLayer::AiSchema,
            TestCategory::AiSchema,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite
}

/// Builds the widget test suite for Flutter widgets.
pub fn build_widget_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("widget_tests", TestLayer::Widget);

    suite.add(
        TestConfig::new(
            "widget_dashboard_render",
            "Widget: dashboard renders correctly",
            TestLayer::Widget,
            TestCategory::Widget,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite.add(
        TestConfig::new(
            "widget_workout_screen",
            "Widget: workout screen displays live stats",
            TestLayer::Widget,
            TestCategory::Widget,
        )
        .with_severity(TestSeverity::High),
    );

    suite.add(
        TestConfig::new(
            "widget_navigation_auth_gate",
            "Widget: auth gate redirects unauthenticated users",
            TestLayer::Widget,
            TestCategory::Navigation,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite
}

/// Builds the navigation test suite for GoRouter routing.
pub fn build_navigation_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("navigation_tests", TestLayer::Navigation);

    suite.add(
        TestConfig::new(
            "nav_auth_gate",
            "Navigation: auth gate blocks unauthenticated access",
            TestLayer::Navigation,
            TestCategory::Navigation,
        )
        .with_severity(TestSeverity::Critical),
    );

    suite.add(
        TestConfig::new(
            "nav_deep_link",
            "Navigation: deep link routing",
            TestLayer::Navigation,
            TestCategory::Navigation,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite.add(
        TestConfig::new(
            "nav_tab_switching",
            "Navigation: tab switching preserves state",
            TestLayer::Navigation,
            TestCategory::Navigation,
        )
        .with_severity(TestSeverity::Low),
    );

    suite
}

/// Builds the integration test suite for end-to-end engine flow.
pub fn build_integration_test_suite() -> TestSuite {
    let mut suite = TestSuite::new("integration_tests", TestLayer::Integration);

    suite.add(
        TestConfig::new(
            "integ_full_workout_flow",
            "Integration: full workout lifecycle (start → pause → resume → finish)",
            TestLayer::Integration,
            TestCategory::Integration,
        )
        .with_description("Tests the complete workout lifecycle on a real device")
        .with_severity(TestSeverity::Critical)
        .with_timeout(600_000)
        .with_retries(1),
    );

    suite.add(
        TestConfig::new(
            "integ_gps_tracking",
            "Integration: GPS tracking with real signal",
            TestLayer::Integration,
            TestCategory::Gps,
        )
        .with_severity(TestSeverity::Critical)
        .with_timeout(300_000),
    );

    suite.add(
        TestConfig::new(
            "integ_background_tracking",
            "Integration: background tracking with screen off",
            TestLayer::Integration,
            TestCategory::Background,
        )
        .with_severity(TestSeverity::Critical)
        .with_timeout(600_000),
    );

    suite.add(
        TestConfig::new(
            "integ_offline_recovery",
            "Integration: offline recording → sync when connection returns",
            TestLayer::Integration,
            TestCategory::Sync,
        )
        .with_severity(TestSeverity::Critical)
        .with_timeout(300_000),
    );

    suite.add(
        TestConfig::new(
            "integ_watch_sync",
            "Integration: Samsung Galaxy Watch Ultra → phone sync",
            TestLayer::Integration,
            TestCategory::Wearable,
        )
        .with_severity(TestSeverity::High)
        .with_timeout(300_000),
    );

    suite.add(
        TestConfig::new(
            "integ_music_coaching",
            "Integration: music pause for coaching prompts",
            TestLayer::Integration,
            TestCategory::Music,
        )
        .with_severity(TestSeverity::Medium),
    );

    suite
}

// ---------------------------------------------------------------------------
// Real-device scenario builders
// ---------------------------------------------------------------------------

/// Builds the full real-device test suite with all required scenarios
/// from the specification.
pub fn build_real_device_suite() -> Vec<RealDeviceScenario> {
    let mut scenarios: Vec<RealDeviceScenario> = Vec::new();

    // Low-end phone
    scenarios.push(
        RealDeviceScenario::new(
            "rd_low_end_phone",
            "Low-end phone basic workout",
            DeviceProfile::LowEndPhone,
            TestEnvironment::for_device(DeviceProfile::LowEndPhone),
        )
        .with_description("Tests the app on a low-end phone with limited RAM")
        .with_behavior("App launches without crashing")
        .with_behavior("Workout starts and records GPS points")
        .with_behavior("Stats update every second")
        .with_behavior("Workout saves to SQLite")
        .with_duration(30),
    );

    // Flagship phone
    scenarios.push(
        RealDeviceScenario::new(
            "rd_flagship_phone",
            "Flagship phone full workout",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone),
        )
        .with_description("Tests the app on a flagship phone with all features")
        .with_behavior("Full workout lifecycle works")
        .with_behavior("Background tracking works with screen off")
        .with_behavior("Sync works when connection returns")
        .with_behavior("Music controls work")
        .with_duration(60),
    );

    // Tablet
    scenarios.push(
        RealDeviceScenario::new(
            "rd_tablet",
            "Tablet layout and screen sizes",
            DeviceProfile::Tablet,
            TestEnvironment::for_device(DeviceProfile::Tablet),
        )
        .with_description("Tests the app UI on a tablet with a large screen")
        .with_behavior("UI renders correctly at tablet size")
        .with_behavior("All screens are reachable")
        .with_behavior("No layout overflow")
        .with_duration(20),
    );

    // Samsung Galaxy Watch Ultra
    scenarios.push(
        RealDeviceScenario::new(
            "rd_watch_ultra",
            "Samsung Galaxy Watch Ultra integration",
            DeviceProfile::WatchUltra,
            TestEnvironment::for_device(DeviceProfile::WatchUltra),
        )
        .with_description("Tests Wear OS integration with the Galaxy Watch Ultra")
        .with_behavior("Watch heart-rate data syncs to phone")
        .with_behavior("Watch step data syncs to phone")
        .with_behavior("No duplicate steps counted")
        .with_behavior("Watch disconnects gracefully")
        .with_duration(45),
    );

    // No watch
    scenarios.push(
        RealDeviceScenario::new(
            "rd_no_watch",
            "Phone-only tracking (no watch)",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone).with_gps(GpsQuality::Good),
        )
        .with_description("Tests phone-only tracking without a paired watch")
        .with_behavior("Phone sensors track workout correctly")
        .with_behavior("No watch-dependent features break")
        .with_behavior("Fallback to phone-only mode works")
        .with_duration(30),
    );

    // Poor GPS area
    scenarios.push(
        RealDeviceScenario::new(
            "rd_poor_gps",
            "Poor GPS area tracking",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone)
                .with_gps(GpsQuality::Poor)
                .with_location(LocationType::Indoor),
        )
        .with_description("Tests the app in an area with poor GPS signal")
        .with_behavior("App degrades gracefully with poor GPS")
        .with_behavior("GPS filter removes bad points")
        .with_behavior("User is notified of GPS quality issues")
        .with_behavior("Workout continues with last known position")
        .with_duration(30),
    );

    // Urban environment
    scenarios.push(
        RealDeviceScenario::new(
            "rd_urban",
            "Urban environment tracking",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone)
                .with_gps(GpsQuality::Good)
                .with_location(LocationType::Urban),
        )
        .with_description("Tests the app in an urban environment with multipath GPS")
        .with_behavior("GPS handles urban multipath")
        .with_behavior("Route polyline renders correctly")
        .with_behavior("Distance is accurate within tolerance")
        .with_duration(45),
    );

    // Rural area
    scenarios.push(
        RealDeviceScenario::new(
            "rd_rural",
            "Rural area tracking",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone)
                .with_gps(GpsQuality::Excellent)
                .with_location(LocationType::Rural),
        )
        .with_description("Tests the app in a rural area with excellent GPS")
        .with_behavior("GPS tracking is highly accurate")
        .with_behavior("Long-distance tracking is stable")
        .with_duration(60),
    );

    // Dead cellular zone
    scenarios.push(
        RealDeviceScenario::new(
            "rd_dead_zone",
            "Dead cellular zone offline tracking",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone)
                .with_network(NetworkState::Offline)
                .with_location(LocationType::DeadZone),
        )
        .with_description("Tests the app in a dead cellular zone with no connectivity")
        .with_behavior("Workout records locally without internet")
        .with_behavior("No sync errors crash the app")
        .with_behavior("Data syncs when connection returns")
        .with_behavior("Pending uploads queue correctly")
        .with_duration(60),
    );

    // Screen off
    scenarios.push(
        RealDeviceScenario::new(
            "rd_screen_off",
            "Background tracking with screen off",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone).with_screen_off(true),
        )
        .with_description("Tests background GPS tracking with the screen off")
        .with_behavior("Foreground service keeps tracking")
        .with_behavior("GPS points are recorded while screen is off")
        .with_behavior("Notification persists during tracking")
        .with_behavior("Stats are correct when screen turns back on")
        .with_severity(TestSeverity::Critical)
        .with_duration(60),
    );

    // Battery saver
    scenarios.push(
        RealDeviceScenario::new(
            "rd_battery_saver",
            "Battery saver mode",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone)
                .with_battery_saver(true)
                .with_battery(15),
        )
        .with_description("Tests the app with battery saver enabled and low battery")
        .with_behavior("App enters reduced-power mode")
        .with_behavior("GPS sampling strategy adjusts")
        .with_behavior("Workout continues with reduced accuracy")
        .with_behavior("Battery drain is within acceptable limits")
        .with_severity(TestSeverity::High)
        .with_duration(45),
    );

    // App force-stopped
    scenarios.push(
        RealDeviceScenario::new(
            "rd_force_stopped",
            "App force-stopped recovery",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone).with_force_stopped(true),
        )
        .with_description("Tests recovery after the app is force-stopped")
        .with_behavior("Active workout is recoverable after force-stop")
        .with_behavior("Checkpoint data is intact")
        .with_behavior("User is prompted to resume the workout")
        .with_behavior("No data corruption occurs")
        .with_severity(TestSeverity::Critical)
        .with_duration(30),
    );

    // Phone restarted
    scenarios.push(
        RealDeviceScenario::new(
            "rd_phone_restarted",
            "Phone restart recovery",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone).with_phone_restarted(true),
        )
        .with_description("Tests recovery after the phone is restarted")
        .with_behavior("App recovers active workout after reboot")
        .with_behavior("Foreground service restarts if needed")
        .with_behavior("No data loss occurs")
        .with_behavior("User is prompted to resume")
        .with_severity(TestSeverity::Critical)
        .with_duration(45),
    );

    // Long multi-hour workout
    scenarios.push(
        RealDeviceScenario::new(
            "rd_long_workout",
            "Long multi-hour workout",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone).with_battery(90),
        )
        .with_description("Tests a long multi-hour workout (3+ hours)")
        .with_behavior("App stays stable for 3+ hours")
        .with_behavior("No memory leaks")
        .with_behavior("GPS tracking remains accurate")
        .with_behavior("Battery drain is linear")
        .with_behavior("Stats remain correct throughout")
        .with_severity(TestSeverity::High)
        .with_duration(180),
    );

    // Interrupted workout
    scenarios.push(
        RealDeviceScenario::new(
            "rd_interrupted",
            "Interrupted workout (call during tracking)",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone),
        )
        .with_description("Tests workout interrupted by a phone call")
        .with_behavior("Workout pauses during call")
        .with_behavior("Music pauses during call")
        .with_behavior("Workout resumes after call ends")
        .with_behavior("No data loss during interruption")
        .with_duration(30),
    );

    // Very slow movement
    scenarios.push(
        RealDeviceScenario::new(
            "rd_very_slow",
            "Very slow movement (walking slowly)",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone).with_gps(GpsQuality::Good),
        )
        .with_description("Tests very slow movement (e.g., walking at 2 km/h)")
        .with_behavior("Slow movement is classified as walking")
        .with_behavior("Auto-pause does not trigger at very slow speed")
        .with_behavior("Distance accumulates correctly at slow speed")
        .with_behavior("Pace is calculated correctly")
        .with_duration(30),
    );

    // Very fast movement
    scenarios.push(
        RealDeviceScenario::new(
            "rd_very_fast",
            "Very fast movement (running fast)",
            DeviceProfile::FlagshipPhone,
            TestEnvironment::for_device(DeviceProfile::FlagshipPhone).with_gps(GpsQuality::Good),
        )
        .with_description("Tests very fast movement (e.g., running at 15+ km/h)")
        .with_behavior("Fast movement is classified as running")
        .with_behavior("Distance accumulates correctly at high speed")
        .with_behavior("Speed and pace are calculated correctly")
        .with_behavior("GPS filter handles high-speed points")
        .with_duration(20),
    );

    scenarios
}

// ---------------------------------------------------------------------------
// Test registry
// ---------------------------------------------------------------------------

/// A registry of all test suites and their results, for tracking the
/// overall test status across CI runs and manual testing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRegistry {
    /// All test suites in the registry.
    pub suites: Vec<TestSuite>,
    /// All real-device scenarios in the registry.
    pub scenarios: Vec<RealDeviceScenario>,
    /// All scenario results.
    pub scenario_results: Vec<ScenarioResult>,
}

impl Default for TestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRegistry {
    /// Creates a new, empty test registry.
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
            scenarios: Vec::new(),
            scenario_results: Vec::new(),
        }
    }

    /// Adds a test suite to the registry.
    pub fn add_suite(&mut self, suite: TestSuite) {
        self.suites.push(suite);
    }

    /// Adds a real-device scenario to the registry.
    pub fn add_scenario(&mut self, scenario: RealDeviceScenario) {
        self.scenarios.push(scenario);
    }

    /// Adds a scenario result to the registry.
    pub fn add_scenario_result(&mut self, result: ScenarioResult) {
        self.scenario_results.push(result);
    }

    /// Returns the total number of test configs across all suites.
    pub fn total_tests(&self) -> usize {
        self.suites.iter().map(|s| s.configs.len()).sum()
    }

    /// Returns the total number of test results across all suites.
    pub fn total_results(&self) -> usize {
        self.suites.iter().map(|s| s.results.len()).sum()
    }

    /// Returns the number of passing test results.
    pub fn passed(&self) -> usize {
        self.suites.iter().map(|s| s.passed()).sum()
    }

    /// Returns the number of failing test results.
    pub fn failed(&self) -> usize {
        self.suites.iter().map(|s| s.failed()).sum()
    }

    /// Returns the names of all suites in the registry.
    pub fn suite_names(&self) -> Vec<String> {
        self.suites.iter().map(|s| s.name.clone()).collect()
    }

    /// Returns the names of all scenarios in the registry.
    pub fn scenario_names(&self) -> Vec<String> {
        self.scenarios.iter().map(|s| s.name.clone()).collect()
    }

    /// Returns the overall pass rate as a fraction (0.0 to 1.0).
    pub fn pass_rate(&self) -> f64 {
        let total = self.total_results();
        if total == 0 {
            return 0.0;
        }
        self.passed() as f64 / total as f64
    }

    /// Clears all results (but keeps configs).
    pub fn clear_results(&mut self) {
        for suite in &mut self.suites {
            suite.results.clear();
        }
        self.scenario_results.clear();
    }
}

/// Builds the complete test registry with all standard test suites
/// and real-device scenarios.
pub fn build_full_registry() -> TestRegistry {
    let mut registry = TestRegistry::new();

    registry.add_suite(build_unit_test_suite());
    registry.add_suite(build_database_test_suite());
    registry.add_suite(build_repository_test_suite());
    registry.add_suite(build_sync_test_suite());
    registry.add_suite(build_security_rules_test_suite());
    registry.add_suite(build_ai_schema_test_suite());
    registry.add_suite(build_widget_test_suite());
    registry.add_suite(build_navigation_test_suite());
    registry.add_suite(build_integration_test_suite());

    for scenario in build_real_device_suite() {
        registry.add_scenario(scenario);
    }

    registry
}

/// Builds a complete test report from a test registry, aggregating
/// all suite results and scenario results.
pub fn build_report(
    name: impl Into<String>,
    registry: &TestRegistry,
) -> TestReport {
    let mut report = TestReport::new(name);
    for suite in &registry.suites {
        report.add_suite(suite.clone());
    }
    for result in &registry.scenario_results {
        report.add_scenario_result(result.clone());
    }
    report
}

// ---------------------------------------------------------------------------
// Suite runner (simulated — actual execution is on Dart/Kotlin side)
// ---------------------------------------------------------------------------

/// Runs a test suite and returns a result suite with all results
/// marked as "passed" with zero duration. This is a *simulated* run —
/// the actual test execution happens on the Dart/Kotlin side. This
/// function exists to produce a result structure that can be
/// serialized and sent to the UI.
pub fn run_suite(suite: &TestSuite) -> TestSuite {
    let mut result_suite = TestSuite::new(suite.name.clone(), suite.layer);
    result_suite.configs = suite.configs.clone();
    for config in &suite.configs {
        result_suite.results.push(TestResult::passed(config.clone(), 0, 0));
    }
    result_suite
}

/// Runs a real-device scenario and returns a passing result with
/// zero duration. This is a *simulated* run — the actual test
/// execution happens on the Dart/Kotlin side.
pub fn run_scenario(scenario: &RealDeviceScenario) -> ScenarioResult {
    let behaviors = scenario.expected_behaviors.len() as u32;
    ScenarioResult::passed(scenario.clone(), 0, behaviors)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── TestLayer tests ─────────────────────────────────────────────

    #[test]
    fn layer_labels_are_non_empty() {
        for layer in [
            TestLayer::Unit,
            TestLayer::Database,
            TestLayer::Repository,
            TestLayer::Sync,
            TestLayer::SecurityRules,
            TestLayer::AiSchema,
            TestLayer::Widget,
            TestLayer::Navigation,
            TestLayer::Integration,
        ] {
            assert!(!layer.label().is_empty());
        }
    }

    #[test]
    fn layer_requires_device_integration_only() {
        assert!(TestLayer::Integration.requires_device());
        assert!(!TestLayer::Unit.requires_device());
        assert!(!TestLayer::Widget.requires_device());
    }

    #[test]
    fn layer_ci_friendly_excludes_widget_and_integration() {
        assert!(TestLayer::Unit.is_ci_friendly());
        assert!(TestLayer::Database.is_ci_friendly());
        assert!(TestLayer::SecurityRules.is_ci_friendly());
        assert!(!TestLayer::Widget.is_ci_friendly());
        assert!(!TestLayer::Navigation.is_ci_friendly());
        assert!(!TestLayer::Integration.is_ci_friendly());
    }

    #[test]
    fn layer_requires_flutter_for_widget_and_navigation() {
        assert!(TestLayer::Widget.requires_flutter());
        assert!(TestLayer::Navigation.requires_flutter());
        assert!(TestLayer::Integration.requires_flutter());
        assert!(!TestLayer::Unit.requires_flutter());
        assert!(!TestLayer::Database.requires_flutter());
    }

    #[test]
    fn layer_serializes_round_trip() {
        for layer in [
            TestLayer::Unit,
            TestLayer::Database,
            TestLayer::Repository,
            TestLayer::Sync,
            TestLayer::SecurityRules,
            TestLayer::AiSchema,
            TestLayer::Widget,
            TestLayer::Navigation,
            TestLayer::Integration,
        ] {
            let json = serde_json::to_string(&layer).unwrap();
            let back: TestLayer = serde_json::from_str(&json).unwrap();
            assert_eq!(layer, back);
        }
    }

    // ── TestCategory tests ──────────────────────────────────────────

    #[test]
    fn category_labels_are_non_empty() {
        for cat in [
            TestCategory::Distance,
            TestCategory::Pace,
            TestCategory::Speed,
            TestCategory::Calories,
            TestCategory::Timing,
            TestCategory::PlanProgression,
            TestCategory::Database,
            TestCategory::Repository,
            TestCategory::Sync,
            TestCategory::Security,
            TestCategory::AiSchema,
            TestCategory::Widget,
            TestCategory::Navigation,
            TestCategory::Integration,
            TestCategory::Gps,
            TestCategory::Background,
            TestCategory::Wearable,
            TestCategory::Music,
            TestCategory::Notifications,
            TestCategory::ErrorRecovery,
        ] {
            assert!(!cat.label().is_empty());
        }
    }

    #[test]
    fn category_default_layer_is_correct() {
        assert_eq!(TestCategory::Distance.default_layer(), TestLayer::Unit);
        assert_eq!(TestCategory::Database.default_layer(), TestLayer::Database);
        assert_eq!(TestCategory::Repository.default_layer(), TestLayer::Repository);
        assert_eq!(TestCategory::Sync.default_layer(), TestLayer::Sync);
        assert_eq!(TestCategory::Security.default_layer(), TestLayer::SecurityRules);
        assert_eq!(TestCategory::AiSchema.default_layer(), TestLayer::AiSchema);
        assert_eq!(TestCategory::Widget.default_layer(), TestLayer::Widget);
        assert_eq!(TestCategory::Navigation.default_layer(), TestLayer::Navigation);
        assert_eq!(TestCategory::Integration.default_layer(), TestLayer::Integration);
    }

    #[test]
    fn category_serializes_round_trip() {
        for cat in [
            TestCategory::Distance,
            TestCategory::Pace,
            TestCategory::Speed,
            TestCategory::Calories,
            TestCategory::Timing,
            TestCategory::PlanProgression,
            TestCategory::Database,
            TestCategory::Repository,
            TestCategory::Sync,
            TestCategory::Security,
            TestCategory::AiSchema,
            TestCategory::Widget,
            TestCategory::Navigation,
            TestCategory::Integration,
            TestCategory::Gps,
            TestCategory::Background,
            TestCategory::Wearable,
            TestCategory::Music,
            TestCategory::Notifications,
            TestCategory::ErrorRecovery,
        ] {
            let json = serde_json::to_string(&cat).unwrap();
            let back: TestCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(cat, back);
        }
    }

    // ── TestStatus tests ────────────────────────────────────────────

    #[test]
    fn status_labels_are_non_empty() {
        for status in [
            TestStatus::Pending,
            TestStatus::Running,
            TestStatus::Passed,
            TestStatus::Failed,
            TestStatus::Skipped,
            TestStatus::Flaky,
        ] {
            assert!(!status.label().is_empty());
        }
    }

    #[test]
    fn status_is_passing() {
        assert!(TestStatus::Passed.is_passing());
        assert!(TestStatus::Flaky.is_passing());
        assert!(!TestStatus::Failed.is_passing());
        assert!(!TestStatus::Skipped.is_passing());
        assert!(!TestStatus::Pending.is_passing());
    }

    #[test]
    fn status_is_terminal() {
        assert!(TestStatus::Passed.is_terminal());
        assert!(TestStatus::Failed.is_terminal());
        assert!(TestStatus::Skipped.is_terminal());
        assert!(TestStatus::Flaky.is_terminal());
        assert!(!TestStatus::Pending.is_terminal());
        assert!(!TestStatus::Running.is_terminal());
    }

    #[test]
    fn status_is_failure() {
        assert!(TestStatus::Failed.is_failure());
        assert!(!TestStatus::Passed.is_failure());
        assert!(!TestStatus::Skipped.is_failure());
        assert!(!TestStatus::Flaky.is_failure());
    }

    #[test]
    fn status_serializes_round_trip() {
        for status in [
            TestStatus::Pending,
            TestStatus::Running,
            TestStatus::Passed,
            TestStatus::Failed,
            TestStatus::Skipped,
            TestStatus::Flaky,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: TestStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    // ── TestSeverity tests ──────────────────────────────────────────

    #[test]
    fn severity_ordering() {
        assert!(TestSeverity::Critical > TestSeverity::High);
        assert!(TestSeverity::High > TestSeverity::Medium);
        assert!(TestSeverity::Medium > TestSeverity::Low);
    }

    #[test]
    fn severity_blocks_release() {
        assert!(TestSeverity::Critical.blocks_release());
        assert!(TestSeverity::High.blocks_release());
        assert!(!TestSeverity::Medium.blocks_release());
        assert!(!TestSeverity::Low.blocks_release());
    }

    #[test]
    fn severity_serializes_round_trip() {
        for sev in [
            TestSeverity::Low,
            TestSeverity::Medium,
            TestSeverity::High,
            TestSeverity::Critical,
        ] {
            let json = serde_json::to_string(&sev).unwrap();
            let back: TestSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(sev, back);
        }
    }

    // ── TestConfig tests ────────────────────────────────────────────

    #[test]
    fn config_new_sets_defaults() {
        let config = TestConfig::new(
            "test1",
            "Test 1",
            TestLayer::Unit,
            TestCategory::Distance,
        );
        assert_eq!(config.id, "test1");
        assert_eq!(config.name, "Test 1");
        assert_eq!(config.layer, TestLayer::Unit);
        assert_eq!(config.category, TestCategory::Distance);
        assert_eq!(config.timeout_ms, 30_000);
        assert_eq!(config.retry_count, 0);
        assert_eq!(config.severity, TestSeverity::Medium);
        assert!(config.tags.is_empty());
        assert!(!config.requires_device);
    }

    #[test]
    fn config_builder_methods() {
        let config = TestConfig::new("t2", "T2", TestLayer::Integration, TestCategory::Gps)
            .with_description("A test")
            .with_timeout(60_000)
            .with_retries(3)
            .with_severity(TestSeverity::Critical)
            .with_tag("release-blocker");
        assert_eq!(config.description, "A test");
        assert_eq!(config.timeout_ms, 60_000);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.severity, TestSeverity::Critical);
        assert!(config.has_tag("release-blocker"));
        assert!(!config.has_tag("nonexistent"));
        assert!(config.requires_device);
    }

    #[test]
    fn config_serializes_round_trip() {
        let config = TestConfig::new("t3", "T3", TestLayer::Unit, TestCategory::Pace)
            .with_tag("fast");
        let json = serde_json::to_string(&config).unwrap();
        let back: TestConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    // ── TestResult tests ───────────────────────────────────────────

    #[test]
    fn result_passed_constructor() {
        let config = TestConfig::new("t4", "T4", TestLayer::Unit, TestCategory::Distance);
        let result = TestResult::passed(config, 150, 5);
        assert_eq!(result.status, TestStatus::Passed);
        assert_eq!(result.duration_ms, 150);
        assert_eq!(result.assertions_passed, 5);
        assert_eq!(result.assertions_failed, 0);
        assert!(result.error_message.is_none());
        assert!(result.is_passing());
        assert!(!result.is_blocking());
    }

    #[test]
    fn result_failed_constructor() {
        let config = TestConfig::new("t5", "T5", TestLayer::Unit, TestCategory::Distance)
            .with_severity(TestSeverity::Critical);
        let result = TestResult::failed(
            config,
            200,
            "assertion failed: expected 100, got 50",
            3,
            2,
            1,
        );
        assert_eq!(result.status, TestStatus::Failed);
        assert_eq!(result.duration_ms, 200);
        assert_eq!(result.error_message, Some("assertion failed: expected 100, got 50".to_string()));
        assert_eq!(result.assertions_passed, 3);
        assert_eq!(result.assertions_failed, 2);
        assert_eq!(result.retries_used, 1);
        assert!(!result.is_passing());
        assert!(result.is_blocking());
    }

    #[test]
    fn result_skipped_constructor() {
        let config = TestConfig::new("t6", "T6", TestLayer::Unit, TestCategory::Distance);
        let result = TestResult::skipped(config, "prerequisite failed");
        assert_eq!(result.status, TestStatus::Skipped);
        assert_eq!(result.error_message, Some("prerequisite failed".to_string()));
        assert!(!result.is_passing());
        assert!(!result.is_blocking());
    }

    #[test]
    fn result_serializes_round_trip() {
        let config = TestConfig::new("t7", "T7", TestLayer::Unit, TestCategory::Pace);
        let result = TestResult::passed(config, 100, 3);
        let json = serde_json::to_string(&result).unwrap();
        let back: TestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }

    // ── TestSuite tests ─────────────────────────────────────────────

    #[test]
    fn suite_add_and_count() {
        let mut suite = TestSuite::new("suite1", TestLayer::Unit);
        assert_eq!(suite.total(), 0);

        suite.add(TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance));
        suite.add(TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace));
        assert_eq!(suite.total(), 2);
    }

    #[test]
    fn suite_add_all() {
        let mut suite = TestSuite::new("suite2", TestLayer::Unit);
        suite.add_all(vec![
            TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance),
            TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace),
            TestConfig::new("t3", "T3", TestLayer::Unit, TestCategory::Speed),
        ]);
        assert_eq!(suite.total(), 3);
    }

    #[test]
    fn suite_results_counts() {
        let mut suite = TestSuite::new("suite3", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace);
        let c3 = TestConfig::new("t3", "T3", TestLayer::Unit, TestCategory::Speed);

        suite.configs = vec![c1.clone(), c2.clone(), c3.clone()];
        suite.results = vec![
            TestResult::passed(c1, 100, 2),
            TestResult::failed(c2, 200, "error", 1, 1, 0),
            TestResult::skipped(c3, "dep failed"),
        ];

        assert_eq!(suite.passed(), 1);
        assert_eq!(suite.failed(), 1);
        assert_eq!(suite.skipped(), 1);
        assert_eq!(suite.total(), 3);
    }

    #[test]
    fn suite_pass_rate() {
        let mut suite = TestSuite::new("suite4", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace);

        suite.configs = vec![c1.clone(), c2.clone()];
        suite.results = vec![
            TestResult::passed(c1, 100, 2),
            TestResult::passed(c2, 100, 2),
        ];
        assert_eq!(suite.pass_rate(), 1.0);

        let c3 = TestConfig::new("t3", "T3", TestLayer::Unit, TestCategory::Speed);
        suite.configs.push(c3.clone());
        suite.results.push(TestResult::failed(c3, 200, "err", 0, 1, 0));
        assert!((suite.pass_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn suite_pass_rate_empty_is_zero() {
        let suite = TestSuite::new("suite5", TestLayer::Unit);
        assert_eq!(suite.pass_rate(), 0.0);
    }

    #[test]
    fn suite_all_passed() {
        let mut suite = TestSuite::new("suite6", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace);

        suite.configs = vec![c1.clone(), c2.clone()];
        suite.results = vec![
            TestResult::passed(c1, 100, 2),
            TestResult::passed(c2, 100, 2),
        ];
        assert!(suite.all_passed());

        // Not all results yet
        suite.results.pop();
        assert!(!suite.all_passed());
    }

    #[test]
    fn suite_has_blocking_failures() {
        let mut suite = TestSuite::new("suite7", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance)
            .with_severity(TestSeverity::Critical);
        suite.configs = vec![c1.clone()];
        suite.results = vec![TestResult::failed(c1, 100, "err", 0, 1, 0)];
        assert!(suite.has_blocking_failures());

        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Distance)
            .with_severity(TestSeverity::Low);
        let mut suite2 = TestSuite::new("suite8", TestLayer::Unit);
        suite2.configs = vec![c2.clone()];
        suite2.results = vec![TestResult::failed(c2, 100, "err", 0, 1, 0)];
        assert!(!suite2.has_blocking_failures());
    }

    #[test]
    fn suite_serializes_round_trip() {
        let mut suite = TestSuite::new("suite9", TestLayer::Unit);
        suite.add(TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance));
        let json = serde_json::to_string(&suite).unwrap();
        let back: TestSuite = serde_json::from_str(&json).unwrap();
        assert_eq!(suite, back);
    }

    // ── DeviceProfile tests ─────────────────────────────────────────

    #[test]
    fn device_labels_are_non_empty() {
        for dev in [
            DeviceProfile::LowEndPhone,
            DeviceProfile::FlagshipPhone,
            DeviceProfile::Tablet,
            DeviceProfile::WatchUltra,
        ] {
            assert!(!dev.label().is_empty());
        }
    }

    #[test]
    fn device_is_phone_and_wearable() {
        assert!(DeviceProfile::LowEndPhone.is_phone());
        assert!(DeviceProfile::FlagshipPhone.is_phone());
        assert!(!DeviceProfile::Tablet.is_phone());
        assert!(!DeviceProfile::WatchUltra.is_phone());
        assert!(DeviceProfile::WatchUltra.is_wearable());
        assert!(!DeviceProfile::FlagshipPhone.is_wearable());
    }

    #[test]
    fn device_serializes_round_trip() {
        for dev in [
            DeviceProfile::LowEndPhone,
            DeviceProfile::FlagshipPhone,
            DeviceProfile::Tablet,
            DeviceProfile::WatchUltra,
        ] {
            let json = serde_json::to_string(&dev).unwrap();
            let back: DeviceProfile = serde_json::from_str(&json).unwrap();
            assert_eq!(dev, back);
        }
    }

    // ── NetworkState tests ──────────────────────────────────────────

    #[test]
    fn network_labels_and_availability() {
        assert_eq!(NetworkState::Online.label(), "Online");
        assert_eq!(NetworkState::Offline.label(), "Offline");
        assert!(NetworkState::Online.is_available());
        assert!(NetworkState::Weak.is_available());
        assert!(NetworkState::Intermittent.is_available());
        assert!(!NetworkState::Offline.is_available());
    }

    #[test]
    fn network_serializes_round_trip() {
        for net in [
            NetworkState::Online,
            NetworkState::Weak,
            NetworkState::Offline,
            NetworkState::Intermittent,
        ] {
            let json = serde_json::to_string(&net).unwrap();
            let back: NetworkState = serde_json::from_str(&json).unwrap();
            assert_eq!(net, back);
        }
    }

    // ── GpsQuality tests ────────────────────────────────────────────

    #[test]
    fn gps_quality_labels_and_usability() {
        assert_eq!(GpsQuality::NoFix.label(), "No fix");
        assert_eq!(GpsQuality::Excellent.label(), "Excellent");
        assert!(GpsQuality::Good.is_usable());
        assert!(GpsQuality::Excellent.is_usable());
        assert!(!GpsQuality::Poor.is_usable());
        assert!(!GpsQuality::NoFix.is_usable());
    }

    #[test]
    fn gps_quality_serializes_round_trip() {
        for q in [
            GpsQuality::NoFix,
            GpsQuality::Poor,
            GpsQuality::Good,
            GpsQuality::Excellent,
        ] {
            let json = serde_json::to_string(&q).unwrap();
            let back: GpsQuality = serde_json::from_str(&json).unwrap();
            assert_eq!(q, back);
        }
    }

    // ── LocationType tests ──────────────────────────────────────────

    #[test]
    fn location_labels_and_default_gps() {
        assert_eq!(LocationType::Urban.label(), "Urban");
        assert_eq!(LocationType::Rural.default_gps_quality(), GpsQuality::Excellent);
        assert_eq!(LocationType::Urban.default_gps_quality(), GpsQuality::Good);
        assert_eq!(LocationType::Indoor.default_gps_quality(), GpsQuality::Poor);
        assert_eq!(LocationType::DeadZone.default_gps_quality(), GpsQuality::NoFix);
    }

    #[test]
    fn location_serializes_round_trip() {
        for loc in [
            LocationType::Indoor,
            LocationType::Urban,
            LocationType::Rural,
            LocationType::DeadZone,
        ] {
            let json = serde_json::to_string(&loc).unwrap();
            let back: LocationType = serde_json::from_str(&json).unwrap();
            assert_eq!(loc, back);
        }
    }

    // ── TestEnvironment tests ──────────────────────────────────────

    #[test]
    fn env_for_device_defaults() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        assert_eq!(env.device, DeviceProfile::FlagshipPhone);
        assert_eq!(env.os_version, "14");
        assert!((env.screen_size_inches - 6.7).abs() < 0.01);
        assert!(!env.has_watch);
        assert_eq!(env.gps_quality, GpsQuality::Good);
        assert_eq!(env.network_state, NetworkState::Online);
        assert_eq!(env.battery_level, 80);
        assert_eq!(env.location_type, LocationType::Urban);
        assert!(!env.screen_off);
        assert!(!env.battery_saver);
    }

    #[test]
    fn env_for_watch_has_watch_true() {
        let env = TestEnvironment::for_device(DeviceProfile::WatchUltra);
        assert!(env.has_watch);
        assert!((env.screen_size_inches - 1.5).abs() < 0.01);
    }

    #[test]
    fn env_builder_methods() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone)
            .with_gps(GpsQuality::Poor)
            .with_network(NetworkState::Offline)
            .with_battery(20)
            .with_location(LocationType::Rural)
            .with_screen_off(true)
            .with_battery_saver(true)
            .with_force_stopped(true)
            .with_phone_restarted(true);
        assert_eq!(env.gps_quality, GpsQuality::Poor);
        assert_eq!(env.network_state, NetworkState::Offline);
        assert_eq!(env.battery_level, 20);
        assert_eq!(env.location_type, LocationType::Rural);
        assert!(env.screen_off);
        assert!(env.battery_saver);
        assert!(env.force_stopped);
        assert!(env.phone_restarted);
    }

    #[test]
    fn env_description_is_non_empty() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        assert!(!env.description().is_empty());
        assert!(env.description().contains("Flagship phone"));
    }

    #[test]
    fn env_serializes_round_trip() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone)
            .with_gps(GpsQuality::Excellent)
            .with_battery(50);
        let json = serde_json::to_string(&env).unwrap();
        let back: TestEnvironment = serde_json::from_str(&json).unwrap();
        assert_eq!(env, back);
    }

    // ── RealDeviceScenario tests ────────────────────────────────────

    #[test]
    fn scenario_new_defaults() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s1", "Scenario 1", DeviceProfile::FlagshipPhone, env);
        assert_eq!(scenario.id, "s1");
        assert_eq!(scenario.name, "Scenario 1");
        assert_eq!(scenario.device, DeviceProfile::FlagshipPhone);
        assert!(scenario.description.is_empty());
        assert!(scenario.expected_behaviors.is_empty());
        assert_eq!(scenario.duration_minutes, 30);
        assert_eq!(scenario.severity, TestSeverity::High);
    }

    #[test]
    fn scenario_builder_methods() {
        let env = TestEnvironment::for_device(DeviceProfile::LowEndPhone);
        let scenario = RealDeviceScenario::new("s2", "S2", DeviceProfile::LowEndPhone, env)
            .with_description("A test scenario")
            .with_behavior("App launches")
            .with_behavior("GPS works")
            .with_duration(60)
            .with_severity(TestSeverity::Critical);
        assert_eq!(scenario.description, "A test scenario");
        assert_eq!(scenario.expected_behaviors.len(), 2);
        assert_eq!(scenario.duration_minutes, 60);
        assert_eq!(scenario.severity, TestSeverity::Critical);
    }

    #[test]
    fn scenario_serializes_round_trip() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s3", "S3", DeviceProfile::FlagshipPhone, env)
            .with_behavior("test");
        let json = serde_json::to_string(&scenario).unwrap();
        let back: RealDeviceScenario = serde_json::from_str(&json).unwrap();
        assert_eq!(scenario, back);
    }

    // ── ScenarioResult tests ────────────────────────────────────────

    #[test]
    fn scenario_result_passed() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s1", "S1", DeviceProfile::FlagshipPhone, env)
            .with_behavior("b1")
            .with_behavior("b2");
        let result = ScenarioResult::passed(scenario, 60000, 2);
        assert!(result.is_passing());
        assert_eq!(result.behaviors_passed, 2);
        assert_eq!(result.behaviors_failed, 0);
        assert_eq!(result.behavior_pass_rate(), 1.0);
    }

    #[test]
    fn scenario_result_failed() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s2", "S2", DeviceProfile::FlagshipPhone, env)
            .with_behavior("b1")
            .with_behavior("b2")
            .with_behavior("b3");
        let result = ScenarioResult::failed(scenario, 60000, "GPS failed", 1, 2);
        assert!(!result.is_passing());
        assert_eq!(result.behaviors_passed, 1);
        assert_eq!(result.behaviors_failed, 2);
        assert!((result.behavior_pass_rate() - 0.3333).abs() < 0.01);
    }

    #[test]
    fn scenario_result_pass_rate_no_behaviors() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s3", "S3", DeviceProfile::FlagshipPhone, env);
        let result = ScenarioResult::passed(scenario, 100, 0);
        assert_eq!(result.behavior_pass_rate(), 1.0);
    }

    // ── TestReport tests ────────────────────────────────────────────

    #[test]
    fn report_empty() {
        let report = TestReport::new("empty_report");
        assert_eq!(report.total_tests(), 0);
        assert_eq!(report.total_results(), 0);
        assert_eq!(report.passed(), 0);
        assert_eq!(report.failed(), 0);
        assert_eq!(report.test_pass_rate(), 0.0);
        assert!(!report.is_release_ready());
    }

    #[test]
    fn report_with_passing_suite() {
        let mut report = TestReport::new("passing");
        let mut suite = TestSuite::new("s1", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace);
        suite.configs = vec![c1.clone(), c2.clone()];
        suite.results = vec![
            TestResult::passed(c1, 100, 2),
            TestResult::passed(c2, 100, 2),
        ];
        report.add_suite(suite);
        assert_eq!(report.total_tests(), 2);
        assert_eq!(report.total_results(), 2);
        assert_eq!(report.passed(), 2);
        assert_eq!(report.failed(), 0);
        assert_eq!(report.test_pass_rate(), 1.0);
        assert!(report.is_release_ready());
    }

    #[test]
    fn report_with_failing_suite() {
        let mut report = TestReport::new("failing");
        let mut suite = TestSuite::new("s1", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance)
            .with_severity(TestSeverity::Critical);
        suite.configs = vec![c1.clone()];
        suite.results = vec![TestResult::failed(c1, 100, "err", 0, 1, 0)];
        report.add_suite(suite);
        assert_eq!(report.passed(), 0);
        assert_eq!(report.failed(), 1);
        assert!(report.has_blocking_failures());
        assert!(!report.is_release_ready());
    }

    #[test]
    fn report_summary_non_empty() {
        let report = TestReport::new("summary_test");
        let summary = report.summary();
        assert!(!summary.is_empty());
        assert!(summary.contains("summary_test"));
    }

    #[test]
    fn report_with_scenarios() {
        let mut report = TestReport::new("scenario_report");
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s1", "S1", DeviceProfile::FlagshipPhone, env);
        report.add_scenario_result(ScenarioResult::passed(scenario, 60000, 3));
        assert_eq!(report.total_scenarios(), 1);
        assert_eq!(report.scenarios_passed(), 1);
        assert_eq!(report.scenario_pass_rate(), 1.0);
    }

    // ── Coverage tests ───────────────────────────────────────────────

    #[test]
    fn coverage_by_layer_empty_report() {
        let report = TestReport::new("empty");
        let metrics = compute_coverage_by_layer(&report);
        assert!(metrics.is_empty());
    }

    #[test]
    fn coverage_by_layer_with_suite() {
        let mut report = TestReport::new("coverage");
        let mut suite = TestSuite::new("s1", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace);
        suite.configs = vec![c1.clone(), c2.clone()];
        suite.results = vec![
            TestResult::passed(c1, 100, 2),
            TestResult::failed(c2, 100, "err", 0, 1, 0),
        ];
        report.add_suite(suite);

        let metrics = compute_coverage_by_layer(&report);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "Unit");
        assert_eq!(metrics[0].total, 2);
        assert_eq!(metrics[0].passed, 1);
        assert_eq!(metrics[0].failed, 1);
        assert!((metrics[0].pass_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn coverage_by_category_empty_report() {
        let report = TestReport::new("empty");
        let metrics = compute_coverage_by_category(&report);
        assert!(metrics.is_empty());
    }

    #[test]
    fn coverage_by_category_with_suite() {
        let mut report = TestReport::new("coverage");
        let mut suite = TestSuite::new("s1", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace);
        suite.configs = vec![c1.clone(), c2.clone()];
        suite.results = vec![
            TestResult::passed(c1, 100, 2),
            TestResult::passed(c2, 100, 2),
        ];
        report.add_suite(suite);

        let metrics = compute_coverage_by_category(&report);
        assert_eq!(metrics.len(), 2);
        let distance = metrics.iter().find(|m| m.name == "Distance").unwrap();
        assert_eq!(distance.passed, 1);
        assert_eq!(distance.pass_rate, 1.0);
    }

    // ── Suite builder tests ─────────────────────────────────────────

    #[test]
    fn build_unit_test_suite_has_tests() {
        let suite = build_unit_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::Unit);
        // Check that it includes distance, pace, speed, calories, timing, plan
        let categories: Vec<TestCategory> = suite.configs.iter().map(|c| c.category).collect();
        assert!(categories.contains(&TestCategory::Distance));
        assert!(categories.contains(&TestCategory::Pace));
        assert!(categories.contains(&TestCategory::Speed));
        assert!(categories.contains(&TestCategory::Calories));
        assert!(categories.contains(&TestCategory::Timing));
        assert!(categories.contains(&TestCategory::PlanProgression));
    }

    #[test]
    fn build_database_test_suite_has_tests() {
        let suite = build_database_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::Database);
    }

    #[test]
    fn build_sync_test_suite_has_tests() {
        let suite = build_sync_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::Sync);
    }

    #[test]
    fn build_security_rules_test_suite_has_tests() {
        let suite = build_security_rules_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::SecurityRules);
    }

    #[test]
    fn build_ai_schema_test_suite_has_tests() {
        let suite = build_ai_schema_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::AiSchema);
    }

    #[test]
    fn build_widget_test_suite_has_tests() {
        let suite = build_widget_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::Widget);
    }

    #[test]
    fn build_navigation_test_suite_has_tests() {
        let suite = build_navigation_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::Navigation);
    }

    #[test]
    fn build_integration_test_suite_has_tests() {
        let suite = build_integration_test_suite();
        assert!(!suite.configs.is_empty());
        assert_eq!(suite.layer, TestLayer::Integration);
        // Integration tests should require a device
        assert!(suite.configs.iter().all(|c| c.requires_device));
    }

    // ── Real-device suite builder tests ─────────────────────────────

    #[test]
    fn build_real_device_suite_has_scenarios() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.len() >= 16); // All required scenarios
    }

    #[test]
    fn build_real_device_suite_includes_low_end_phone() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.device == DeviceProfile::LowEndPhone));
    }

    #[test]
    fn build_real_device_suite_includes_flagship_phone() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.device == DeviceProfile::FlagshipPhone));
    }

    #[test]
    fn build_real_device_suite_includes_tablet() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.device == DeviceProfile::Tablet));
    }

    #[test]
    fn build_real_device_suite_includes_watch_ultra() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.device == DeviceProfile::WatchUltra));
    }

    #[test]
    fn build_real_device_suite_includes_dead_zone() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.environment.network_state == NetworkState::Offline));
    }

    #[test]
    fn build_real_device_suite_includes_screen_off() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.environment.screen_off));
    }

    #[test]
    fn build_real_device_suite_includes_battery_saver() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.environment.battery_saver));
    }

    #[test]
    fn build_real_device_suite_includes_force_stopped() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.environment.force_stopped));
    }

    #[test]
    fn build_real_device_suite_includes_phone_restarted() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.environment.phone_restarted));
    }

    #[test]
    fn build_real_device_suite_includes_long_workout() {
        let scenarios = build_real_device_suite();
        assert!(scenarios.iter().any(|s| s.duration_minutes >= 180));
    }

    // ── TestRegistry tests ──────────────────────────────────────────

    #[test]
    fn registry_new_is_empty() {
        let registry = TestRegistry::new();
        assert!(registry.suites.is_empty());
        assert!(registry.scenarios.is_empty());
        assert!(registry.scenario_results.is_empty());
        assert_eq!(registry.total_tests(), 0);
    }

    #[test]
    fn registry_add_suite_and_scenario() {
        let mut registry = TestRegistry::new();
        registry.add_suite(build_unit_test_suite());
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        registry.add_scenario(RealDeviceScenario::new("s1", "S1", DeviceProfile::FlagshipPhone, env));
        assert_eq!(registry.suites.len(), 1);
        assert_eq!(registry.scenarios.len(), 1);
        assert!(registry.total_tests() > 0);
    }

    #[test]
    fn registry_suite_and_scenario_names() {
        let mut registry = TestRegistry::new();
        registry.add_suite(build_unit_test_suite());
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        registry.add_scenario(RealDeviceScenario::new("rd1", "RD1", DeviceProfile::FlagshipPhone, env));
        let suite_names = registry.suite_names();
        assert!(suite_names.contains(&"unit_tests".to_string()));
        let scenario_names = registry.scenario_names();
        assert!(scenario_names.contains(&"RD1".to_string()));
    }

    #[test]
    fn registry_pass_rate_empty_is_zero() {
        let registry = TestRegistry::new();
        assert_eq!(registry.pass_rate(), 0.0);
    }

    #[test]
    fn registry_pass_rate_with_results() {
        let mut registry = TestRegistry::new();
        let mut suite = TestSuite::new("s1", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        let c2 = TestConfig::new("t2", "T2", TestLayer::Unit, TestCategory::Pace);
        suite.configs = vec![c1.clone(), c2.clone()];
        suite.results = vec![
            TestResult::passed(c1, 100, 2),
            TestResult::passed(c2, 100, 2),
        ];
        registry.add_suite(suite);
        assert_eq!(registry.pass_rate(), 1.0);
    }

    #[test]
    fn registry_clear_results() {
        let mut registry = TestRegistry::new();
        let mut suite = TestSuite::new("s1", TestLayer::Unit);
        let c1 = TestConfig::new("t1", "T1", TestLayer::Unit, TestCategory::Distance);
        suite.configs = vec![c1.clone()];
        suite.results = vec![TestResult::passed(c1, 100, 2)];
        registry.add_suite(suite);

        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s1", "S1", DeviceProfile::FlagshipPhone, env);
        registry.add_scenario_result(ScenarioResult::passed(scenario, 100, 1));

        assert_eq!(registry.total_results(), 1);
        assert_eq!(registry.scenario_results.len(), 1);

        registry.clear_results();
        assert_eq!(registry.total_results(), 0);
        assert_eq!(registry.scenario_results.len(), 0);
        // Configs should still be there
        assert_eq!(registry.total_tests(), 1);
    }

    #[test]
    fn registry_serializes_round_trip() {
        let mut registry = TestRegistry::new();
        registry.add_suite(build_unit_test_suite());
        let json = serde_json::to_string(&registry).unwrap();
        let back: TestRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(registry.suites.len(), back.suites.len());
    }

    // ── build_full_registry tests ───────────────────────────────────

    #[test]
    fn build_full_registry_has_all_suites() {
        let registry = build_full_registry();
        // Should have all 9 test layers
        assert_eq!(registry.suites.len(), 9);
        // Should have real-device scenarios
        assert!(registry.scenarios.len() >= 16);
        assert!(registry.total_tests() > 0);
    }

    #[test]
    fn build_full_registry_suite_names() {
        let registry = build_full_registry();
        let names = registry.suite_names();
        assert!(names.contains(&"unit_tests".to_string()));
        assert!(names.contains(&"database_tests".to_string()));
        assert!(names.contains(&"sync_tests".to_string()));
        assert!(names.contains(&"integration_tests".to_string()));
    }

    // ── build_report tests ──────────────────────────────────────────

    #[test]
    fn build_report_from_registry() {
        let registry = build_full_registry();
        let report = build_report("test_report", &registry);
        assert_eq!(report.name, "test_report");
        assert_eq!(report.suites.len(), registry.suites.len());
        assert_eq!(report.total_tests(), registry.total_tests());
    }

    // ── run_suite / run_scenario tests ──────────────────────────────

    #[test]
    fn run_suite_produces_passing_results() {
        let suite = build_unit_test_suite();
        let result_suite = run_suite(&suite);
        assert_eq!(result_suite.configs.len(), suite.configs.len());
        assert_eq!(result_suite.results.len(), suite.configs.len());
        assert!(result_suite.results.iter().all(|r| r.status == TestStatus::Passed));
    }

    #[test]
    fn run_scenario_produces_passing_result() {
        let env = TestEnvironment::for_device(DeviceProfile::FlagshipPhone);
        let scenario = RealDeviceScenario::new("s1", "S1", DeviceProfile::FlagshipPhone, env)
            .with_behavior("b1")
            .with_behavior("b2");
        let result = run_scenario(&scenario);
        assert!(result.is_passing());
        assert_eq!(result.behaviors_passed, 2);
    }

    // ── Integration with existing modules (sanity) ──────────────────

    #[test]
    fn full_registry_coverage_by_layer_matches() {
        let registry = build_full_registry();
        let report = build_report("full", &registry);
        let metrics = compute_coverage_by_layer(&report);
        // Should have metrics for all 9 layers
        assert_eq!(metrics.len(), 9);
        // Each should have total > 0
        assert!(metrics.iter().all(|m| m.total > 0));
    }
}
