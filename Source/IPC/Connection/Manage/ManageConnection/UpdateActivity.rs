//! `ManageConnection::UpdateActivity`

use super::Struct;
use std::{
	sync::{
		Arc,
		Mutex,
		atomic::{AtomicBool, Ordering},
	},
	time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use serde::Serialize;

pub fn Fn(This:&Struct) -> u64 {

		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

		match This.LastActivity.lock() {

			Ok(mut activity) => {

				*activity = now;
				now
			},

			Err(_) => now,
		}
	}
