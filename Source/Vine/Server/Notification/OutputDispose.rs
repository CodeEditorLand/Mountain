#![allow(non_snake_case)]
//! Cocoon → Mountain `output.dispose` notification.
//! Forwarding atom for `OutputChannel.dispose()` - removes the channel
//! from Sky's `OutputChannels` map.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputDispose(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/dispose", Parameter);

	dev_log!(
		"grpc",
		"[Output] dispose channel={}",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
