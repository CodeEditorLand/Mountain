//! `ConnectionHandle::New`

use std::time::{Duration, Instant};

use uuid::Uuid;

use super::Struct;
use crate::IPC::Enhanced::Struct::ConnectionHealth;

pub fn Fn() -> Struct {
	Self {
		id:Uuid::new_v4().to_string(),

		created_at:Instant::now(),

		last_used:Instant::now(),

		health_score:100.0,

		error_count:0,

		successful_operations:0,

		total_operations:0,

		is_active:true,

		reuse_count:0,

		health:ConnectionHealth::Enum::Healthy,
	}
}
