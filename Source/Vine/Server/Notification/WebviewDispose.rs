#![allow(non_snake_case)]
//! Cocoon → Mountain `webview.dispose` notification.
//! Emitted by `Cocoon/.../Services/Window/WebviewPanel.ts:155` when the
//! extension disposes a webview panel or the user closes the tab. Sky's
//! webview shim listens on `sky://webview/dispose` and tears down the
//! DOM container + unregisters the handle lookup.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WebviewDispose(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://webview/dispose", Parameter);

	dev_log!(
		"grpc",
		"[Webview] dispose handle={}",
		Parameter.get("handle").and_then(Value::as_str).unwrap_or("?")
	);
}
