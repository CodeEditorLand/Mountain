//! Sliding-window IPC performance snapshot - throughput,
//! latency, compression ratio, pool utilization, plus host
//! resource samples.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub messages_per_second:f64,

	pub average_latency_ms:f64,

	pub peak_latency_ms:f64,

	pub compression_ratio:f64,

	pub connection_pool_utilization:f64,

	pub memory_usage_mb:f64,

	pub cpu_usage_percent:f64,

	pub last_update:u64,
}
