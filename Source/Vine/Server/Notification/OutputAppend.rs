#![allow(non_snake_case)]
//! Cocoon → Mountain `output.append` notification.
//! Emitted by `Cocoon/.../Services/Window/OutputChannel.ts:50` whenever
//! an extension calls `OutputChannel.append(text)`. Forwards verbatim to
//! `sky://output/append` - the Sky listener mirrors the text into both
//! the in-memory `OutputChannels` map and VS Code's logger sink.

use serde_json::Value;

use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputAppend(Service:&MountainVinegRPCService, Parameter:&Value) {

	let _ = Service.ApplicationHandle().emit("sky://output/append", Parameter);

	dev_log!(
		"grpc",

		"[Output] append channel={} bytes={}",

		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?"),

		Parameter.get("text").and_then(Value::as_str).map(str::len).unwrap_or(0)
	);
}
