//! Clear an output channel. Emits `sky://output/clear` with the
//! channel name; Sky drops the channel's scroll buffer.

use CommonLibrary::IPC::SkyEvent::SkyEvent;

use serde_json::{Value, json};

use tauri::{AppHandle, Emitter};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {

	let ChannelName = arg_string(&Arguments, 0);

	let _ = ApplicationHandle.emit(SkyEvent::OutputClear.AsStr(), json!({ "channel": ChannelName }));

	Ok(Value::Null)
}
