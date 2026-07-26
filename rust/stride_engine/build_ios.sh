#!/usr/bin/env bash
# Cross-compiles the stride_engine crate as a static library for iOS
# device + simulator and combines them into an XCFramework Xcode can link
# directly. Run this from a macOS machine with Xcode command line tools
# installed (iOS cross-compilation cannot be done from this Linux sandbox).
#
# Prerequisites (one-time setup, on macOS):
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
#
# Usage (on macOS):
#   ./build_ios.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building stride_engine for iOS device (arm64)..."
cargo build --release --target aarch64-apple-ios

echo "Building stride_engine for iOS simulator (arm64, Apple Silicon Macs)..."
cargo build --release --target aarch64-apple-ios-sim

echo "Building stride_engine for iOS simulator (x86_64, Intel Macs)..."
cargo build --release --target x86_64-apple-ios

OUT_DIR="$SCRIPT_DIR/target/ios"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

# Combine the two simulator slices into one fat static lib.
lipo -create \
  target/aarch64-apple-ios-sim/release/libstride_engine.a \
  target/x86_64-apple-ios/release/libstride_engine.a \
  -output "$OUT_DIR/libstride_engine_sim.a"

cp target/aarch64-apple-ios/release/libstride_engine.a "$OUT_DIR/libstride_engine_device.a"

xcodebuild -create-xcframework \
  -library "$OUT_DIR/libstride_engine_device.a" \
  -library "$OUT_DIR/libstride_engine_sim.a" \
  -output "$OUT_DIR/StrideEngine.xcframework"

echo "Done. Add $OUT_DIR/StrideEngine.xcframework to the Xcode project"
echo "(Runner target -> General -> Frameworks, Libraries, and Embedded Content)."
