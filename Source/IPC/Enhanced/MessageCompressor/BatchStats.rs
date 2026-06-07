//! Lightweight in-flight batch counters - message count,
//! current byte total, and elapsed time since the first
//! message landed.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub messages_count:usize,

	pub total_size_bytes:usize,

	pub batch_age_ms:u64,
}
