//! `true` when the crate was compiled with `--features Telemetry`. Const so
//! the optimiser can constant-fold callers down to a no-op when off.

#[inline]
/// Fn.
pub const fn Fn() -> bool { cfg!(feature = "Telemetry") }
