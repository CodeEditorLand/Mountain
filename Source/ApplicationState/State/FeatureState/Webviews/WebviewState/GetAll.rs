//! `WebviewState::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<String, WebviewStateDTO> {
		This.ActiveWebviews.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}
