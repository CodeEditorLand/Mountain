//! `WebviewState::AddOrUpdate`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::WebviewStateDTO::WebviewStateDTO, dev_log};

pub fn Fn(This:&Struct, id:String, webview:WebviewStateDTO) {
		if let Ok(mut guard) = This.ActiveWebviews.lock() {
			guard.insert(id, webview);

			dev_log!("extensions", "[WebviewState] Webview added/updated");
		}
	}
