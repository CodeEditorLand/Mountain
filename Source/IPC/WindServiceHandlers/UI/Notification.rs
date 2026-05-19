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
	use std::sync::atomic::{AtomicU64, Ordering};

	static SEQ:AtomicU64 = AtomicU64::new(1);

	format!("{}-{}", Prefix, SEQ.fetch_add(1, Ordering::Relaxed))
}

pub async fn NotificationShow(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Message = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Severity = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("info").to_string();

	let Actions = Arguments.get(2).cloned().unwrap_or(json!([]));

	let Id = NewId("notification");

	let _ = ApplicationHandle.emit(
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

pub async fn NotificationShowProgress(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Title = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Cancellable = Arguments.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = NewId("progress");

	let _ = ApplicationHandle.emit(
		SkyEvent::NotificationProgressBegin.AsStr(),
		json!({
			"id": Id,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

pub async fn NotificationUpdateProgress(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Increment = Arguments.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);

	let Message = Arguments.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = ApplicationHandle.emit(
		SkyEvent::NotificationProgressUpdate.AsStr(),
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

pub async fn NotificationEndProgress(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = ApplicationHandle.emit(SkyEvent::NotificationProgressEnd.AsStr(), json!({ "id": Id }));

	Ok(Value::Null)
}
