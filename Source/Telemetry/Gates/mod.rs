//! # Telemetry Gates
//!
//! Compile-time and runtime feature gates that decide which telemetry
//! code paths are alive in the current binary.
//!
//! ## Layout
//!
//! - `IsDebugBuild`, `IsDevelopmentBuild`, `IsTelemetryEnabled`,
//!   `IsMetricsEnabled`, `IsDistributedTracingEnabled`,
//!   `IsFeatureFlagsEnabled`: `cfg!`-driven `const fn` predicates.
//! - `GetRuntimeGates`, `RuntimeGateEnabled`, `ListEnabledGates`,
//!   `ValidateRequiredGates`: runtime set accessors.
//! - `EnableRuntimeGate`: no-op shim until storage moves from
//!   `OnceLock<HashSet>` to `RwLock<HashSet>`.
//! - `RuntimeGates`: module-private singleton and initializer.
//!
//! ## Status
//!
//! No callers as of 2026-05-02. Wire into IPC dispatch and command execution
//! once the gates are read from runtime config.

/// Runtime gate enablement (no-op shim).
pub mod EnableRuntimeGate;

/// Runtime gate list accessor.
pub mod GetRuntimeGates;

/// Debug build detection.
pub mod IsDebugBuild;

/// Development build detection.
pub mod IsDevelopmentBuild;

/// Distributed tracing enabled check.
pub mod IsDistributedTracingEnabled;

/// Feature flags enabled check.
pub mod IsFeatureFlagsEnabled;

/// Metrics enabled check.
pub mod IsMetricsEnabled;

/// Telemetry enabled check.
pub mod IsTelemetryEnabled;

/// Enabled gates list accessor.
pub mod ListEnabledGates;

/// Single runtime gate enabled check.
pub mod RuntimeGateEnabled;

/// Required gates validation.
pub mod ValidateRequiredGates;

pub(crate) mod RuntimeGates;
