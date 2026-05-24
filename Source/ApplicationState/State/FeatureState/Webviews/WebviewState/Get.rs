//! `WebviewState::Get`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) -> Option<WebviewStateDTO> {
		This.ActiveWebviews.lock().ok().and_then(|guard| guard.get(id).cloned())
	}
