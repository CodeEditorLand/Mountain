//! `WebviewState::Remove`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:&str) {
		if let Ok(mut guard) = This.ActiveWebviews.lock() {
			guard.remove(id);

			dev_log!("extensions", "[WebviewState] Webview removed: {}", id);
		}
	}
