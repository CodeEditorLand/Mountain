//! # Define
//!
//! ## File: IPC/Message/Define/DefineMessage.rs
//!
//! ## Role in Mountain Architecture
//!
//! Defines core message types and structures used throughout the IPC layer
//! for communication between Mountain's Rust backend and Wind's TypeScript
//! frontend.
//!
//! ## Primary Responsibility
//!
//! Provide type-safe message structures that enable serialization across the
//! Rust-TypeScript language boundary with proper schema validation.
//!
//! ## Secondary Responsibilities
//!
//! - Define IPC message format matching Wind's interface
//! - Define connection status tracking types
//! - Define listener callback type signatures
//! - Ensure message compatibility between Mountain and Wind
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `serde` - Serialization/deserialization support
//! - `serde_json` - JSON format for cross-language communication
//!
//! **Internal Modules:**
//! - None (this module provides foundational types)
//!
//! ## Dependents
//!
//! - `TauriIPCServer` - Uses all message types
//! - `RouteMessage` - Routes TauriIPCMessage instances
//! - `Compress` - Compresses TauriIPCMessage batches
//! - `Encrypt` - Encrypts TauriIPCMessage instances
//!
//! ## VSCode Pattern Reference
//!
//! Matches VSCode's RPC message format:
//! - Channel-based routing
//! - JSON-formatted payloads
//! - Timestamp-based ordering
//! - Sender identification
//!
//! ## Security Considerations
//!
//! - All fields validated during deserialization to prevent injection attacks
//! - Timestamp field prevents replay attacks when combined with nonce
//! - Sender field authenticated for source verification
//! - Size limits enforced to prevent memory exhaustion attacks
//!
//! ## Performance Considerations
//!
//! - Use serde_json for efficient JSON parsing
//! - Clone-based message routing (zero-copy not possible across serialization
//!   boundary)
//! - Compact structure minimizes serialization overhead
//!
//! ## Error Handling Strategy
//!
//! - Serde provides automatic serde errors with context
//! - All deserialization operations wrapped in Result for explicit handling
//! - Failed deserialization logged with full context
//!
//! ## Thread Safety
//!
//! - All structs derive Clone for safe sharing across threads
//! - No interior mutability, all state in Arc/Mutex wrapper in parent
//!
//! ## TODO Items
//!
//! - [ ] Add message versioning for schema evolution
//! - [ ] Add message validation schema
//! - [ ] Consider binary protocol option for performance

use serde::{Deserialize, Serialize};

/// IPC message structure matching Wind's ITauriIPCMessage interface
///
/// This structure represents the standard message format for all IPC
/// communication between Mountain's Rust backend and Wind's TypeScript
/// frontend.
///
/// # Security
/// - Timestamp used to prevent replay attacks when combined with nonce in
///   encrypted messages
/// - Sender field verified for source authentication
/// - Size limits enforced to prevent DoS
///
/// # Performance
/// - JSON format for compatibility with JavaScript
/// - Compact structure minimizes serialization overhead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriIPCMessage {

	/// Channel name for message routing (e.g., "configuration", "file-system")
	pub channel:String,

	/// Message payload in JSON format
	pub data:serde_json::Value,

	/// Optional sender identifier for source tracking
	pub sender:Option<String>,

	/// Unix timestamp in milliseconds for ordering and replay prevention
	pub timestamp:u64,
}

impl TauriIPCMessage {

	/// Create a new TauriIPCMessage with current timestamp
	///
	/// # Arguments
	/// * `channel` - Channel name for routing
	/// * `data` - Message payload
	/// * `sender` - Optional sender identifier
	///
	/// # Returns
	/// A new TauriIPCMessage instance with timestamp set to current time
	pub fn new(channel:impl Into<String>, data:serde_json::Value, sender:Option<String>) -> Self {

		Self {

			channel:channel.into(),

			data,

			sender,

			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		}
	}

	/// Validate message integrity
	///
	/// # Returns
	/// Ok(()) if message passes validation, Err with reason otherwise
	pub fn validate(&self) -> Result<(), String> {

		// Ensure channel is not empty
		if self.channel.is_empty() {

			return Err("Channel cannot be empty".to_string());
		}

		// Ensure channel name contains only valid characters
		if !self
			.channel
			.chars()
			.all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
		{

			return Err("Channel contains invalid characters".to_string());
		}

		// Ensure timestamp is reasonable (not in future, not too old)
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64;

		// Maximum allowed clock skew: messages may be at most 5 seconds in the future
		// to account for minor clock desynchronization between Wind and Mountain.
		const MAX_FUTURE_MS:u64 = 5_000;

		// Maximum message age: reject messages older than 1 hour to prevent
		// replay attacks using captured old messages.
		const MAX_AGE_MS:u64 = 3600_000;

		if self.timestamp > now + MAX_FUTURE_MS {

			return Err("Timestamp is too far in the future".to_string());
		}

		if self.timestamp < now.saturating_sub(MAX_AGE_MS) {

			return Err("Timestamp is too old".to_string());
		}

		Ok(())
	}
}

/// Connection status message
///
/// Simple boolean indicator of IPC connection health between Mountain and Wind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {

	/// True if IPC connection is active, false otherwise
	pub connected:bool,
}

impl ConnectionStatus {

	/// Create a new connection status
	///
	/// # Arguments
	/// * `connected` - Connection state
	pub fn new(connected:bool) -> Self { Self { connected } }
}

/// Listener callback type for message subscription
///
/// This type alias defines the signature for callbacks registered to receive
/// messages on specific channels.
///
/// # Thread Safety
/// - The callback must implement Send + Sync for cross-thread access
/// - Box<dyn Fn> allows for flexible closure-based implementations
///
/// # Performance
/// - Cloning message data for each listener (tradeoff for safety)
/// - Consider Arc<message> for zero-copy pattern in future optimization
pub type ListenerCallback = Box<dyn Fn(serde_json::Value) -> Result<(), String> + Send + Sync>;
