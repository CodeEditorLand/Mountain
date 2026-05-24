//! `TreeViewState::AddOrUpdate`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:String, tree_view:TreeViewStateDTO) {
		if let Ok(mut guard) = This.ActiveTreeViews.lock() {
			guard.insert(id, tree_view);

			dev_log!("extensions", "[TreeViewState] Tree view added/updated");
		}
	}
