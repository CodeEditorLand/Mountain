//! Compile-time gate - `true` under the `RuntimeFeatureFlags` feature.

#[inline]
/// Fn.
pub const fn Fn() -> bool { cfg!(feature = "RuntimeFeatureFlags") }
