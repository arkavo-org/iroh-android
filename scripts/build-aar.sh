#!/usr/bin/env bash
# Build the iroh-android AAR.
#
# Steps:
#   1. cargo-ndk cross-compiles the Rust cdylib for the 3 Android ABIs
#      (arm64-v8a / armeabi-v7a / x86_64).
#   2. Compiled .so files are placed in kotlin/library/src/main/jniLibs/<ABI>/.
#   3. Gradle assembles the AAR which then contains the .so libs.
#
# Output: kotlin/library/build/outputs/aar/iroh-android-release.aar
#
# Requirements:
#   - Android NDK r25+ (set ANDROID_NDK_HOME, or have one under
#     $ANDROID_HOME/ndk/* or $HOME/Library/Android/sdk/ndk/*).
#   - Rust toolchain with the three Android targets installed (run
#     ./scripts/install-targets.sh once).
#   - cargo-ndk (install via `cargo install cargo-ndk`).
#   - JDK 17+ and an Android SDK (sdkmanager has installed the matching
#     platform; Gradle picks it up via ANDROID_HOME).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$PROJECT_ROOT/rust"
KOTLIN_DIR="$PROJECT_ROOT/kotlin"
JNI_LIBS_DIR="$KOTLIN_DIR/library/src/main/jniLibs"

# --- Locate NDK -------------------------------------------------------------
detect_ndk() {
    if [[ -n "${ANDROID_NDK_HOME:-}" && -d "$ANDROID_NDK_HOME" ]]; then
        echo "$ANDROID_NDK_HOME"
        return
    fi
    local candidates=(
        "${ANDROID_HOME:-}/ndk"
        "$HOME/Library/Android/sdk/ndk"
        "$HOME/Android/Sdk/ndk"
    )
    for base in "${candidates[@]}"; do
        if [[ -d "$base" ]]; then
            # Pick the lexicographically greatest (newest) version dir.
            local newest
            newest="$(ls -1 "$base" 2>/dev/null | sort -V | tail -n1 || true)"
            if [[ -n "$newest" && -d "$base/$newest" ]]; then
                echo "$base/$newest"
                return
            fi
        fi
    done
    echo ""
}

NDK="$(detect_ndk)"
if [[ -z "$NDK" ]]; then
    echo "error: Android NDK not found." >&2
    echo "  set ANDROID_NDK_HOME, or install via Android Studio's SDK Manager." >&2
    exit 1
fi
export ANDROID_NDK_HOME="$NDK"
echo "Using NDK: $ANDROID_NDK_HOME"

# --- Verify cargo-ndk -------------------------------------------------------
if ! command -v cargo-ndk >/dev/null 2>&1; then
    echo "error: cargo-ndk not installed. Run ./scripts/install-targets.sh" >&2
    exit 1
fi

# --- Cross-compile ----------------------------------------------------------
echo "Cleaning previous jniLibs..."
rm -rf "$JNI_LIBS_DIR"
mkdir -p "$JNI_LIBS_DIR"

echo "Building Rust cdylib for Android ABIs..."
(
    cd "$RUST_DIR"
    # cargo-ndk handles target sysroot/linker setup and copies .so files
    # into the right ABI subdirectories under -o.
    cargo ndk \
        --target aarch64-linux-android \
        --target armv7-linux-androideabi \
        --target x86_64-linux-android \
        --platform 28 \
        --output-dir "$JNI_LIBS_DIR" \
        build --release
)

echo "Pruning intermediate cargo build artifacts..."
# cargo-ndk's --output-dir greedily copies every .so under target/<abi>/release/
# deps/, including intermediate dependency dylibs (libiroh-*, libredb-*, etc.)
# that our crate does not actually link against (verified via DT_NEEDED).
# Keep only libiroh_android.so per ABI.
for abi in arm64-v8a armeabi-v7a x86_64; do
    abi_dir="$JNI_LIBS_DIR/$abi"
    [[ -d "$abi_dir" ]] || continue
    find "$abi_dir" -mindepth 1 -name "*.so" ! -name "libiroh_android.so" -delete
done

echo "Verifying .so layout..."
for abi in arm64-v8a armeabi-v7a x86_64; do
    so="$JNI_LIBS_DIR/$abi/libiroh_android.so"
    if [[ ! -f "$so" ]]; then
        echo "error: missing $so" >&2
        exit 1
    fi
    echo "  $(file "$so" | head -c 200)"
done

# --- Gradle AAR -------------------------------------------------------------
echo "Assembling AAR..."
(
    cd "$KOTLIN_DIR"
    if [[ ! -x ./gradlew ]]; then
        echo "note: ./gradlew not present in kotlin/. Falling back to system gradle." >&2
        gradle :iroh-android:assembleRelease
    else
        ./gradlew :iroh-android:assembleRelease
    fi
)

AAR="$KOTLIN_DIR/library/build/outputs/aar/iroh-android-release.aar"
if [[ -f "$AAR" ]]; then
    echo
    echo "Built: $AAR"
    ls -la "$AAR"
else
    echo "warning: expected AAR not found at $AAR" >&2
fi
