#![allow(non_snake_case)]
//! Cocoon → Mountain `progress.report` notification.
//! Fires on every `Progress.report({ message, increment })` callback
//! within a `vscode.window.withProgress(...)` run. Forwarded onto
//! `sky://notification/progress-update`.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ProgressReport(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Handle = Parameter.get("handle").and_then(Value::as_str).unwrap_or("");
	let Message = Parameter.get("message").and_then(Value::as_str).unwrap_or("");
	let Increment = Parameter.get("increment").and_then(Value::as_f64).unwrap_or(0.0);
	if let Err(Error) = Service.ApplicationHandle().emit(
		"sky://notification/progress-update",
		json!({
			"id": Handle,
			"message": Message,
			"increment": Increment,
		}),
	) {
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] sky://notification/progress-update emit failed: {}",
			Error
		);
	}
}
