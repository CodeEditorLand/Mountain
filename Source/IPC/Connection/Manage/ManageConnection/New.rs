//! `ManageConnection::New`

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

pub fn Fn(ConnectionId:String, TimeoutSeconds:u64) -> Struct {

		Self {

			Connected:AtomicBool::new(false),

			AppHandle:Mutex::new(None),

			LastActivity:Mutex::new(0),

			ConnectionId,

			TimeoutSeconds,
		}
	}
