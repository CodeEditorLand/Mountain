//! Wire method: `progress:begin`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::{arg_bool, arg_string, arg_string_or};

fn NewProgressId() -> String {
	format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	)
}

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Location = arg_string_or(&Arguments, 0, "notification");

	let Title = arg_string(&Arguments, 1);

	let Cancellable = arg_bool(&Arguments, 2);

	let Id = NewProgressId();

	let _ = ApplicationHandle.emit(
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
