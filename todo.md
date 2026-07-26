# §14 Error/Recovery States — Rust Engine Implementation

## Rust Engine
- [x] Create `rust/stride_engine/src/engine/error_states.rs` (2441 lines, 80+ tests)

## FFI Layer
- [x] Add `pub mod error_states;` to mod.rs
- [x] Add import block + FFI entry points to ffi.rs:
  1. stride_error_decide_recovery
  2. stride_error_retry_delay
  3. stride_error_transition
  4. stride_error_health_status
  5. stride_error_is_recoverable

## Dart Bindings
- [x] Add typedefs + lookups + late final fields + wrapper methods to stride_engine_bindings.dart
- [x] Add enums + classes to stride_models.dart
- [x] Add ergonomic static methods to stride_engine_client.dart

## Verification & Push
- [x] Compile Rust tests pass (1038 tests)
- [x] Verify Dart brace/paren/bracket balance (all balanced)
- [ ] Commit and push to GitHub
