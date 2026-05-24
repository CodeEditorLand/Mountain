//! `TreeViewStateDTO::SetDescription`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

pub fn Fn(This:&mut Struct, Description:String) -> Result<(), String> {
		if Description.len() > MAX_DESCRIPTION_LENGTH {
			return Err(format!(
				"Description exceeds maximum length of {} bytes",
				MAX_DESCRIPTION_LENGTH
			));
		}

		This.Description = Some(Description);

		Ok(())
	}
