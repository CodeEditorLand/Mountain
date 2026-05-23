//! Aggregated dashboard counters - cumulative metric / trace /
//! alert counts plus the rolled-up averages and last-update
//! tick.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub total_metrics_collected:u64,

	pub total_traces_collected:u64,

	pub total_alerts_triggered:u64,

	pub average_processing_time_ms:f64,

	pub peak_processing_time_ms:u64,

	pub error_rate_percentage:f64,

	pub throughput_messages_per_second:f64,

	pub memory_usage_mb:f64,

	pub last_update:u64,
}
