//! JNI bindings for Iroh blob operations on Android.
//!
//! See `jni_bridge.rs` for the Kotlin-facing entry points and `node.rs`
//! for the shared blob put/get core. This crate mirrors the iroh-swift
//! `rust/` crate but exposes JNI instead of a C ABI.

mod jni_bridge;
mod node;
