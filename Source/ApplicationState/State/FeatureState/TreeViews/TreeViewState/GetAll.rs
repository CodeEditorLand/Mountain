//! `TreeViewState::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<String, TreeViewStateDTO> {
		This.ActiveTreeViews.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}
