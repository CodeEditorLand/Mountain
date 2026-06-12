//! `GetStatus` response DTO. Carries uptime, request counts, health flag.

#[derive(Debug, Clone)]
/// Data for struct.
pub struct Struct {
	pub version:String,

	pub uptime_seconds:u64,

	pub total_requests:u64,

	pub successful_requests:u64,

	pub failed_requests:u64,

	pub active_requests:u32,

	pub healthy:bool,

	pub error:String,
}
