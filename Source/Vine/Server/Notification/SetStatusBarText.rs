#![allow(non_snake_case)]
//! Cocoon → Mountain `setStatusBarText` notification.
//! Emitted three times by `Cocoon/.../Services/Window/StatusBar.ts`
//! (`:92`, `:123`, `:131`) whenever an extension calls
//! `vscode.window.setStatusBarMessage(...)`, or an extension-owned
//! `StatusBarItem.text = "..."` mutates. Distinct from the typed
//! `statusBar.update` notification (which carries colour/tooltip/command
//! fields): this wire form is the pure text-only fast path.
//!
//! Forwards onto `sky://statusbar/set-entry` so the Sky `StatusBar`
//! shim's existing fan-out listener picks it up without a new channel.

use serde_json::{Value, json};

use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn SetStatusBarText(Service:&MountainVinegRPCService, Parameter:&Value) {

	let Id = Parameter.get("id").and_then(Value::as_str).unwrap_or("");

	let Text = Parameter.get("text").and_then(Value::as_str).unwrap_or("");

	let Tooltip = Parameter.get("tooltip").and_then(Value::as_str).unwrap_or("");

	let _ = Service.ApplicationHandle().emit(
		"sky://statusbar/set-entry",

		json!({
			"id": Id,
			"text": Text,
			"tooltip": Tooltip,
		}),
	);

	dev_log!("grpc", "[StatusBar] set-text id={} len={}", Id, Text.len());
}
