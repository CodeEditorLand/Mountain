//! Air daemon status DTO returned by `GetAirStatus`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub version:String,

	pub uptime_seconds:u64,

	pub total_requests:u64,

	pub successful_requests:u64,

	pub failed_requests:u64,

	pub active_requests:u32,

	pub healthy:bool,
}
