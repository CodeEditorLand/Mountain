#![allow(non_snake_case, unused_variables)]
//! Progress indicator handlers (`progress:begin/report/end`). Distinct
//! from the notification-scoped progress surface in
//! `UI::Notification` - these drive window-level / status-bar progress
//! via `SkyEvent::Progress*`.

use serde_json::{Value, json};
use tauri::AppHandle;
use CommonLibrary::IPC::SkyEvent::SkyEvent;

fn NewProgressId() -> String {
	format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	)
}

pub async fn ProgressBegin(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Location = args.first().and_then(|V| V.as_str()).unwrap_or("notification").to_string();
	let Title = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = NewProgressId();
	let _ = app_handle.emit(
		SkyEvent::ProgressBegin.AsStr(),
		json!({
			"id": Id,
			"location": Location,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

pub async fn ProgressReport(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		SkyEvent::ProgressReport.AsStr(),
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

pub async fn ProgressEnd(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit(SkyEvent::ProgressEnd.AsStr(), json!({ "id": Id }));
	Ok(Value::Null)
}
