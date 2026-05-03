#![allow(non_snake_case)]

//! Snapshot of the channel's current key, age, usage count,
//! number of retained previous keys, and the active config.

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::SecureMessageChannel::SecurityConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub current_key_id:String,
	pub current_key_age_seconds:u64,
	pub current_key_usage_count:usize,
	pub previous_keys_count:usize,
	pub config:SecurityConfig::Struct,
}
