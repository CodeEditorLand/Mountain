#![allow(non_snake_case)]
//! Cocoon → Mountain `progress.update` notification.
//! Cocoon's `Services/Window/Progress.ts:56` emits this on every
//! `Progress.report({ message, increment })` callback during an
//! extension's `vscode.window.withProgress(...)` invocation. Stock
//! Mountain already handles `progress.report` with identical payload
//! semantics; this atom funnels into the same `sky://` channel so the
//! notification-surface name mismatch (update vs report) doesn't leak
//! into the renderer contract.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ProgressUpdate(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service
		.ApplicationHandle()
		.emit("sky://notification/progress-update", Parameter);

	dev_log!(
		"grpc",
		"[Progress] update id={}",
		Parameter.get("id").and_then(Value::as_str).unwrap_or("?")
	);
}
