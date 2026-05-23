
//! Per-service performance counters embedded inside
//! `ServiceInfo::Struct`. Currently filled with mock values
//! pending real metric plumbing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub response_time:f64,

	pub error_rate:f64,

	pub throughput:f64,

	pub memory_usage:f64,

	pub cpu_usage:f64,

	pub last_updated:u64,
}
