//! `OutputChannelStateDTO::Create`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(Name:&str, LanguageIdentifier:Option<String>) -> Result<Self, String> {
		// Validate name length
		if Name.len() > MAX_CHANNEL_NAME_LENGTH {
			return Err(format!(
				"Channel name exceeds maximum length of {} bytes",
				MAX_CHANNEL_NAME_LENGTH
			));
		}

		// Validate language identifier length
		if let Some(LangID) = &LanguageIdentifier {
			if LangID.len() > MAX_LANGUAGE_ID_LENGTH {
				return Err(format!(
					"Language identifier exceeds maximum length of {} bytes",
					MAX_LANGUAGE_ID_LENGTH
				));
			}
		}

		Ok(Self { Name:Name.to_string(), LanguageIdentifier, Buffer:String::new(), IsVisible:false })
	}
