
//! Single-tick IPC status report sent to Sky: connection
//! state, queue depth, listener inventory, recent message
//! stats, error count, uptime.

use serde::{Deserialize, Serialize};

use crate::IPC::StatusReporter::{ConnectionStatus, MessageStats};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub timestamp:u64,

	pub connection_status:ConnectionStatus::Struct,

	pub message_queue_size:usize,

	pub active_listeners:Vec<String>,

	pub recent_messages:Vec<MessageStats::Struct>,

	pub error_count:u32,

	pub uptime_seconds:u64,
}
