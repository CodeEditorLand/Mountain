//! Compile-time gate - `true` under the `DistributedTracing` feature.

#[inline]
/// Fn.
pub const fn Fn() -> bool { cfg!(feature = "DistributedTracing") }
