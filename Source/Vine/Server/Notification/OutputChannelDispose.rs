#![allow(non_snake_case)]
//! Cocoon → Mountain `outputChannel.dispose` notification (twin of
//! `output.dispose`).

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelDispose(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/dispose", Parameter);
	dev_log!(
		"grpc",
		"[OutputChannel] dispose channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
