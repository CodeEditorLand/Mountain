//! `CustomDocumentStateDTO::AddEdit`

use super::Struct;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

pub fn Fn(This:&mut Struct, EditID:u32, EditData:serde_json::Value) -> Result<(), String> {
		if This.Edits.len() >= MAX_EDITS_PER_DOCUMENT {
			return Err(format!("Maximum edit limit of {} reached for document", MAX_EDITS_PER_DOCUMENT));
		}

		This.Edits.insert(EditID, EditData);

		Ok(())
	}
