#![allow(non_snake_case)]

//! Combined status report - basic IPC slice + performance
//! metrics + health monitor - emitted to Sky periodically and
//! returned by `mountain_get_comprehensive_status`.

use serde::{Deserialize, Serialize};

use crate::IPC::StatusReporter::{HealthMonitor, IPCStatusReport, PerformanceMetrics};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub basic_status:IPCStatusReport::Struct,
	pub performance_metrics:PerformanceMetrics::Struct,
	pub health_status:HealthMonitor::Struct,
	pub timestamp:u64,
}
