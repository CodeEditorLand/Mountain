//! `TreeViewStateDTO::SetMessage`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

pub fn Fn(This:&mut Struct, Message:String) -> Result<(), String> {
		if Message.len() > MAX_MESSAGE_LENGTH {
			return Err(format!("Message exceeds maximum length of {} bytes", MAX_MESSAGE_LENGTH));
		}

		This.Message = Some(Message);

		Ok(())
	}
