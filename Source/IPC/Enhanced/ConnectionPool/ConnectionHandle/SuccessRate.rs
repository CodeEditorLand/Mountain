//! `ConnectionHandle::SuccessRate`

use std::time::{Duration, Instant};

use uuid::Uuid;

use super::Struct;
use crate::IPC::Enhanced::Struct::ConnectionHealth;

pub fn Fn(This:&Struct) -> f64 {
	if This.total_operations == 0 {
		1.0
	} else {
		This.successful_operations as f64 / This.total_operations as f64
	}
}
