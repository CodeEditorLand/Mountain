#![allow(non_snake_case)]
//! Cocoon → Mountain `disposeStatusBarItem` notification.
//! Emitted once by `Cocoon/.../Services/Window/StatusBar.ts:139` when an
//! extension calls `StatusBarItem.dispose()` (or the whole subscription
//! set tears down). Forwards onto the canonical
//! `sky://statusbar/dispose-entry` channel so the Sky shim's
//! fan-out listener removes the DOM node.

use serde_json::{Value, json};

use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn DisposeStatusBarItem(Service:&MountainVinegRPCService, Parameter:&Value) {

	let Id = Parameter.get("id").and_then(Value::as_str).unwrap_or("");

	if Id.is_empty() {

		dev_log!("grpc", "[StatusBar] dispose skip: missing id");

		return;
	}

	let _ = Service
		.ApplicationHandle()
		.emit("sky://statusbar/dispose-entry", json!({ "id": Id }));

	dev_log!("grpc", "[StatusBar] dispose id={}", Id);
}
