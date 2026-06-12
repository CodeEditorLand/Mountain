//! # Message Types (IPC)
//!
//! ## RESPONSIBILITIES
//! Defines the core data structures used for IPC communication
//! between Wind (frontend) and Mountain (backend). It provides type-safe
//! message formats that are serialized/deserialized for transport across the
//! IPC boundary.
//!
//! ## ARCHITECTURAL ROLE
//! Defines the contract for all IPC messages. It's the foundation
//! of the IPC communication layer, ensuring type safety and consistency across
//! the Wind-Mountain bridge.
//!
//! ## KEY COMPONENTS
//!
//! - **TauriIPCMessage**: Standard message format for all IPC communication
//! - **ConnectionStatus**: Connection health status reporting
//! - **ListenerCallback**: Type definition for message event listeners
//!
//! ## ERROR HANDLING
//! Message types use serde for serialization/deserialization. Invalid messages
//! will fail to parse with descriptive error messages.
//!
//! ## LOGGING
//! Debug-level logging for message metadata, trace for detailed message
//! inspection.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Messages use efficient serde_json::Value for flexible data payloads
//! - Timestamp uses u64 for compact representation
//! - Option<> used for optional fields to minimize serialization overhead

use serde::{Deserialize, Serialize};

/// IPC message structure matching Wind's ITauriIPCMessage interface
///
/// This is the standard message format for all communication between Wind
/// (TypeScript frontend) and Mountain (Rust backend).
///
/// ## Message Flow
///
/// ```text
/// Wind Frontend
///     |
///     | 2. Serialize to JSON
///     v
/// Tauri Bridge (Webview)
///     |
///     | 1. Create TauriIPCMessage
///     v
/// TauriIPCServer (Rust)
///     |
///     | 3. Deserialize and route
///     v
/// Mountain Services
/// ```
///
/// ## Example Usage
///
/// ```rust,ignore
/// let message = TauriIPCMessage {
///     channel: "mountain_file_read".to_string(),
///     data: serde_json::json!({ "path": "/path/to/file" }),
///     sender: Some("wind-frontend".to_string()),
///     timestamp: SystemTime::now()
///         .duration_since(UNIX_EPOCH)
///         .unwrap()
///         .as_millis() as u64,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriIPCMessage {
	/// IPC channel identifier that determines which handler processes the
	/// message
	pub channel:String,

	/// Message payload data in flexible JSON format
	pub data:serde_json::Value,

	/// Optional sender identifier for tracking message origin
	pub sender:Option<String>,

	/// Unix timestamp in milliseconds for message ordering and debugging
	pub timestamp:u64,
}

impl TauriIPCMessage {
	/// Create a new IPC message
	///
	/// ## Parameters
	/// - `channel`: The IPC channel name
	/// - `data`: The message payload
	/// - `sender`: Optional sender identifier
	///
	/// ## Returns
	/// A new TauriIPCMessage with current timestamp
	pub fn new(channel:String, data:serde_json::Value, sender:Option<String>) -> Self {
		Self {
			channel,

			data,

			sender,

			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		}
	}

	/// Check if message is from a specific sender
	pub fn is_from(&self, sender:&str) -> bool { self.sender.as_deref() == Some(sender) }

	/// Get message age in milliseconds
	pub fn age_ms(&self) -> u64 {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		now.saturating_sub(self.timestamp)
	}
}

/// Connection status message for health monitoring
///
/// Is used to report the IPC connection status between Wind
/// and Mountain, enabling the frontend to display connection state to users.
///
/// ## Status Reporting Flow
///
/// ```text
/// Mounntain IPC Server
///     |
///     | 1. Detect connection change
///     v
/// ConnectionStatus
///     |
///     | 2. Emit via IPC
///     v
/// Wind Frontend
///     |
///     | 3. Update UI
///     v
/// User (see connection status)
/// ```
/// Simple connection status message for health monitoring
///
/// Is used to report the IPC connection status between Wind
/// and Mountain, enabling the frontend to display connection state to users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConnectionStatus {
	/// Whether the IPC connection is currently active
	pub connected:bool,
}

impl SimpleConnectionStatus {
	/// Create a new connection status
	pub fn new(connected:bool) -> Self { Self { connected } }

	/// Get human-readable status description
	pub fn description(&self) -> &'static str {
		if self.connected {
			"Connected to Mountain"
		} else {
			"Disconnected from Mountain"
		}
	}
}

/// Listener callback type for handling incoming IPC messages
///
/// Defines the signature for callbacks that can be registered
/// to handle messages on specific IPC channels.
///
/// ## Callback Signature
///
/// ```rust,ignore
/// pub type ListenerCallback = Box<dyn Fn(serde_json::Value) -> Result<(), String> + Send + Sync>;
/// ```
///
/// ## Example Usage
///
/// ```rust,ignore
/// // Register a listener for file operations
/// ipc_server.on("mountain_file_read", Box::new(|data| {
///     // Handle file read request
///     Ok(())
/// }))?;
/// ```
pub type ListenerCallback = Box<dyn Fn(serde_json::Value) -> Result<(), String> + Send + Sync>;

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_message_creation() {
		let message = TauriIPCMessage::new(
			"test_channel".to_string(),
			serde_json::json!({ "key": "value" }),
			Some("test_sender".to_string()),
		);

		assert_eq!(message.channel, "test_channel");

		assert!(message.is_from("test_sender"));
	}

	#[test]
	fn test_message_age() {
		let message = TauriIPCMessage::new("test_channel".to_string(), serde_json::json!({}), None);

		// Age should be small (less than 100ms)
		assert!(message.age_ms() < 100);
	}

	#[test]
	fn test_connection_status() {
		let status = SimpleConnectionStatus::new(true);

		assert!(status.connected);

		assert_eq!(status.description(), "Connected to Mountain");

		let status = SimpleConnectionStatus::new(false);

		assert!(!status.connected);

		assert_eq!(status.description(), "Disconnected from Mountain");
	}
}
