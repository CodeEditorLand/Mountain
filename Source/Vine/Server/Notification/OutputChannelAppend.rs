#![allow(non_snake_case)]
//! Cocoon → Mountain `outputChannel.append` notification.
//! Twin of `output.append`; see `OutputCreate.rs` for the duplicate-wire
//! rationale.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelAppend(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/append", Parameter);
	// Per-append fire - `roo-cline`, `TypeScript`, `dart-code` all stream
	// stdout into their output channels which fires 200+ appends per
	// boot. The Sky-side consumer already sees the data via
	// `sky://output/append`; the tag line here adds no signal beyond
	// volume. Route to `output-verbose`.
	dev_log!(
		"output-verbose",
		"[OutputChannel] append channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
