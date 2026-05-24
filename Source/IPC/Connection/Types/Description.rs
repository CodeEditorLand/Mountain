//! `Types::Description`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> &'static str {
		match self {
			ConnectionStatus::Connected => "Connected and healthy",

			ConnectionStatus::Disconnected => "Disconnected",

			ConnectionStatus::Degraded => "Degraded - experiencing issues",

			ConnectionStatus::Failed => "Failed - connection lost",
		}
	}
