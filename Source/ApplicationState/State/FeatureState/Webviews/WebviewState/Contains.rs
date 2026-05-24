//! `WebviewState::Contains`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) -> bool {
		This.ActiveWebviews
			.lock()
			.ok()
			.map(|guard| guard.contains_key(id))
			.unwrap_or(false)
	}
