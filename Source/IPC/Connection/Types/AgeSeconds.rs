//! `Types::AgeSeconds`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> u64 {
		This.created_at
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0)
	}
