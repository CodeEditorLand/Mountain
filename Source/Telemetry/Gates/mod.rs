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

/// Enableruntimegate module.
pub mod EnableRuntimeGate;

/// Getruntimegates module.
pub mod GetRuntimeGates;

/// Isdebugbuild module.
pub mod IsDebugBuild;

/// Isdevelopmentbuild module.
pub mod IsDevelopmentBuild;

/// Isdistributedtracingenabled module.
pub mod IsDistributedTracingEnabled;

/// Isfeatureflagsenabled module.
pub mod IsFeatureFlagsEnabled;

/// Ismetricsenabled module.
pub mod IsMetricsEnabled;

/// Istelemetryenabled module.
pub mod IsTelemetryEnabled;

/// Listenabledgates module.
pub mod ListEnabledGates;

/// Runtimegateenabled module.
pub mod RuntimeGateEnabled;

/// Validaterequiredgates module.
pub mod ValidateRequiredGates;

pub(crate) mod RuntimeGates;
