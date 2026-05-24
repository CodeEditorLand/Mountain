//! `ManageConnection::Initialize`

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

pub fn Fn(This:&Struct, Handle:AppHandle) -> Result<(), String> {

		match This.AppHandle.lock() {

			Ok(mut handle) => {

				*handle = Some(Handle);
				This.Connected.store(true, Ordering::Release);

				This.UpdateActivity();

				Ok(())
			},

			Err(e) => Err(format!("Failed to acquire app handle lock: {}", e)),
		}
	}
