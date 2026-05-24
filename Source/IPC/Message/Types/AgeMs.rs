//! `Types::AgeMs`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> u64 {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		now.saturating_sub(This.timestamp)
	}
