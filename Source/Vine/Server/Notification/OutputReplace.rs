//! Cocoon → Mountain `output.replace` notification.
//! Emitted when an extension calls `LogOutputChannel.replace(value)` to
//! swap the channel's entire contents. Sky doesn't yet have a dedicated
//! `sky://output/replace` listener, so this atom maps replace → (clear +
//! append) on the existing channels. That preserves semantics without
//! requiring a coordinated Sky-side change.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputReplace(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Channel = Parameter.get("channel").and_then(Value::as_str).unwrap_or("");

	let Text = Parameter.get("text").and_then(Value::as_str).unwrap_or("");

	let Handle = Service.ApplicationHandle();

	let _ = Handle.emit("sky://output/clear", json!({ "channel": Channel }));

	let _ = Handle.emit("sky://output/append", json!({ "channel": Channel, "text": Text }));

	dev_log!("grpc", "[Output] replace channel={} bytes={}", Channel, Text.len());
}
