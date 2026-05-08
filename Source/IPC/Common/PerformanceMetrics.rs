#![allow(non_snake_case)]

//! # IPC Performance Metrics
//!
//! Two complementary observation primitives:
//!
//! - `PerformanceMetrics::Struct` - mixed snapshot of throughput, latency (avg
//!   + peak), success rate, compression, pool, and host resource counters. The
//!   natural shape for a single point-in-time readout
//!   (`get_performance_metrics` Tauri command).
//! - `ThroughputMetrics::Struct` - directional message/byte counters pinned to
//!   a `StartTime`. The natural shape for rate calculations over a window.
//!
//! Both files use `pub struct Struct` so callers spell the full
//! reverse-hierarchical path:
//! `IPC::Common::PerformanceMetrics::PerformanceMetrics::Struct`.
//!
//! TODO: zero callers as of 2026-05-02. `IPC::StatusReporter` defines
//! its own `PerformanceMetrics` struct with a different shape; the two
//! should converge in a future batch.

pub mod PerformanceMetrics;

pub mod ThroughputMetrics;
