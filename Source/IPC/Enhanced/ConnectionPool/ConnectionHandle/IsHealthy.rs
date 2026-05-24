//! `ConnectionHandle::IsHealthy`

use std::time::{Duration, Instant};

use uuid::Uuid;

use super::Struct;
use crate::IPC::Enhanced::Struct::ConnectionHealth;

pub fn Fn(This:&Struct) -> bool {
	This.health_score > 50.0 && This.error_count < 5 && This.is_active && This.Age().as_secs() < 300
}
