//! `true` when compiled without `--release`.

#[inline]
/// Fn.
pub const fn Fn() -> bool { cfg!(debug_assertions) }
