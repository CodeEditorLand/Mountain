//! `WorkspaceFolderStateDTO::New`

use super::Struct;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(URI:Url, Name:String, Index:usize) -> Result<Self, String> {
		// Validate URI is not empty
		if URI.as_str().is_empty() {
			return Err("URI cannot be empty".to_string());
		}

		// Validate name length
		if Name.len() > MAX_FOLDER_NAME_LENGTH {
			return Err(format!(
				"Folder name exceeds maximum length of {} bytes",
				MAX_FOLDER_NAME_LENGTH
			));
		}

		// Validate index range
		if Index >= MAX_WORKSPACE_FOLDERS {
			return Err(format!(
				"Folder index {} exceeds maximum workspace folders count of {}",
				Index, MAX_WORKSPACE_FOLDERS
			));
		}

		Ok(Self { URI, Name, Index })
	}
