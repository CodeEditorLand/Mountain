#![allow(non_snake_case)]
//! Cocoon → Mountain `output.create` notification.
//! Emitted by `Cocoon/.../Services/Window/OutputChannel.ts:39` once per
//! `vscode.window.createOutputChannel(name)` call. Re-emits on the
//! canonical `sky://output/create` channel the Sky `OutputChannels` map
//! already listens for.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputCreate(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://output/create", Parameter);

	dev_log!(
		"grpc",
		"[Output] create id={} name={}",
		Parameter.get("id").and_then(Value::as_str).unwrap_or("?"),
		Parameter.get("name").and_then(Value::as_str).unwrap_or("?")
	);
}
