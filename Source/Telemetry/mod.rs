//! Telemetry & observability surface. Currently a feature-gated stub stack:
//! tracing/metrics live behind `--features Telemetry`; the build flags
//! (`IsEnabled`, `IsDebugBuild`, `IsDevelopmentBuild`) are always available
//! so callers can gate cheap diagnostic paths without an `#[cfg]` block.
//!
//! ## Status
//!
//! Zero external call sites today. Wire from `Binary::Main` when the
//! Telemetry feature ships.

/// Gates module.
pub mod Gates;

/// Isdebugbuild module.
pub mod IsDebugBuild;

/// Isdevelopmentbuild module.
pub mod IsDevelopmentBuild;

/// Isenabled module.
pub mod IsEnabled;

#[cfg(feature = "Telemetry")]
/// Initialize module.
pub mod Initialize;

#[cfg(feature = "Telemetry")]
/// Metrics module.
pub mod Metrics;

#[cfg(feature = "Telemetry")]
/// Tracing module.
pub mod Tracing;

/// Featureflags module.
pub mod FeatureFlags;
