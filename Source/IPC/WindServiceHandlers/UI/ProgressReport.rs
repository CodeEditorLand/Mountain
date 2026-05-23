//! Wire method: `progress:report`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::{arg_f64, arg_string};

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = arg_string(&Arguments, 0);

	let Increment = arg_f64(&Arguments, 1);

	let Message = arg_string(&Arguments, 2);

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
