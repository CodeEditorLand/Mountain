//! `Types::ResetHealth`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct) {
		This.health_score = 100.0;

		This.error_count = 0;

		This.last_used = std::time::SystemTime::now();
	}
