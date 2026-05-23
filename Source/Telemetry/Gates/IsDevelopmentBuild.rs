//! Compile-time gate - `true` under the `Development` feature OR
//! `cfg!(debug_assertions)`. Lets dev-only code paths run in CI debug
//! builds without flipping a feature flag.

#[inline]
pub const fn Fn() -> bool { cfg!(feature = "Development") || cfg!(debug_assertions) }
