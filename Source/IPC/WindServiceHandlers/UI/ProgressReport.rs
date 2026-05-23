//! Wire method: `progress:report`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Increment = Arguments.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);

	let Message = Arguments.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = ApplicationHandle.emit(
		SkyEvent::ProgressReport.AsStr(),
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}
