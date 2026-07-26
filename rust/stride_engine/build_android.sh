#!/usr/bin/env bash
# Cross-compiles the stride_engine crate for every Android ABI Flutter
# ships by default and copies the resulting .so files into the Flutter
# project's jniLibs directories so they're bundled into the APK/AAB
# automatically by Gradle.
#
# Prerequisites (one-time setup):
#   rustup target add aarch64-linux-android armv7-linux-androideabi \
#       x86_64-linux-android i686-linux-android
#   cargo install cargo-ndk
#   Android NDK installed (set ANDROID_NDK_HOME) — the ndkVersion in
#   android/app/build.gradle.kts (27.3.13750724) must match.
#
# Usage:
#   ./build_android.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
JNI_LIBS_DIR="$PROJECT_ROOT/android/app/src/main/jniLibs"

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "cargo-ndk not found. Install with: cargo install cargo-ndk" >&2
  exit 1
fi

cd "$SCRIPT_DIR"

echo "Building stride_engine for Android (arm64-v8a, armeabi-v7a, x86_64)..."
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -o "$JNI_LIBS_DIR" \
  build --release

echo "Done. Libraries placed under: $JNI_LIBS_DIR"
find "$JNI_LIBS_DIR" -name "libstride_engine.so" -exec ls -la {} \;
