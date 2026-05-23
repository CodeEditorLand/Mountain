//! Per-channel message counters used to compute throughput
//! and average processing time inside `IPCStatusReport::Struct`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub channel:String,

	pub message_count:u32,

	pub last_message_time:u64,

	pub average_processing_time_ms:f64,
}
