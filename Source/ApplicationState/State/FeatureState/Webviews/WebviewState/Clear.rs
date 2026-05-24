//! `WebviewState::Clear`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.ActiveWebviews.lock() {
			guard.clear();

			dev_log!("extensions", "[WebviewState] All webviews cleared");
		}
	}
