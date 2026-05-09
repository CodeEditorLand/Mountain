#![allow(non_snake_case)]
//! Cocoon → Mountain `progress.complete` notification.
//! Fires once `vscode.window.withProgress(...)` settles - either the task
//! finishes or Cocoon cancels it. Emits onto Sky's canonical
//! `sky://progress/complete` channel so the renderer's progress indicator
//! tears down. (Earlier code emitted `sky://progress/end` which Sky never
//! listened on - the indicator stayed pinned forever.)

use serde_json::Value;

use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ProgressComplete(Service:&MountainVinegRPCService, Parameter:&Value) {

	let _ = Service.ApplicationHandle().emit("sky://progress/complete", Parameter);

	dev_log!(
		"grpc",

		"[Progress] complete id={}",

		Parameter.get("id").and_then(Value::as_str).unwrap_or("?")
	);
}
