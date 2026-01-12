#!/bin/bash
set -e

# Target Architectures
targets=("aarch64-apple-ios" "aarch64-apple-ios-sim" "x86_64-apple-ios")

for target in "${targets[@]}"; do
    echo "Building for $target..."
    rustup target add "$target"
    cargo build --release --target "$target"
done

# Create Fat Library for Simulator (Intel + Apple Silicon)
mkdir -p target/universal-sim/release
lipo -create \
    target/aarch64-apple-ios-sim/release/libtelco_core.a \
    target/x86_64-apple-ios/release/libtelco_core.a \
    -output target/universal-sim/release/libtelco_core.a

# Create XCFramework
rm -rf TelcoCore.xcframework
xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libtelco_core.a \
    -headers bindings/ \
    -library target/universal-sim/release/libtelco_core.a \
    -headers bindings/ \
    -output TelcoCore.xcframework

echo "Successfully generated TelcoCore.xcframework"
