//! `ManageConnection::SendStatus`

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

pub fn Fn(This:&Struct, Connected:bool) -> Result<(), String> {

		let Handle = match This.AppHandle.lock() {

			Ok(h) => h.clone(),

			Err(e) => return Err(format!("Failed to acquire app handle lock: {}", e)),
		};

		let Handle = match handle {

			Some(h) => h,

			None => return Err("App handle not initialized".to_string()),
		};

		let event = ConnectionStatusEvent {

			connection_id:This.ConnectionId.clone(),

			connected:Connected,

			timestamp:This.UpdateActivity(),
		};

		handle
			.emit("ipc:connection_status", event)
			.map_err(|E| format!("Failed to emit connection status event: {}", e))?;

		Ok(())
	}
