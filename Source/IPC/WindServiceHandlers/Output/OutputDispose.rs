//! Dispose of an output channel. Emits `sky://output/dispose` with
//! `{channel}` so Sky can remove the channel from its drop-down and
//! free the buffer.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = arg_string(&Arguments, 0);

	let _ = ApplicationHandle.emit(
		SkyEvent::OutputDispose.AsStr(),
		json!({ "channel": ChannelName }),
	);

	Ok(Value::Null)
}
