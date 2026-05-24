//! `TreeViewState::Get`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) -> Option<TreeViewStateDTO> {
		This.ActiveTreeViews.lock().ok().and_then(|guard| guard.get(id).cloned())
	}
