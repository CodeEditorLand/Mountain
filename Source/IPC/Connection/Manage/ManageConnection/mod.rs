pub mod New;
pub mod Initialize;
pub mod UpdateActivity;
pub mod IsConnected;
pub mod IsTimedOut;
pub mod SendStatus;
pub mod Close;
pub mod GetStatistics;

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

/// Connection state tracker for IPC server
pub struct ConnectionState {

	/// Current connection status (thread-safe atomic)
	pub Connected:AtomicBool,

	/// Tauri application handle for event emission
	pub AppHandle:Mutex<Option<AppHandle>>,

	/// Last activity timestamp for connection health monitoring
	pub LastActivity:Mutex<u64>,

	/// Connection ID for distinguishing multiple connections
	pub ConnectionId:String,

	/// Connection timeout in seconds
	pub TimeoutSeconds:u64,

/// Connection status event published to frontend
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatusEvent {

	pub connection_id:String,

	pub connected:bool,

	pub timestamp:u64,
}
}

#[derive(Debug, Clone)]
pub struct Struct;
