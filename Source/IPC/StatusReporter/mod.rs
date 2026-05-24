//! # IPC Status Reporter
//!
//! Mountain-side observability for the Wind ↔ Mountain bridge.
//! Collects per-channel message counters, latency, queue depth,
//! and host resource samples; aggregates them into health-score
//! and service-discovery snapshots; emits to Sky via Tauri
//! events on a configurable interval.
//!
//! Layout:
//! - DTO siblings (`Struct` / `Enum` per file): the wire shapes Sky
//!   deserialises.
//! - `Reporter::Struct` - the aggregator + 30-method impl (one cohesive unit;
//!   constructor, periodic loops, health check, service discovery, recovery).
//! - `mountain_*` Tauri commands - one wire-bound entry per file, all
//!   delegating to the `Reporter::Struct` in Tauri state.
//! - `InitializeStatusReporter::Fn` - bootstrap helper
//!   called from `Binary/Register/StatusReporterRegister.rs`.

pub mod ComprehensiveStatusReport;

pub mod ConnectionStatus;

pub mod HealthIssue;

pub mod HealthIssueType;

pub mod HealthMonitor;

pub mod IPCStatusReport;

pub mod InitializeStatusReporter;

pub mod MessageStats;

pub mod PerformanceMetrics;

#[path = "Reporter/mod.rs"]
pub mod Reporter;

pub mod ServiceInfo;

pub mod ServiceMetrics;

pub mod ServiceRegistry;

pub mod ServiceStatus;

pub mod SeverityLevel;

pub mod MountainAttemptRecovery;

pub mod MountainDiscoverServices;

pub mod MountainGetComprehensiveStatus;

pub mod MountainGetHealthStatus;

pub mod MountainGetIpcStatus;

pub mod MountainGetIpcStatusHistory;

pub mod MountainGetPerformanceMetrics;

pub mod MountainGetServiceInfo;

pub mod MountainGetServiceRegistry;

pub mod MountainPerformHealthCheck;

pub mod MountainStartIpcStatusReporting;

pub mod MountainStartServiceDiscovery;
