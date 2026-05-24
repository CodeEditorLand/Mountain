//! `DecorationsState::ClearDecoration`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct, Uri:&str) {
		if let Ok(mut Guard) = This.Entries.lock() {
			Guard.remove(Uri);

			dev_log!("decorations", "[DecorationsState] Decoration cleared for: {}", Uri);
		}
	}
