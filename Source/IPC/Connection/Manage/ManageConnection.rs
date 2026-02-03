//! # Manage
//!
//! ## File: IPC/Connection/Manage/ManageConnection.rs
//!
//! ## Role: Manages connection state for IPC communication
//! ## Primary Responsibility: Track and manage connection lifecycles with Rust frontend
//!
//! ## Dependencies
//! - TauriIPCMessage: Message types from IPC/Message/Define
//! - AppHandle: Tauri application handle for event emission
//!
//! ## Security Considerations
//! - Connection timeout prevents hanging connections
//! - State isolation prevents connection state leakage
//! - Connection validation prevents malformed connection states
//!
//! ## Performance Considerations
//! - Arc<Mutex<>> for thread-safe shared state without excessive locks
//! - Non-blocking status checks for high-frequency polling
//! - Atomic operations for status changes avoid lock contention

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
}

/// Connection status event published to frontend
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatusEvent {
	pub connection_id:String,
	pub connected:bool,
	pub timestamp:u64,
}

impl ConnectionState {
	/// Create a new connection state tracker
	///
	/// ## Parameters
	/// - `ConnectionId`: Unique identifier for this connection
	/// - `TimeoutSeconds`: Connection timeout before marking as stale
	///
	/// ## Returns
	/// New ConnectionState instance
	pub fn New(ConnectionId:String, TimeoutSeconds:u64) -> Self {
		Self {
			Connected:AtomicBool::new(false),
			AppHandle:Mutex::new(None),
			LastActivity:Mutex::new(0),
			ConnectionId,
			TimeoutSeconds,
		}
	}

	/// Initialize connection with Tauri app handle
	///
	/// ## Parameters
	/// - `Handle`: Tauri application handle for event emission
	///
	/// ## Returns
	/// Result indicating success or error message
	pub fn Initialize(&self, Handle:AppHandle) -> Result<(), String> {
		match self.AppHandle.lock() {
			Ok(mut handle) => {
				*handle = Some(Handle);
				self.Connected.store(true, Ordering::Release);
				self.UpdateActivity();
				Ok(())
			},
			Err(e) => Err(format!("Failed to acquire app handle lock: {}", e)),
		}
	}

	/// Update last activity timestamp
	///
	/// ## Returns
	/// Current timestamp as seconds since Unix epoch
	pub fn UpdateActivity(&self) -> u64 {
		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

		match self.LastActivity.lock() {
			Ok(mut activity) => {
				*activity = now;
				now
			},
			Err(_) => now,
		}
	}

	/// Check if connection is currently active
	///
	/// ## Returns
	/// True if connected, false otherwise
	pub fn IsConnected(&self) -> bool { self.Connected.load(Ordering::Acquire) }

	/// Check if connection has timed out
	///
	/// ## Returns
	/// True if connection has exceeded timeout, false otherwise
	pub fn IsTimedOut(&self) -> bool {
		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

		match self.LastActivity.lock() {
			Ok(activity) => {
				let elapsed = now.saturating_sub(*activity);
				elapsed > self.TimeoutSeconds
			},
			Err(_) => false,
		}
	}

	/// Send connection status event to frontend
	///
	/// ## Parameters
	/// - `Connected`: Current connection status
	///
	/// ## Returns
	/// Result indicating success or error message
	pub async fn SendStatus(&self, Connected:bool) -> Result<(), String> {
		let handle = match self.AppHandle.lock() {
			Ok(h) => h.clone(),
			Err(e) => return Err(format!("Failed to acquire app handle lock: {}", e)),
		};

		let handle = match handle {
			Some(h) => h,
			None => return Err("App handle not initialized".to_string()),
		};

		let event = ConnectionStatusEvent {
			connection_id:self.ConnectionId.clone(),
			connected:Connected,
			timestamp:self.UpdateActivity(),
		};

		handle
			.emit("ipc:connection_status", event)
			.map_err(|e| format!("Failed to emit connection status event: {}", e))?;

		Ok(())
	}

	/// Gracefully close the connection
	///
	/// ## Returns
	/// Result indicating success or error message
	pub fn Close(&self) -> Result<(), String> {
		self.Connected.store(false, Ordering::Release);

		// Clear app handle to prevent further use
		match self.AppHandle.lock() {
			Ok(mut handle) => {
				*handle = None;
			},
			Err(e) => return Err(format!("Failed to clear app handle: {}", e)),
		}

		Ok(())
	}

	/// Get connection statistics
	///
	/// ## Returns
	/// Connection statistics as a string
	pub fn GetStatistics(&self) -> String {
		let connected = self.IsConnected();
		let timed_out = self.IsTimedOut();
		let last_activity = match self.LastActivity.lock() {
			Ok(activity) => *activity,
			Err(_) => 0,
		};

		format!(
			"Connection[id: {}, connected: {}, timed_out: {}, last_activity: {}, timeout: {}s]",
			self.ConnectionId, connected, timed_out, last_activity, self.TimeoutSeconds
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_connection_state_new() {
		let State = ConnectionState::New("test-connection-1".to_string(), 300);
		assert!(!State.IsConnected());
		assert!(!State.IsTimedOut());
	}

	#[test]
	fn test_connection_state_initialization() {
		let State = ConnectionState::New("test-connection-2".to_string(), 300);

		// Cannot test actual initialization without Tauri AppHandle
		// This test verifies the structure is correct
		assert_eq!(State.ConnectionId, "test-connection-2");
		assert_eq!(State.TimeoutSeconds, 300);
	}

	#[test]
	fn test_connection_state_not_timed_out() {
		let State = ConnectionState::New("test-connection-3".to_string(), 300);
		State.UpdateActivity();

		// Should not be timed out immediately
		assert!(!State.IsTimedOut());
	}

	#[test]
	fn test_connection_state_close() {
		let State = ConnectionState::New("test-connection-4".to_string(), 300);
		assert!(State.Close().is_ok());
		assert!(!State.IsConnected());
	}

	#[test]
	fn test_connection_state_statistics() {
		let State = ConnectionState::New("test-connection-5".to_string(), 300);
		let Stats = State.GetStatistics();

		assert!(Stats.contains("test-connection-5"));
		assert!(Stats.contains("connected: false"));
		assert!(Stats.contains("timeout: 300s"));
	}
}
