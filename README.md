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

## Consuming in closurekb/android

Either reference the prebuilt AAR via flatDir, or include the Gradle project
as a composite build. See `closurekb/android/app/src/main/java/com/closurekb/data/iroh/README.md`.

## License

Dual licensed under MIT and Apache 2.0.
