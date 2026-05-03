#![allow(non_snake_case)]

//! Show an output channel panel. Emits `sky://output/show`; Sky
//! brings the panel to the front and focuses it.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

pub async fn OutputShow(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = ApplicationHandle.emit(SkyEvent::OutputShow.AsStr(), json!({ "channel": ChannelName }));
	Ok(Value::Null)
}
