//! `Types::HealthPercentage`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> f64 {
		if This.total_connections == 0 {
			return 100.0;
		}

		(This.healthy_connections as f64 / This.total_connections as f64) * 100.0
	}
