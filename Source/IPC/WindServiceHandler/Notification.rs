#![allow(non_snake_case)]

//! Notification domain handlers for Wind IPC.

use serde_json::{Value, json};
use tauri::AppHandle;

/// Show a notification message - emits sky://notification/show for Sky to
/// render.
pub async fn handle_notification_show(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Message = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Severity = Args.get(1).and_then(|V| V.as_str()).unwrap_or("info").to_string();
	let Actions = Args.get(2).cloned().unwrap_or(json!([]));

	let Id = format!(
		"notification-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = AppHandle.emit(
		"sky://notification/show",
		json!({
			"id": Id,
			"message": Message,
			"severity": Severity,
			"actions": Actions,
		}),
	);

	Ok(json!(Id))
}

/// Begin a progress notification - emits sky://notification/progress-begin.
pub async fn handle_notification_show_progress(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Title = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = Args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = AppHandle.emit(
		"sky://notification/progress-begin",
		json!({
			"id": Id,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Update an in-progress notification progress bar.
pub async fn handle_notification_update_progress(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = Args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = Args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = AppHandle.emit(
		"sky://notification/progress-update",
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress notification.
pub async fn handle_notification_end_progress(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = AppHandle.emit("sky://notification/progress-end", json!({ "id": Id }));

	Ok(Value::Null)
}
