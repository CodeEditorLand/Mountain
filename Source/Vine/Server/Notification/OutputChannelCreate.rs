#![allow(non_snake_case)]
//! Cocoon → Mountain `outputChannel.create` notification.
//! Parallel wire name to `output.create` used by Cocoon's
//! `SendToMountain` call sites. The two should converge to one in a
//! future Cocoon refactor; for now Mountain handles both by routing
//! into the same `sky://output/create` channel.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelCreate(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/create", Parameter);
	dev_log!(
		"grpc",
		"[OutputChannel] create id={}",
		Parameter.get("id").and_then(Value::as_str).unwrap_or("?")
	);
}
