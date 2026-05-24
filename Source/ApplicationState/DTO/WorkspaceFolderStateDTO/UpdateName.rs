//! `WorkspaceFolderStateDTO::UpdateName`

use super::Struct;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(This:&mut Struct, Name:String) -> Result<(), String> {
		if Name.len() > MAX_FOLDER_NAME_LENGTH {
			return Err(format!(
				"Folder name exceeds maximum length of {} bytes",
				MAX_FOLDER_NAME_LENGTH
			));
		}

		This.Name = Name;

		Ok(())
	}
