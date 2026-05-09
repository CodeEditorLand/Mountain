#![allow(non_snake_case)]

//! Append text to an output channel. Emits the
//! `sky://output/append` Tauri event with `{channel, text}`.
//! Sky's output-channel panel mounts the text into its scroll
//! buffer.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

pub async fn OutputAppend(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = ApplicationHandle.emit(SkyEvent::OutputAppend.AsStr(), json!({ "channel": ChannelName, "text": Text }));

	Ok(Value::Null)
}
