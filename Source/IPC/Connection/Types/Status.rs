//! `Types::Status`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> ConnectionStatus {
		if This.IsHealthy() {
			ConnectionStatus::Connected
		} else if This.health_score > 25.0 {
			ConnectionStatus::Degraded
		} else {
			ConnectionStatus::Failed
		}
	}
