//! Append a line (text + `\n`) to an output channel. Twin of
//! `OutputAppend` with the newline pre-applied so the renderer
//! doesn't need its own line-mode toggle.

use CommonLibrary::IPC::SkyEvent::SkyEvent;

use serde_json::{Value, json};

use tauri::{AppHandle, Emitter};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {

	let ChannelName = arg_string(&Arguments, 0);

	let Text = arg_string(&Arguments, 1);

	let Line = format!("{}\n", Text);

	let _ = ApplicationHandle.emit(SkyEvent::OutputAppend.AsStr(), json!({ "channel": ChannelName, "text": Line }));

	Ok(Value::Null)
}
