# §15 Testing — Rust Engine Implementation

## Rust Engine
- [ ] Create `rust/stride_engine/src/engine/testing.rs` with:
  - TestLayer enum (unit, database, repository, sync, security_rules, ai_schema, widget, navigation, integration)
  - TestCategory enum (distance, pace, speed, calories, timing, plan_progression, database, repository, sync, security, ai_schema, widget, navigation, integration)
  - TestStatus enum (pending, running, passed, failed, skipped, flaky)
  - TestSeverity enum (critical, high, medium, low)
  - TestConfig (name, layer, category, description, timeout_ms, retry_count, tags, severity)
  - TestResult (config, status, duration_ms, error_message, timestamp, assertions_passed, assertions_failed)
  - TestSuite (name, configs, results, total, passed, failed, skipped)
  - TestEnvironment (device_type, os_version, screen_size, has_watch, gps_quality, network_state, battery_level, location_type)
  - DeviceProfile (low_end_phone, flagship_phone, tablet, watch_ultra)
  - RealDeviceScenario (name, description, device_profile, environment, expected_behaviors, duration_minutes)
  - TestScenarioRunner / run_scenario() — run a real-device scenario and produce results
  - TestSuiteRunner / run_suite() — run a suite of test configs
  - TestRegistry — collection of all test suites and their results
  - TestReport — aggregate report with summary stats, pass rate, coverage
  - build_test_suite() / build_integration_test_suite() / build_real_device_suite()
  - compute_pass_rate() / compute_coverage()
  - 80+ unit tests

## FFI Layer
- [ ] Add `pub mod testing;` to mod.rs
- [ ] Add import block + FFI entry points to ffi.rs:
  1. stride_testing_run_suite
  2. stride_testing_run_scenario
  3. stride_testing_build_report
  4. stride_testing_get_config
  5. stride_testing_list_suites

## Dart Bindings
- [ ] Add typedefs + lookups + late final fields + wrapper methods to stride_engine_bindings.dart
- [ ] Add enums + classes to stride_models.dart
- [ ] Add ergonomic static methods to stride_engine_client.dart

## Verification & Push
- [ ] Compile Rust tests pass
- [ ] Verify Dart brace/paren/bracket balance
- [ ] Commit and push to GitHub
