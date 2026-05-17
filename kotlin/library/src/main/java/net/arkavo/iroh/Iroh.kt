package net.arkavo.iroh

import android.content.Context

/**
 * Library-level bootstrap.
 *
 * Call [initContext] before constructing any [IrohNode], typically from
 * `Application.onCreate`. The Rust dependencies under iroh require an
 * initialized `ndk-context` with a long-lived Context global ref to access
 * Android networking APIs.
 *
 * Idempotent: subsequent successful calls are silent no-ops. Only the first
 * call's Context is retained for the process lifetime. If an init call
 * throws [IrohException.NodeUnavailable], retrying is permitted — the guard
 * is reset on failure.
 *
 * Passing `applicationContext` (rather than an Activity) is strongly
 * recommended — the native side holds a JNI global ref for the
 * process lifetime.
 */
object Iroh {
    @JvmStatic
    fun initContext(context: Context) {
        IrohNative.initContext(context.applicationContext)
    }
}
