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
//! - `InitializeStatusReporter::initialize_status_reporter` - bootstrap helper
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

pub mod Reporter;

pub mod ServiceInfo;

pub mod ServiceMetrics;

pub mod ServiceRegistry;

pub mod ServiceStatus;

pub mod SeverityLevel;

pub mod mountain_attempt_recovery;

pub mod mountain_discover_services;

pub mod mountain_get_comprehensive_status;

pub mod mountain_get_health_status;

pub mod mountain_get_ipc_status;

pub mod mountain_get_ipc_status_history;

pub mod mountain_get_performance_metrics;

pub mod mountain_get_service_info;

pub mod mountain_get_service_registry;

pub mod mountain_perform_health_check;

pub mod mountain_start_ipc_status_reporting;

pub mod mountain_start_service_discovery;
