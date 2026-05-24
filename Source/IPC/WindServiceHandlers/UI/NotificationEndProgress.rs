//! Wire method: `notification:endProgress`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgString;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = ArgString(&Arguments, 0);

	let _ = ApplicationHandle.emit(SkyEvent::NotificationProgressEnd.AsStr(), json!({ "id": Id }));

	Ok(Value::Null)
}
