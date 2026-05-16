package net.arkavo.iroh

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.Closeable
import java.util.concurrent.atomic.AtomicLong

/**
 * An Iroh node — a full peer that publishes and fetches blobs.
 *
 * Wraps a Rust-owned `IrohNode` allocated by [IrohNative.create]. Native
 * JNI calls are blocking; this class moves them to [Dispatchers.IO] so the
 * Kotlin surface is suspend.
 *
 * Lifecycle:
 *   val node = IrohNode.create("/data/.../iroh", relayEnabled = true)
 *   try { node.put(bytes); node.get(ticket) } finally { node.close() }
 *
 * Calls made after [close] throw [IrohException.NodeUnavailable]. The
 * handle is freed exactly once even under concurrent close.
 */
class IrohNode private constructor(handle: Long) : Closeable {

    private val handleRef = AtomicLong(handle)

    /** Hex-encoded NodeId visible to peers. */
    suspend fun nodeId(): String = onIo { IrohNative.nodeId(it) }

    /** Add bytes to the local blob store, return a shareable ticket. */
    suspend fun put(bytes: ByteArray): String = onIo { IrohNative.put(it, bytes) }

    /** Download the blob referenced by [ticket]; returns the bytes. */
    suspend fun get(ticket: String): ByteArray = onIo { IrohNative.get(it, ticket) }

    /** Tear down the node and its tokio runtime. Idempotent. */
    override fun close() {
        val h = handleRef.getAndSet(0L)
        if (h != 0L) {
            IrohNative.destroy(h)
        }
    }

    private suspend fun <T> onIo(block: (Long) -> T): T = withContext(Dispatchers.IO) {
        val h = handleRef.get()
        if (h == 0L) throw IrohException.NodeUnavailable("IrohNode is closed")
        block(h)
    }

    companion object {
        /**
         * Create a new Iroh node with persistent blob storage at [storagePath].
         *
         * The directory is created on demand. Pass [customRelayUrl] = null
         * to use n0's public relays.
         */
        suspend fun create(
            storagePath: String,
            relayEnabled: Boolean = true,
            customRelayUrl: String? = null,
        ): IrohNode = withContext(Dispatchers.IO) {
            val handle = IrohNative.create(storagePath, relayEnabled, customRelayUrl)
            if (handle == 0L) {
                throw IrohException.NodeUnavailable("IrohNative.create returned 0")
            }
            IrohNode(handle)
        }
    }
}
