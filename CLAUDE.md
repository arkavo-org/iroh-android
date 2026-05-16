# CLAUDE.md

Guidance for Claude Code working in this repo.

## What this is

Kotlin/Android bindings for Iroh blob storage. Sibling to `../iroh-swift/`:
same Rust ecosystem (iroh 0.98 / iroh-blobs 0.100), but the FFI uses JNI
(via the `jni` crate) instead of a C ABI, and Kotlin instead of Swift.

## Layout

```
iroh-android/
├── rust/                  Rust cdylib exporting Java_net_arkavo_iroh_* JNI fns
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs         module wiring
│       ├── jni_bridge.rs  JNI entry points (one per Kotlin native method)
│       └── node.rs        IrohNode core (put/get/lifecycle) — mirrors iroh-swift
├── kotlin/                Gradle Android library that bundles the .so + .kt
│   ├── build.gradle.kts
│   ├── settings.gradle.kts
│   └── src/main/
│       ├── AndroidManifest.xml
│       └── java/net/arkavo/iroh/
│           ├── IrohNode.kt       suspend API surface
│           ├── IrohConfig.kt
│           └── IrohException.kt
└── scripts/
    ├── build-aar.sh       cargo build (3 ABIs) → copy .so into jniLibs → gradle assembleRelease
    └── install-targets.sh rustup target add ...
```

## Build commands

```bash
# Rust checks (no NDK needed — uses host target)
cd rust && cargo check
cd rust && cargo test
cd rust && cargo fmt --check
cd rust && cargo clippy

# Cross-compile to Android targets (needs NDK)
./scripts/build-aar.sh

# Build only the Kotlin/Gradle part (requires .so already in jniLibs)
cd kotlin && ./gradlew assembleRelease
```

## JNI conventions

- Native methods declared in `net.arkavo.iroh.IrohNode` companion object map
  to `Java_net_arkavo_iroh_IrohNode_<name>` in `jni_bridge.rs`.
- Node handles are `Box<IrohNode>` raw pointers cast to `jlong`. Kotlin holds
  the handle as a `Long`; `close()` calls back into Rust to free the box.
- Errors thrown from Rust use `IrohException` (subclasses match Kotlin's
  sealed hierarchy: NodeUnavailable / PublishFailed / FetchFailed / TicketInvalid).
- Each `IrohNode` owns its own tokio Runtime — JNI calls are blocking and
  Kotlin wraps them with `withContext(Dispatchers.IO)`.

## Versioning

`VERSION` (root) is source of truth. Sync into:
- `rust/Cargo.toml` `package.version`
- `kotlin/build.gradle.kts` `version`
- README badges (if added later)

## Constraints

- No Ruby code (per user preference)
- Min SDK 28 (matches closurekb/android)
- ABIs: arm64-v8a, armeabi-v7a, x86_64
- Dual licensed: MIT + Apache 2.0
