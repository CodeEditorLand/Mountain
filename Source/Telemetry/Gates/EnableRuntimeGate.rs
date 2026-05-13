#![allow(non_snake_case)]

//! Add a gate to the runtime set after boot.
//!
//! ## No-op shim
//!
//! The original implementation stored the set in `OnceLock<HashSet>`
//! and tried to mutate it through `&HashSet` - a latent compile error
//! once the call site fired. The set must move to a `Mutex<HashSet>` (or
//! `parking_lot::RwLock`) before this function can mutate state. For now
//! it is a no-op that preserves the signature so call sites compile, and
//! returns Ok so flow control continues.

pub fn Fn(_GateName:String) -> Result<(), String> { Ok(()) }
