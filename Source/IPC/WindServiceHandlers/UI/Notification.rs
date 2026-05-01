#![allow(non_snake_case, unused_variables)]
//! Notification toast handlers. Both the plain-message and the
//! progress-bar variants go through here; each emits on an
//! `SkyEvent::Notification*` channel so Sky's toast stack renders
//! without a round-trip back through Mountain.
//!
//! Note: these are the Wind-facing IPC invocations (called from the
//! renderer's `INotificationService`). The Cocoon-side notification
//! path for extensions lives in `Vine::Server::Notification::*`.

use serde_json::{Value, json};
use tauri::AppHandle;
use CommonLibrary::IPC::SkyEvent::SkyEvent;

fn NewId(Prefix:&str) -> String {
	format!(
		"{}-{}",
		Prefix,
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	)
}

pub async fn NotificationShow(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Message = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Severity = args.get(1).and_then(|V| V.as_str()).unwrap_or("info").to_string();
	let Actions = args.get(2).cloned().unwrap_or(json!([]));

	let Id = NewId("notification");
	let _ = app_handle.emit(
		SkyEvent::NotificationShow.AsStr(),
		json!({
			"id": Id,
			"message": Message,
			"severity": Severity,
			"actions": Actions,
		}),
	);

	Ok(json!(Id))
}

pub async fn NotificationShowProgress(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Title = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = NewId("progress");
	let _ = app_handle.emit(
		SkyEvent::NotificationProgressBegin.AsStr(),
		json!({
			"id": Id,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

pub async fn NotificationUpdateProgress(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		SkyEvent::NotificationProgressUpdate.AsStr(),
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

pub async fn NotificationEndProgress(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit(SkyEvent::NotificationProgressEnd.AsStr(), json!({ "id": Id }));
	Ok(Value::Null)
}
