#![allow(non_snake_case)]

//! Progress domain handlers for Wind IPC.

use serde_json::{Value, json};
use tauri::AppHandle;

/// Begin a window-level or status-bar progress indicator.
pub async fn handle_progress_begin(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Location = Args.first().and_then(|V| V.as_str()).unwrap_or("notification").to_string();
	let Title = Args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = Args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = AppHandle.emit(
		"sky://progress/begin",
		json!({
			"id": Id,
			"location": Location,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Report incremental progress on an active indicator.
pub async fn handle_progress_report(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = Args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = Args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = AppHandle.emit(
		"sky://progress/report",
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress indicator.
pub async fn handle_progress_end(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = AppHandle.emit("sky://progress/end", json!({ "id": Id }));

	Ok(Value::Null)
}
