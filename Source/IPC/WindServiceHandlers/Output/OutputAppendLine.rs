//! Append a line (text + `\n`) to an output channel. Twin of
//! `OutputAppend` with the newline pre-applied so the renderer
//! doesn't need its own line-mode toggle.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgString;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = ArgString(&Arguments, 0);

	let Text = ArgString(&Arguments, 1);

	let Line = format!("{}\n", Text);

	let _ = ApplicationHandle.emit(SkyEvent::Fn.AsStr(), json!({ "channel": ChannelName, "text": Line }));

	Ok(Value::Null)
}
