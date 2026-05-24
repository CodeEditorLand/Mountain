//! `ManageConnection::IsTimedOut`

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

pub fn Fn(This:&Struct) -> bool {

		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

		match This.LastActivity.lock() {

			Ok(activity) => {

				let elapsed = now.saturating_sub(*activity);

				elapsed > This.TimeoutSeconds
			},

			Err(_) => false,
		}
	}
