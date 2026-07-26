import 'dart:ffi';
import 'dart:io';

/// Loads the compiled `stride_engine` Rust crate as a dynamic library.
///
/// Platform packaging expectations:
///   - Android: `libstride_engine.so` per-ABI under
///     `android/app/src/main/jniLibs/<abi>/`. Gradle packages these into
///     the APK automatically; `DynamicLibrary.open` finds it by name.
///   - iOS/macOS: the `staticlib` build of the crate is linked directly
///     into the app binary (added as a linked library in the Xcode
///     project), so its symbols are already resolved in the running
///     process — `DynamicLibrary.process()` is used instead of `open()`.
///   - Linux desktop / this sandbox (for local dev & testing only):
///     `libstride_engine.so` next to the executable or on the library
///     search path (e.g. built via `cargo build --release` and copied
///     into the bundle's `lib/` directory at packaging time).
///   - Windows desktop: `stride_engine.dll`.
DynamicLibrary loadStrideEngineLibrary() {
  if (Platform.isAndroid) {
    return DynamicLibrary.open('libstride_engine.so');
  }
  if (Platform.isIOS || Platform.isMacOS) {
    return DynamicLibrary.process();
  }
  if (Platform.isLinux) {
    return DynamicLibrary.open('libstride_engine.so');
  }
  if (Platform.isWindows) {
    return DynamicLibrary.open('stride_engine.dll');
  }
  throw UnsupportedError(
    'stride_engine has no native library loading strategy for this platform',
  );
}
