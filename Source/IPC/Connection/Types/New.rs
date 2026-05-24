//! `Types::New`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn() -> Struct {
		let now = std::time::SystemTime::now();

		Self {
			id:uuid::Uuid::new_v4().to_string(),

			created_at:now,

			last_used:now,

			health_score:100.0,

			error_count:0,
		}
	}
