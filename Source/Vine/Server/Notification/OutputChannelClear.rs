#![allow(non_snake_case)]
//! Cocoon → Mountain `outputChannel.clear` notification (twin of
//! `output.clear`).

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelClear(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/clear", Parameter);

	dev_log!(
		"grpc",
		"[OutputChannel] clear channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
