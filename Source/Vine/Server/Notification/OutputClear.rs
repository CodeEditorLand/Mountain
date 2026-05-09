#![allow(non_snake_case)]
//! Cocoon → Mountain `output.clear` notification.
//! Forwarding atom for `OutputChannel.clear()` - the Sky listener resets
//! the in-memory buffer for the matching channel.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputClear(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/clear", Parameter);

	dev_log!(
		"grpc",
		"[Output] clear channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
