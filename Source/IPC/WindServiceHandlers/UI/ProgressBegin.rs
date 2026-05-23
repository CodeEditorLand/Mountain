#![allow(unused_variables)]

//! Wire method: `progress:begin`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
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

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Location = Arguments.first().and_then(|V| V.as_str()).unwrap_or("notification").to_string();

	let Title = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Cancellable = Arguments.get(2).and_then(|V| V.as_bool()).unwrap_or(false);

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
