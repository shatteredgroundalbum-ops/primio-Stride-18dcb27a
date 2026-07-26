# stride_engine

Core walking/running workout engine for **S.T.R.I.D.E.**, written in Rust
and exposed to the Flutter UI through a C-ABI FFI boundary consumed via
`dart:ffi`.

## What lives here

- `src/models/` — plain data types (points, sessions, splits, goals,
  checkpoints, sensor samples, coaching events, summaries). Canonical
  units: meters, meters/second, meters (elevation), kilograms, epoch
  milliseconds.
- `src/engine/` — 22 independently unit-tested subsystems (GPS filtering,
  distance, timekeeping, speed, pace, movement detection, auto-pause,
  walk/run/vehicle classification, steps/cadence, elevation, calories,
  heart rate, splits, route building + polyline encoding, goal tracking,
  coaching events, personal records, validation, permissions, battery/
  sampling policy, unit conversions, crash recovery) plus `controller.rs`,
  which wires all of them into one `WorkoutSessionController` per live
  workout.
- `src/ffi.rs` — the `extern "C"` boundary Dart calls into. Every
  function takes/returns JSON strings, is `catch_unwind`-guarded so a
  Rust panic can never crash the Flutter host process, and returns a
  uniform `{"ok": bool, "data"|"error": ...}` envelope.

## Why dart:ffi + JSON instead of flutter_rust_bridge

`flutter_rust_bridge`'s codegen toolchain has a much larger disk/tooling
footprint. Since every value crossing the boundary is already a
`serde`-serializable Rust struct, plain JSON strings over `dart:ffi` give
us the same safety (no hand-maintained struct layouts) with a much
lighter footprint and no codegen step to keep in sync.

## Building

Run all unit tests (153 tests across every module):

```bash
cargo test
```

### Android

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi \
    x86_64-linux-android i686-linux-android
cargo install cargo-ndk
./build_android.sh
```

This copies `libstride_engine.so` for each ABI into
`android/app/src/main/jniLibs/<abi>/`, which Gradle bundles into the APK
automatically — no manifest changes needed.

### iOS (must run on macOS)

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
./build_ios.sh
```

Produces `target/ios/StrideEngine.xcframework`. Add it to the Runner
Xcode target under *Frameworks, Libraries, and Embedded Content*. Because
it's a static library, the Dart side loads it via
`DynamicLibrary.process()` rather than `DynamicLibrary.open()` (see
`lib/native/stride_native_library.dart`).

### Linux / desktop dev builds

```bash
cargo build --release
# libstride_engine.so ends up in target/release/
```

## FFI contract (Dart side)

See `lib/native/`:

- `stride_native_library.dart` — platform-specific `DynamicLibrary`
  loading.
- `stride_engine_bindings.dart` — raw 1:1 `dart:ffi` signatures for every
  `extern "C"` function, plus JSON encode/decode and native string
  lifetime management (`stride_free_string` is called automatically for
  every returned pointer).
- `stride_models.dart` — typed Dart mirrors of the JSON payloads
  (`StrideWorkoutSession`, `StrideLiveUpdate`, `StrideWorkoutSummary`,
  etc.) so callers never touch raw maps.
- `stride_engine_client.dart` — the ergonomic per-workout facade
  (`StrideEngineClient.create(...)`, `.start()`, `.addLocationSample(...)`,
  `.tick(...)`, `.finish(...)`, `.dispose()`) that `WorkoutRecorder` uses.

`lib/services/workout_recorder.dart` now delegates every calculation
(distance, pace, speed, calories, elevation, splits, auto-pause,
walk/run classification, GPS filtering) to the native engine — it only
coordinates GPS/sensor input, SQLite persistence, and translating the
engine's `LiveUpdate`/`WorkoutSummary` back into the existing
`WalkingSession` model so the rest of the app is unaffected.

## Memory-safety contract across the FFI boundary

- Every `*mut c_char` returned by a `stride_*` function is heap-allocated
  by Rust (`CString::into_raw`) and **must** be freed by calling
  `stride_free_string` exactly once. The Dart bindings do this
  automatically inside `StrideEngineBindings._consume`, so nothing above
  that layer needs to think about it.
- `*const c_char` arguments passed *into* Rust (e.g. request JSON) remain
  owned by Dart; Rust only borrows them via `CStr` and never frees them.
  The Dart bindings free their own native argument allocations
  (`malloc.free`) right after each call.
- Every entry point is wrapped in `std::panic::catch_unwind`, so a bug in
  the engine surfaces as a normal `{"ok": false, "error": "internal_panic: ..."}`
  response instead of aborting the whole process — this is why the crate's
  release profile intentionally leaves `panic = "unwind"` (the default)
  rather than `"abort"`.
- Each live workout gets an opaque `i64` handle from `stride_create_session`;
  Dart never sees a raw Rust pointer. Call `stride_destroy_session` once
  the workout is finished/discarded and persisted, or the controller
  leaks for the life of the app process.
