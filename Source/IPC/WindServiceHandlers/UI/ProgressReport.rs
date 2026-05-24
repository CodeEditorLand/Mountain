//! Wire method: `progress:report`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::{ArgF64, ArgString};

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = ArgString(&Arguments, 0);

	let Increment = ArgF64(&Arguments, 1);

	let Message = ArgString(&Arguments, 2);

	let _ = ApplicationHandle.emit(
		SkyEvent::Fn.AsStr(),
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}
