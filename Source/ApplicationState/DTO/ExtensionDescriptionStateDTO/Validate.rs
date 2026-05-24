//! `ExtensionDescriptionStateDTO::Validate`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(This:&Struct) -> Result<(), String> {
		// Validate Name length
		if This.Name.len() > MAX_EXTENSION_NAME_LENGTH {
			return Err(format!(
				"Extension name exceeds maximum length of {} bytes",
				MAX_EXTENSION_NAME_LENGTH
			));
		}

		// Validate Version length
		if This.Version.len() > MAX_VERSION_LENGTH {
			return Err(format!("Version string exceeds maximum length of {} bytes", MAX_VERSION_LENGTH));
		}

		// Validate Publisher length
		if This.Publisher.len() > MAX_PUBLISHER_LENGTH {
			return Err(format!("Publisher exceeds maximum length of {} bytes", MAX_PUBLISHER_LENGTH));
		}

		// Validate ActivationEvents count
		if let Some(Events) = &This.ActivationEvents {
			if Events.len() > MAX_ACTIVATION_EVENTS {
				return Err(format!("Activation events exceed maximum count of {}", MAX_ACTIVATION_EVENTS));
			}
		}

		Ok(())
	}
