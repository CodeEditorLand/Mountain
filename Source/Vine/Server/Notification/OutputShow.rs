#![allow(non_snake_case)]
//! Cocoon → Mountain `output.show` notification.
//! Forwarding atom for `OutputChannel.show(preserveFocus?)`.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputShow(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/show", Parameter);
	dev_log!(
		"grpc",
		"[Output] show channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
