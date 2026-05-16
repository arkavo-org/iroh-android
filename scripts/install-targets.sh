#!/usr/bin/env bash
# Install rustup targets needed for Android cross-compilation.
set -euo pipefail

TARGETS=(
    aarch64-linux-android
    armv7-linux-androideabi
    x86_64-linux-android
)

for t in "${TARGETS[@]}"; do
    rustup target add "$t"
done

if ! command -v cargo-ndk >/dev/null 2>&1; then
    echo "Installing cargo-ndk..."
    cargo install cargo-ndk
fi

echo "Done. Targets installed: ${TARGETS[*]}"
