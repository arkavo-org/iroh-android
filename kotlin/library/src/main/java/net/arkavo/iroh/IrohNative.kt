package net.arkavo.iroh

/**
 * JNI surface. Do not call from app code — use [IrohNode] instead.
 *
 * Native methods correspond 1:1 to functions in `iroh-android/rust/src/jni_bridge.rs`.
 * All native methods throw `IrohException` subclasses on failure.
 */
internal object IrohNative {
    init {
        System.loadLibrary("iroh_android")
    }

    @JvmStatic external fun initContext(context: android.content.Context)

    @JvmStatic external fun create(
        storagePath: String,
        relayEnabled: Boolean,
        customRelayUrl: String?,
    ): Long

    @JvmStatic external fun destroy(handle: Long)

    @JvmStatic external fun nodeId(handle: Long): String

    @JvmStatic external fun put(handle: Long, data: ByteArray): String

    @JvmStatic external fun get(handle: Long, ticket: String): ByteArray
}
