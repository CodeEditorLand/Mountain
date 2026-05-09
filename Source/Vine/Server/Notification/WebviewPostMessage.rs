#![allow(non_snake_case)]
//! Cocoon → Mountain `webview.postMessage` notification.
//! Emitted by `Cocoon/.../Services/Window/WebviewPanel.ts:125` when an
//! extension calls `WebviewPanel.webview.postMessage(...)`. Stock VS
//! Code delivers this as a DOM `message` event inside the webview
//! `iframe`; in Land we emit on `sky://webview/postMessage` and let the
//! Sky bridge relay into the specific webview DOM container keyed on
//! `{ handle, message }`.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WebviewPostMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://webview/postMessage", Parameter);

	dev_log!(
		"grpc",
		"[Webview] postMessage handle={}",
		Parameter.get("handle").and_then(Value::as_str).unwrap_or("?")
	);
}
