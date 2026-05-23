
//! Compile-time gate - `true` under the top-level `Telemetry` feature.

#[inline]
pub const fn Fn() -> bool { cfg!(feature = "Telemetry") }
