//! Single TTL-bound cache entry: payload, insertion timestamp
//! (UNIX seconds), and time-to-live in seconds.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub data:serde_json::Value,

	pub timestamp:u64,

	pub ttl:u64,
}
