//! `TreeViewState::Remove`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) {
		if let Ok(mut guard) = This.ActiveTreeViews.lock() {
			guard.remove(id);

			dev_log!("extensions", "[TreeViewState] Tree view removed: {}", id);
		}
	}
