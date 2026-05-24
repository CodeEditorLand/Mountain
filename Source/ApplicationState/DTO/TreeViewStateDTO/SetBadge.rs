//! `TreeViewStateDTO::SetBadge`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

pub fn Fn(This:&mut Struct, Badge:String) -> Result<(), String> {
		if Badge.len() > MAX_BADGE_LENGTH {
			return Err(format!("Badge exceeds maximum length of {} bytes", MAX_BADGE_LENGTH));
		}

		This.Badge = Some(Badge);

		Ok(())
	}
