//! Show an output channel panel. Emits `sky://output/show`; Sky
//! brings the panel to the front and focuses it.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = arg_string(&Arguments, 0);

	let _ = ApplicationHandle.emit(SkyEvent::OutputShow.AsStr(), json!({ "channel": ChannelName }));

	Ok(Value::Null)
}
