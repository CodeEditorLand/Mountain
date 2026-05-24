//! `ProviderRegistrationDTO::MatchesSelector`

use super::Struct;
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(This:&Struct, _DocumentURI:&str, LanguageIdentifier:&str) -> bool {
		// This is a simplified matching logic
		// A full implementation would traverse the selector value
		if let Some(SelectorObj) = This.Selector.as_object() {
			if let Some(Languages) = SelectorObj.get("language").and_then(Value::as_array) {
				return Languages
					.iter()
					.any(|Lang| Lang.as_str().map_or(false, |L| L == LanguageIdentifier));
			}
		}

		false
	}
