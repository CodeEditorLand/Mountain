//! `WebviewStateDTO::UpdateTitle`

use super::Struct;
use CommonLibrary::Webview::DTO::WebviewContentOptionsDTO::WebviewContentOptionsDTO;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(This:&mut Struct, Title:String) -> Result<(), String> {
		if Title.len() > MAX_TITLE_LENGTH {
			return Err(format!("Title exceeds maximum length of {} bytes", MAX_TITLE_LENGTH));
		}

		This.Title = Title;

		Ok(())
	}
