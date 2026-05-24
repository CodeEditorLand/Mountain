//! `DecorationsState::SetDecoration`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct, Uri:&str, Decoration:Value) {
		if let Ok(mut Guard) = This.Entries.lock() {
			Guard.insert(Uri.to_owned(), Decoration);

			dev_log!("decorations", "[DecorationsState] Decoration set for: {}", Uri);
		}
	}
