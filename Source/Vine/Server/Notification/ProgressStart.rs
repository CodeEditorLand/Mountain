#![allow(non_snake_case)]
//! Cocoon → Mountain `progress.start` notification.
//! Fires at the top of every `vscode.window.withProgress(...)` call.
//! Normalises onto `sky://notification/progress-begin` so Sky's progress
//! indicator renders identically whether an extension or a Mountain
//! handler triggered the progress.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ProgressStart(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Handle = Parameter.get("handle").and_then(Value::as_str).unwrap_or("");
	let Title = Parameter.get("title").and_then(Value::as_str).unwrap_or("");
	let Cancellable = Parameter.get("cancellable").and_then(Value::as_bool).unwrap_or(false);
	if let Err(Error) = Service.ApplicationHandle().emit(
		"sky://notification/progress-begin",
		json!({
			"id": Handle,
			"title": Title,
			"cancellable": Cancellable,
		}),
	) {
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] sky://notification/progress-begin emit failed: {}",
			Error
		);
	}
}
