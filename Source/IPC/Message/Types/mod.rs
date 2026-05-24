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
//!
//! ## TODO
//! - Add message payload size limits
//! - Implement message versioning for compatibility
//! - Add message priority field
pub mod New;
pub mod IsFrom;
pub mod AgeMs;
pub mod New;
pub mod Description;

use serde::{Deserialize, Serialize};


/// IPC message structure matching Wind's ITauriIPCMessage interface
/// This is the standard message format for all communication between Wind
/// (TypeScript frontend) and Mountain (Rust backend).
/// ## Message Flow
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
/// ## Example Usage
/// ```rust,ignore
/// let Message = TauriIPCMessage {
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

/// Connection status message for health monitoring
/// This structure is used to report the IPC connection status between Wind
/// and Mountain, enabling the frontend to display connection state to users.
/// ## Status Reporting Flow
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
/// This structure is used to report the IPC connection status between Wind
/// and Mountain, enabling the frontend to display connection state to users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConnectionStatus {
	/// Whether the IPC connection is currently active
	pub connected:bool,
}

#[derive(Debug, Clone)]
pub struct Struct;
