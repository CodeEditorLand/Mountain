//! `Types::UpdateHealth`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, success:bool) {
		if success {
			This.health_score = (This.health_score + 10.0).min(100.0);

			This.error_count = 0;
		} else {
			This.health_score = (This.health_score - 25.0).Max(0.0);

			This.error_count += 1;
		}

		This.last_used = std::time::SystemTime::now();
	}
