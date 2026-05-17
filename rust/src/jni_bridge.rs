//! JNI entry points called from `net.arkavo.iroh.IrohNative`.
//!
//! Naming convention: Kotlin natives live in `object IrohNative` so JNI
//! mangles to `Java_net_arkavo_iroh_IrohNative_<method>`. Each function below
//! corresponds 1:1 to an `external fun` in `IrohNative.kt`.
//!
//! Error handling: Rust panics are caught and turned into
//! `IrohException` throws on the JVM side. anyhow::Error is also mapped to
//! the appropriate `IrohException` subclass.

use crate::node::IrohNode;
use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JObject, JString};
use jni::sys::{jbyteArray, jlong, jstring};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};

static CONTEXT_INITIALIZED: AtomicBool = AtomicBool::new(false);

const EXC_NODE_UNAVAILABLE: &str = "net/arkavo/iroh/IrohException$NodeUnavailable";
const EXC_PUBLISH_FAILED: &str = "net/arkavo/iroh/IrohException$PublishFailed";
const EXC_FETCH_FAILED: &str = "net/arkavo/iroh/IrohException$FetchFailed";
const EXC_TICKET_INVALID: &str = "net/arkavo/iroh/IrohException$TicketInvalid";

// Each kind of operation maps to a specific exception class so the Kotlin
// caller can build a typed sealed-class result.
fn throw_iroh(env: &mut JNIEnv, class: &str, message: &str) {
    // If throwing fails (extremely unlikely — typically only if class is
    // wrong), the JVM is already in a bad state. We log via println since
    // Android's logcat redirects stdout/stderr from native libs.
    if env.throw_new(class, message).is_err() {
        eprintln!("iroh-android: failed to throw {}: {}", class, message);
    }
}

fn handle_to_node<'a>(handle: jlong) -> Option<&'a IrohNode> {
    if handle == 0 {
        return None;
    }
    // Safety: handle was produced by Box::into_raw in nativeCreate and is
    // not freed until nativeDestroy. Kotlin enforces single-threaded close
    // semantics on IrohNode.
    unsafe { (handle as *const IrohNode).as_ref() }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_arkavo_iroh_IrohNative_initContext<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    context: JObject<'local>,
) {
    // ndk-context's initialize_android_context is once-only — a second
    // successful call would leak another JNI global ref and overwrite the
    // previous VM/Context registration. CAS so concurrent callers race-safely
    // no-op; on failure we reset so the consumer can retry.
    if CONTEXT_INITIALIZED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    let result = catch_unwind(AssertUnwindSafe(|| -> anyhow::Result<()> {
        let vm = env
            .get_java_vm()
            .map_err(|e| anyhow::anyhow!("get_java_vm: {e}"))?;
        let global = env
            .new_global_ref(&context)
            .map_err(|e| anyhow::anyhow!("new_global_ref(context): {e}"))?;
        // ndk-context expects a raw pointer to a JNI global ref that lives
        // for the process lifetime. Leak the GlobalRef so it isn't dropped.
        let raw_context = global.as_raw();
        std::mem::forget(global);
        unsafe {
            ndk_context::initialize_android_context(
                vm.get_java_vm_pointer() as *mut std::ffi::c_void,
                raw_context as *mut std::ffi::c_void,
            );
        }
        Ok(())
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            CONTEXT_INITIALIZED.store(false, Ordering::Release);
            throw_iroh(&mut env, EXC_NODE_UNAVAILABLE, &format!("{e:#}"));
        }
        Err(_) => {
            CONTEXT_INITIALIZED.store(false, Ordering::Release);
            throw_iroh(&mut env, EXC_NODE_UNAVAILABLE, "panic in initContext");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_arkavo_iroh_IrohNative_create(
    mut env: JNIEnv,
    _class: JClass,
    storage_path: JString,
    relay_enabled: jni::sys::jboolean,
    custom_relay_url: JString,
) -> jlong {
    let result = catch_unwind(AssertUnwindSafe(|| -> anyhow::Result<jlong> {
        let storage_path: String = env
            .get_string(&storage_path)
            .map_err(|e| anyhow::anyhow!("storagePath JNI: {e}"))?
            .into();
        let custom_relay: Option<String> = if custom_relay_url.is_null() {
            None
        } else {
            Some(
                env.get_string(&custom_relay_url)
                    .map_err(|e| anyhow::anyhow!("customRelayUrl JNI: {e}"))?
                    .into(),
            )
        };
        let node = IrohNode::new(storage_path.into(), relay_enabled != 0, custom_relay)?;
        let boxed = Box::new(node);
        Ok(Box::into_raw(boxed) as jlong)
    }));

    match result {
        Ok(Ok(handle)) => handle,
        Ok(Err(e)) => {
            throw_iroh(&mut env, EXC_NODE_UNAVAILABLE, &format!("{e:#}"));
            0
        }
        Err(_) => {
            throw_iroh(&mut env, EXC_NODE_UNAVAILABLE, "panic in IrohNode::new");
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_arkavo_iroh_IrohNative_destroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    // Safety: handle came from Box::into_raw in `create`. Reconstructing
    // and dropping shuts down the runtime and frees endpoint/store.
    unsafe {
        let _ = Box::from_raw(handle as *mut IrohNode);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_arkavo_iroh_IrohNative_nodeId<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jstring {
    let node = match handle_to_node(handle) {
        Some(n) => n,
        None => {
            throw_iroh(
                &mut env,
                EXC_NODE_UNAVAILABLE,
                "node handle is 0 / already destroyed",
            );
            return JObject::null().into_raw() as jstring;
        }
    };
    match env.new_string(node.node_id()) {
        Ok(s) => s.into_raw(),
        Err(e) => {
            throw_iroh(&mut env, EXC_NODE_UNAVAILABLE, &format!("new_string: {e}"));
            JObject::null().into_raw() as jstring
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_arkavo_iroh_IrohNative_put<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    data: JByteArray<'local>,
) -> jstring {
    let node = match handle_to_node(handle) {
        Some(n) => n,
        None => {
            throw_iroh(
                &mut env,
                EXC_PUBLISH_FAILED,
                "node handle is 0 / already destroyed",
            );
            return JObject::null().into_raw() as jstring;
        }
    };

    let result = catch_unwind(AssertUnwindSafe(|| -> anyhow::Result<String> {
        let bytes = env
            .convert_byte_array(&data)
            .map_err(|e| anyhow::anyhow!("convert_byte_array: {e}"))?;
        node.put(&bytes)
    }));

    match result {
        Ok(Ok(ticket)) => match env.new_string(ticket) {
            Ok(s) => s.into_raw(),
            Err(e) => {
                throw_iroh(&mut env, EXC_PUBLISH_FAILED, &format!("new_string: {e}"));
                JObject::null().into_raw() as jstring
            }
        },
        Ok(Err(e)) => {
            throw_iroh(&mut env, EXC_PUBLISH_FAILED, &format!("{e:#}"));
            JObject::null().into_raw() as jstring
        }
        Err(_) => {
            throw_iroh(&mut env, EXC_PUBLISH_FAILED, "panic in IrohNode::put");
            JObject::null().into_raw() as jstring
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_arkavo_iroh_IrohNative_get<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    ticket: JString<'local>,
) -> jbyteArray {
    let node = match handle_to_node(handle) {
        Some(n) => n,
        None => {
            throw_iroh(
                &mut env,
                EXC_FETCH_FAILED,
                "node handle is 0 / already destroyed",
            );
            return JObject::null().into_raw() as jbyteArray;
        }
    };

    let result = catch_unwind(AssertUnwindSafe(|| -> anyhow::Result<Vec<u8>> {
        let ticket_str: String = env
            .get_string(&ticket)
            .map_err(|e| anyhow::anyhow!("ticket JNI: {e}"))?
            .into();
        node.get(&ticket_str)
    }));

    match result {
        Ok(Ok(bytes)) => match env.byte_array_from_slice(&bytes) {
            Ok(arr) => arr.into_raw(),
            Err(e) => {
                throw_iroh(
                    &mut env,
                    EXC_FETCH_FAILED,
                    &format!("byte_array_from_slice: {e}"),
                );
                JObject::null().into_raw() as jbyteArray
            }
        },
        Ok(Err(e)) => {
            // Heuristic: parse failure → TicketInvalid; everything else → FetchFailed
            let msg = format!("{e:#}");
            let cls = if msg.contains("parse ticket") {
                EXC_TICKET_INVALID
            } else {
                EXC_FETCH_FAILED
            };
            throw_iroh(&mut env, cls, &msg);
            JObject::null().into_raw() as jbyteArray
        }
        Err(_) => {
            throw_iroh(&mut env, EXC_FETCH_FAILED, "panic in IrohNode::get");
            JObject::null().into_raw() as jbyteArray
        }
    }
}
