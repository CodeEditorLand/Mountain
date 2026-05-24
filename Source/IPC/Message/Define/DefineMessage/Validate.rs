//! `DefineMessage::Validate`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> Result<(), String> {

		// Ensure channel is not empty
		if This.channel.is_empty() {

			return Err("Channel cannot be empty".to_string());
		}

		// Ensure channel name contains only valid characters
		if !self
			.channel
			.chars()
			.all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')

		{

			return Err("Channel contains invalid characters".to_string());
		}

		// Ensure timestamp is reasonable (not in future, not too old)
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		// Maximum allowed clock skew: messages may be at most 5 seconds in the future
		// to account for minor clock desynchronization between Wind and Mountain.
		const MAX_FUTURE_MS:u64 = 5_000;

		// Maximum message age: reject messages older than 1 hour to prevent
		// replay attacks using captured old messages.
		const MAX_AGE_MS:u64 = 3600_000;

		if This.timestamp > now + MAX_FUTURE_MS {

			return Err("Timestamp is too far in the future".to_string());
		}

		if This.timestamp < now.saturating_sub(MAX_AGE_MS) {

			return Err("Timestamp is too old".to_string());
		}

		Ok(())
	}
