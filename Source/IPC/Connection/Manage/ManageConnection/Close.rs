//! `ManageConnection::Close`

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

pub fn Fn(This:&Struct) -> Result<(), String> {

		This.Connected.store(false, Ordering::Release);

		// Clear app handle to prevent further use
		match This.AppHandle.lock() {

			Ok(mut handle) => {

				*handle = None;
			},

			Err(e) => return Err(format!("Failed to clear app handle: {}", e)),
		}

		Ok(())
	}
