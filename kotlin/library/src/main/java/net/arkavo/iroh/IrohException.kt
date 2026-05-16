package net.arkavo.iroh

/**
 * Errors surfaced from the JNI bridge.
 *
 * The Rust side throws subclasses (`IrohException$NodeUnavailable`,
 * etc.) directly via JNI — keep these class names stable.
 */
sealed class IrohException(message: String) : RuntimeException(message) {

    /** Node could not be created, or the handle is gone. */
    class NodeUnavailable(message: String) : IrohException(message)

    /** put() failed to add bytes to the local store. */
    class PublishFailed(message: String) : IrohException(message)

    /** Ticket string is malformed or unrecognized. */
    class TicketInvalid(message: String) : IrohException(message)

    /** get() failed during download or local read. */
    class FetchFailed(message: String) : IrohException(message)
}
