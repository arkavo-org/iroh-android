# iroh-android

Android (Kotlin) bindings for Iroh blob storage. Parallel to `iroh-swift/`:
same Rust core (`iroh 0.98` / `iroh-blobs 0.100`), different FFI strategy.

| Layer              | iroh-swift               | iroh-android                |
| ------------------ | ------------------------ | --------------------------- |
| Surface            | Swift `actor IrohNode`   | Kotlin `class IrohNode`     |
| FFI mechanism      | C ABI + callbacks        | JNI (jni-rs) + blocking     |
| Async bridge       | `CheckedContinuation`    | `withContext(Dispatchers.IO)` |
| Packaging          | XCFramework              | AAR                         |

## Surface (MVP)

```kotlin
val node = IrohNode.create(
    storagePath = "/data/data/com.example/iroh",
    relayEnabled = true,
    customRelayUrl = null,
)
try {
    val ticket = node.put(bytes)              // suspend → String
    val out    = node.get(ticket)             // suspend → ByteArray
} finally {
    node.close()
}
```

Errors surface as `IrohException` (sealed: `NodeUnavailable`, `PublishFailed`,
`FetchFailed`, `TicketInvalid`).

## Build

Requires Android NDK (r25+) and the three Android Rust targets.

```bash
./scripts/install-targets.sh        # rustup target add aarch64/armv7/x86_64-linux-android
./scripts/build-aar.sh              # cross-compile .so + bundle AAR
# → kotlin/build/outputs/aar/iroh-android-release.aar
```

The script auto-detects NDK in this order:
1. `$ANDROID_NDK_HOME`
2. `$ANDROID_HOME/ndk/<latest>`
3. `$HOME/Library/Android/sdk/ndk/<latest>`

## Consuming via GitHub Packages

Tagged releases are published to GitHub Packages by `.github/workflows/release.yml`.
The published AAR bundles `libiroh_android.so` for `arm64-v8a`, `armeabi-v7a`,
and `x86_64`.

In the consumer's `settings.gradle.kts`:

```kotlin
dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
        maven {
            name = "GitHubPackagesIrohAndroid"
            url = uri("https://maven.pkg.github.com/arkavo-org/iroh-android")
            credentials {
                username = providers.gradleProperty("gpr.user").orNull
                    ?: System.getenv("GITHUB_ACTOR")
                password = providers.gradleProperty("gpr.token").orNull
                    ?: System.getenv("GITHUB_TOKEN")
            }
        }
    }
}
```

In the consumer's version catalog:

```toml
iroh-android = { module = "net.arkavo:iroh-android", version = "0.1.0" }
```

GitHub Packages requires authentication even for public packages. Local
developers need a personal access token with `read:packages` scope, exposed
as `GITHUB_TOKEN` (or set `gpr.user` / `gpr.token` in `~/.gradle/gradle.properties`).
CI uses the workflow's built-in `GITHUB_TOKEN`.

For local iteration across both repos, you can still composite-include this
project in the consumer's `settings.gradle.kts` to shadow the published artifact.

## Releasing

1. Bump `VERSION` (root) and `rust/Cargo.toml` `package.version`.
2. Commit, then tag: `git tag v0.1.1 && git push --tags`.
3. The `Release` workflow cross-compiles the Rust cdylib, assembles the AAR,
   publishes to GitHub Packages, and attaches the AAR to the GitHub Release.

## License

Dual licensed under MIT and Apache 2.0.
