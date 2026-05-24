//! `CustomDocumentStateDTO::New`

use super::Struct;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(URI:Url, ViewType:String, SideCarIdentifier:String, IsEditable:bool) -> Result<Self, String> {
		// Validate ViewType length
		if ViewType.len() > MAX_VIEW_TYPE_LENGTH {
			return Err(format!("ViewType exceeds maximum length of {} bytes", MAX_VIEW_TYPE_LENGTH));
		}

		// Validate SideCarIdentifier length
		if SideCarIdentifier.len() > MAX_SIDECAR_IDENTIFIER_LENGTH {
			return Err(format!(
				"SideCarIdentifier exceeds maximum length of {} bytes",
				MAX_SIDECAR_IDENTIFIER_LENGTH
			));
		}

		// Ensure URI is not empty
		if URI.as_str().is_empty() {
			return Err("URI cannot be empty".to_string());
		}

		Ok(Self {
			URI,
			ViewType,
			SideCarIdentifier,
			IsEditable,
			BackupIdentifier:None,
			Edits:HashMap::new(),
		})
	}
