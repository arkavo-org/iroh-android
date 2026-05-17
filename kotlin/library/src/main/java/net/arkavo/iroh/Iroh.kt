package net.arkavo.iroh

import android.content.Context

/**
 * Library-level bootstrap.
 *
 * Call [initContext] exactly once at app startup (typically from
 * `Application.onCreate`) before constructing any [IrohNode]. The Rust
 * dependencies under iroh require an initialized `ndk-context` with a
 * long-lived Context global ref to access Android networking APIs.
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
