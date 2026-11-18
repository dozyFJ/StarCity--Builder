#!/usr/bin/env bash
set -euo pipefail
NDK="$ANDROID_HOME/ndk/27.2.12479018/toolchains/llvm/prebuilt/linux-x86_64"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK/bin/aarch64-linux-android26-clang"
export CC_aarch64_linux_android="$NDK/bin/aarch64-linux-android26-clang"
export CXX_aarch64_linux_android="$NDK/bin/aarch64-linux-android26-clang++"
export AR_aarch64_linux_android="$NDK/bin/llvm-ar"
export RUSTFLAGS="-C link-arg=-Wl,-z,max-page-size=16384"
cargo build --lib --release --target aarch64-linux-android
mkdir -p android-shell/app/src/main/jniLibs/arm64-v8a build
cp target/aarch64-linux-android/release/libstarcity_builder.so android-shell/app/src/main/jniLibs/arm64-v8a/
"$NDK/bin/llvm-strip" --strip-unneeded android-shell/app/src/main/jniLibs/arm64-v8a/libstarcity_builder.so
if "$NDK/bin/llvm-readelf" -d android-shell/app/src/main/jniLibs/arm64-v8a/libstarcity_builder.so | grep -q 'libc++_shared.so'; then
  cp "$NDK/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so" android-shell/app/src/main/jniLibs/arm64-v8a/
fi
if "$NDK/bin/llvm-readelf" -S android-shell/app/src/main/jniLibs/arm64-v8a/libstarcity_builder.so | grep -q '\.debug_info'; then exit 1; fi
ls -lh android-shell/app/src/main/jniLibs/arm64-v8a/
