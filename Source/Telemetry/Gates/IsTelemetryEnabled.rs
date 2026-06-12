//! Compile-time gate - `true` under the top-level `Telemetry` feature.

#[inline]
/// Fn.
pub const fn Fn() -> bool { cfg!(feature = "Telemetry") }
