//! Wire method: `progress:end`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = arg_string(&Arguments, 0);

	let _ = ApplicationHandle.emit(SkyEvent::ProgressEnd.AsStr(), json!({ "id": Id }));

	Ok(Value::Null)
}
