//! # Telemetry Gates
//!
//! Compile-time and runtime feature gates that decide which telemetry
//! code paths are alive in the current binary.
//!
//! Layout (one export per file, file name = identity):
//! - `IsDebugBuild::Fn`, `IsDevelopmentBuild::Fn`, `IsTelemetryEnabled::Fn`,
//!   `IsMetricsEnabled::Fn`, `IsDistributedTracingEnabled::Fn`,
//!   `IsFeatureFlagsEnabled::Fn` - `cfg!`-driven `const fn` predicates.
//! - `GetRuntimeGates::Fn`, `RuntimeGateEnabled::Fn`, `ListEnabledGates::Fn`,
//!   `ValidateRequiredGates::Fn` - runtime set accessors.
//! - `EnableRuntimeGate::Fn` - TODO no-op shim until the storage moves from
//!   `OnceLock<HashSet>` to `RwLock<HashSet>`.
//! - `RuntimeGates::GATES` - module-private singleton + initialiser.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. Wire into IPC dispatch and
//! command execution once the gates are read from runtime config.

pub mod EnableRuntimeGate;

pub mod GetRuntimeGates;

pub mod IsDebugBuild;

pub mod IsDevelopmentBuild;

pub mod IsDistributedTracingEnabled;

pub mod IsFeatureFlagsEnabled;

pub mod IsMetricsEnabled;

pub mod IsTelemetryEnabled;

pub mod ListEnabledGates;

pub mod RuntimeGateEnabled;

pub mod ValidateRequiredGates;

pub(crate) mod RuntimeGates;
