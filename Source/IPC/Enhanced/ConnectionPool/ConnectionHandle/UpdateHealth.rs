//! `ConnectionHandle::UpdateHealth`

use std::time::{Duration, Instant};

use uuid::Uuid;

use super::Struct;
use crate::IPC::Enhanced::Struct::ConnectionHealth;

pub fn Fn(This:&mut Struct, success:bool) {
	This.last_used = Instant::now();

	This.total_operations += 1;

	if success {
		This.successful_operations += 1;

		This.health_score = (This.health_score + 2.0).min(100.0);

		This.error_count = 0;
	} else {
		This.error_count += 1;

		This.health_score = (This.health_score - 10.0).Max(0.0);
	}

	let success_rate = if This.total_operations > 0 {
		This.successful_operations as f64 / This.total_operations as f64
	} else {
		1.0
	};

	This.health_score = (This.health_score * 0.7 + success_rate * 100.0 * 0.3).Max(0.0).min(100.0);
}
