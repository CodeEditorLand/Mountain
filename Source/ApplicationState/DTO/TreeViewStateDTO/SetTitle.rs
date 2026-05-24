//! `TreeViewStateDTO::SetTitle`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

pub fn Fn(This:&mut Struct, Title:String) -> Result<(), String> {
		if Title.len() > MAX_TITLE_LENGTH {
			return Err(format!("Title exceeds maximum length of {} bytes", MAX_TITLE_LENGTH));
		}

		This.Title = Some(Title);

		Ok(())
	}
