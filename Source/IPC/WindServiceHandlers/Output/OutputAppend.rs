//! Append text to an output channel. Emits the
//! `sky://output/append` Tauri event with `{channel, text}`.
//! Sky's output-channel panel mounts the text into its scroll
//! buffer.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgString;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = ArgString(&Arguments, 0);

	let Text = ArgString(&Arguments, 1);

	let _ = ApplicationHandle.emit(SkyEvent::Fn.AsStr(), json!({ "channel": ChannelName, "text": Text }));

	Ok(Value::Null)
}
