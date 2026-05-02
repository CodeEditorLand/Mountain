#![allow(non_snake_case)]

//! Webview message envelope: identifier, type tag, payload, and an
//! optional source identifier. Used for host ↔ webview message
//! passing routed through the Tauri event bus.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub MessageIdentifier:String,
	pub MessageType:String,
	pub Payload:Value,
	pub Source:Option<String>,
}
