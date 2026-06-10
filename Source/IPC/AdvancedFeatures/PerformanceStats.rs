//! Cumulative IPC counters - sent / received message totals,
//! rolled-up average processing time, peak rate, error count,
//! and uptime tick. Returned by
//! `mountain_get_performance_stats`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub total_messages_sent:u64,

	pub total_messages_received:u64,

	pub average_processing_time_ms:f64,

	pub peak_message_rate:u32,

	pub error_count:u32,

	pub last_update:u64,

	pub connection_uptime:u64,
}
