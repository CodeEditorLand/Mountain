//! Webview message envelope: identifier, type tag, payload, and an
//! optional source identifier. Used for host ↔ webview message
//! passing routed through the Tauri event bus.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Message envelope for host ↔ webview IPC.
///
/// Carries a unique identifier, a type tag for dispatch, the JSON
/// payload, and an optional source identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	/// Unique identifier for this message.
	pub MessageIdentifier:String,

	/// Type tag for message routing and dispatch.
	pub MessageType:String,

	/// JSON payload content.
	pub Payload:Value,

	/// Optional source identifier (e.g. extension or webview ID).
	pub Source:Option<String>,
}
