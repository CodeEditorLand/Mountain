//! `TreeViewState::Contains`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) -> bool {
		This.ActiveTreeViews
			.lock()
			.ok()
			.map(|guard| guard.contains_key(id))
			.unwrap_or(false)
	}
