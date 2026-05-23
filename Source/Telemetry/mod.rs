
//! Telemetry & observability surface. Currently a feature-gated stub stack:
//! tracing/metrics live behind `--features Telemetry`; the build flags
//! (`IsEnabled`, `IsDebugBuild`, `IsDevelopmentBuild`) are always available
//! so callers can gate cheap diagnostic paths without an `#[cfg]` block.
//!
//! ## Status
//!
//! Zero external call sites today. Wire from `Binary::Main` when the
//! Telemetry feature ships.

pub mod Gates;

pub mod IsDebugBuild;

pub mod IsDevelopmentBuild;

pub mod IsEnabled;

#[cfg(feature = "Telemetry")]
pub mod Initialize;

#[cfg(feature = "Telemetry")]
pub mod Metrics;

#[cfg(feature = "Telemetry")]
pub mod Tracing;

pub mod FeatureFlags;
