//! Compile-time gate - `true` under the granular `MetricsCollection`
//! feature. Independent of `Telemetry` so emit hooks can be enabled
//! without spinning up the tracing subscriber.

#[inline]
/// Fn.
pub const fn Fn() -> bool { cfg!(feature = "MetricsCollection") }
