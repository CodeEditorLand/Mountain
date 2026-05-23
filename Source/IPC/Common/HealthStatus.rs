//! # Health Status Monitoring
//!
//! Component-level health tracking for IPC subsystems. The score is
//! a deterministic aggregation of currently-tracked issues weighted by
//! severity.
//!
//! Layout (one export per file, file name = identity):
//! - `SeverityLevel::Enum` - Low / Medium / High / Critical (ordered).
//! - `HealthIssue::Enum` - tagged issue with `Severity` and `Description`
//!   accessors.
//! - `HealthMonitor::Struct` - score + issue list + recovery counter.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. `IPC::StatusReporter` defines
//! its own `HealthMonitor` / `HealthIssue` with different shapes; the
//! two should converge in a future batch.

pub mod HealthIssue;

pub mod HealthMonitor;

pub mod SeverityLevel;
