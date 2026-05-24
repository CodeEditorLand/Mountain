pub mod New;
pub mod UpdateHealth;
pub mod IsHealthy;
pub mod Age;
pub mod IdleTime;
pub mod SuccessRate;

use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::IPC::Enhanced::Struct::ConnectionHealth;

#[derive(Debug, Clone)]
pub struct Struct {
	pub id:String,

	pub created_at:Instant,

	pub last_used:Instant,

	pub health_score:f64,

	pub error_count:usize,

	pub successful_operations:usize,

	pub total_operations:usize,

	pub is_active:bool,

	pub reuse_count:u32,

	pub health:ConnectionHealth::Enum,
}
