#![allow(non_snake_case)]
//! Cocoon → Mountain `output.appendLine` notification.
//! Emitted by `Cocoon/.../Services/Window/OutputChannel.ts:56` whenever
//! an extension calls `OutputChannel.appendLine(text)`. The stock
//! semantic contract is "append + trailing \n"; we suffix the newline
//! here so the downstream `sky://output/append` listener stays a single
//! append code path (no `appendLine` listener in Sky).

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputAppendLine(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Channel = Parameter.get("channel").and_then(Value::as_str).unwrap_or("");
	let Text = Parameter.get("text").and_then(Value::as_str).unwrap_or("");
	let _ = Service.ApplicationHandle().emit(
		"sky://output/append",
		json!({
			"channel": Channel,
			"text": format!("{}\n", Text),
		}),
	);
	dev_log!("grpc", "[Output] appendLine channel={} bytes={}", Channel, Text.len());
}
