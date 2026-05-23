//! Wire method: `notification:endProgress`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Id = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = ApplicationHandle.emit(SkyEvent::NotificationProgressEnd.AsStr(), json!({ "id": Id }));

	Ok(Value::Null)
}
