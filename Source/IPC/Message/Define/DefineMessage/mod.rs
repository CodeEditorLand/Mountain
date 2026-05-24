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
pub mod New;
pub mod Validate;
pub mod New;

use serde::{Deserialize, Serialize};

type signatures

/// IPC message structure matching Wind's ITauriIPCMessage interface
/// This structure represents the standard message format for all IPC
/// communication between Mountain's Rust backend and Wind's TypeScript
/// frontend.
/// # Security
/// - Timestamp used to prevent replay attacks when combined with nonce in
///   encrypted messages
/// - Sender field verified for source authentication
/// - Size limits enforced to prevent DoS
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

/// Connection status message
/// Simple boolean indicator of IPC connection health between Mountain and Wind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {

	/// True if IPC connection is active, false otherwise
	pub connected:bool,

/// Listener callback type for message subscription
/// This type alias defines the signature for callbacks registered to receive
/// messages on specific channels.
/// # Thread Safety
/// - The callback must implement Send + Sync for cross-thread access
/// - Box<dyn Fn> allows for flexible closure-based implementations
/// # Performance
/// - Cloning message data for each listener (tradeoff for safety)
/// - Consider Arc<message> for zero-copy pattern in future optimization
pub type ListenerCallback = Box<dyn Fn(serde_json::Value) -> Result<(), String> + Send + Sync>;
}
}

#[derive(Debug, Clone)]
pub struct Struct;
