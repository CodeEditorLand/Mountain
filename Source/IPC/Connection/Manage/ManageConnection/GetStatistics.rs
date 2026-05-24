//! `ManageConnection::GetStatistics`

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

pub fn Fn(This:&Struct) -> String {

		let connected = This.IsConnected();

		let timed_out = This.IsTimedOut();

		let last_activity = match This.LastActivity.lock() {

			Ok(activity) => *activity,

			Err(_) => 0,
		};

		format!(
			"Connection[id: {}, connected: {}, timed_out: {}, last_activity: {}, timeout: {}s]",

			This.ConnectionId, connected, timed_out, last_activity, This.TimeoutSeconds
		)
	}
