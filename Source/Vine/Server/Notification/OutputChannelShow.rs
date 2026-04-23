#![allow(non_snake_case)]
//! Cocoon → Mountain `outputChannel.show` notification (twin of
//! `output.show`).

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelShow(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/show", Parameter);
	dev_log!(
		"grpc",
		"[OutputChannel] show channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
