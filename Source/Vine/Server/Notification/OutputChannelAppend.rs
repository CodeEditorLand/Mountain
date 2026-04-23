#![allow(non_snake_case)]
//! Cocoon → Mountain `outputChannel.append` notification.
//! Twin of `output.append`; see `OutputCreate.rs` for the duplicate-wire
//! rationale.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelAppend(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/append", Parameter);
	dev_log!(
		"grpc",
		"[OutputChannel] append channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
