
//! `true` when compiled without `--release`.

#[inline]
pub const fn Fn() -> bool { cfg!(debug_assertions) }
