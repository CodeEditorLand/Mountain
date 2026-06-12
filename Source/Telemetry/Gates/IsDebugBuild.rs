//! Compile-time gate - evaluates to `true` under `cfg!(debug_assertions)`.

#[inline]
/// Fn.
pub const fn Fn() -> bool { cfg!(debug_assertions) }
