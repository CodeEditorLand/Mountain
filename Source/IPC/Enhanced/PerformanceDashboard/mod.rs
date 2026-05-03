#![allow(non_snake_case)]

//! # Performance Dashboard
//!
//! Advanced monitoring + distributed-tracing module. Records
//! `PerformanceMetric::Struct` samples into a ring buffer,
//! tracks `TraceSpan::Struct` lifecycles, raises
//! `PerformanceAlert::Struct` when configured thresholds are
//! exceeded, and rolls everything into a
//! `DashboardStatistics::Struct`. The `Dashboard::Struct`
//! aggregator + 25-method impl lives in one sibling because
//! the impl is tightly coupled with the DTOs (see Convention's
//! "tightly-coupled cluster" exception).

pub mod AlertSeverity;
pub mod Dashboard;
pub mod DashboardConfig;
pub mod DashboardStatistics;
pub mod LogLevel;
pub mod MetricType;
pub mod PerformanceAlert;
pub mod PerformanceMetric;
pub mod TraceLog;
pub mod TraceSpan;
