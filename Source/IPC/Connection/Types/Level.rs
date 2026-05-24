//! `Types::Level`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> u8 {
		match self {
			ConnectionStatus::Failed => 0,

			ConnectionStatus::Degraded => 1,

			ConnectionStatus::Disconnected => 2,

			ConnectionStatus::Connected => 3,
		}
	}
