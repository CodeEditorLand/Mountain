
//! Message cache state - id → `CachedMessage::Struct` table
//! plus hit / miss counters and a derived size accessor.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::IPC::AdvancedFeatures::CachedMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub cached_messages:HashMap<String, CachedMessage::Struct>,

	pub cache_hits:u64,

	pub cache_misses:u64,

	pub cache_size:usize,
}
