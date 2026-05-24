//! `ProviderRegistrationDTO::New`

use super::Struct;
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(
		Handle:u32,

		ProviderType:ProviderType,

		Selector:Value,

		SideCarIdentifier:String,

		ExtensionIdentifier:Value,
	) -> Result<Self, String> {
		// Validate sidecar identifier length
		if SideCarIdentifier.len() > MAX_SIDECAR_IDENTIFIER_LENGTH {
			return Err(format!(
				"SideCarIdentifier exceeds maximum length of {} bytes",
				MAX_SIDECAR_IDENTIFIER_LENGTH
			));
		}

		Ok(Self {
			Handle,
			ProviderType,
			Selector,
			SideCarIdentifier,
			ExtensionIdentifier,
			Options:None,
		})
	}
