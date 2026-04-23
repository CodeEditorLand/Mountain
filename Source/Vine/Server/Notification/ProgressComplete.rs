#![allow(non_snake_case)]
//! Cocoon → Mountain `progress.complete` notification.
//! Fires once `vscode.window.withProgress(...)` settles - either the task
//! finishes or Cocoon cancels it. Canonical counterpart to Mountain's
//! already-handled `progress.end`; this atom re-routes onto the
//! `sky://progress/end` channel so the renderer's progress indicator
//! tears down regardless of which wire name Cocoon picked.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ProgressComplete(Service:&MountainVinegRPCService, Parameter:&Value) {
	let _ = Service.ApplicationHandle().emit("sky://progress/end", Parameter);
	dev_log!(
		"grpc",
		"[Progress] complete id={}",
		Parameter.get("id").and_then(Value::as_str).unwrap_or("?")
	);
}
