//! Cocoon → Mountain `progress.end` notification.
//! Fires once per `vscode.window.withProgress(...)` call when the task
//! settles. Forwarded onto `sky://notification/progress-end` so Sky's
//! progress indicator tears down.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn Fn(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Handle = Parameter.get("handle").and_then(Value::as_str).unwrap_or("");

	if let Err(Error) = Service
		.ApplicationHandle()
		.emit("sky://notification/progress-end", json!({ "id": Handle }))
	{
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] sky://notification/progress-end emit failed: {}",
			Error
		);
	}
}
