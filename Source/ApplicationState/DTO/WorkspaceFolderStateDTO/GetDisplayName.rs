//! `WorkspaceFolderStateDTO::GetDisplayName`

use super::Struct;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(This:&Struct) -> String {
		if !This.Name.is_empty() {
			This.Name.clone()
		} else {
			// Extract folder name from URI
			This.URI
				.path_segments()
				.and_then(|Segments| Segments.last())
				.unwrap_or("Untitled")
				.to_string()
		}
	}
