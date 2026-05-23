//! Compile-time gate - `true` under the `RuntimeFeatureFlags` feature.

#[inline]
pub const fn Fn() -> bool { cfg!(feature = "RuntimeFeatureFlags") }
