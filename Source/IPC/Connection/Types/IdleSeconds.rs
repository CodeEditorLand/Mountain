//! `Types::IdleSeconds`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> u64 {
		This.last_used
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0)
	}
