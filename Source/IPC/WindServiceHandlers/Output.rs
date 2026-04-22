#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Output channel handlers — create, append, clear, show.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::AppHandle;

use crate::dev_log;

/// Create a named output channel. Returns the channel name as its handle.
pub async fn handle_output_create(_app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("Output").to_string();
	dev_log!("ipc", "output:create channel='{}'", ChannelName);
	// Sky/frontend creates the channel panel on the `sky://output/create` event
	Ok(json!({ "channelName": ChannelName }))
}

/// Append text to an output channel.
pub async fn handle_output_append(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(SkyEvent::OutputAppend.AsStr(), json!({ "channel": ChannelName, "text": Text }));
	Ok(Value::Null)
}

/// Append a line to an output channel (text + newline).
pub async fn handle_output_append_line(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Line = format!("{}\n", Text);

	let _ = app_handle.emit(SkyEvent::OutputAppend.AsStr(), json!({ "channel": ChannelName, "text": Line }));
	Ok(Value::Null)
}

/// Clear an output channel.
pub async fn handle_output_clear(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit(SkyEvent::OutputClear.AsStr(), json!({ "channel": ChannelName }));
	Ok(Value::Null)
}

/// Show an output channel panel.
pub async fn handle_output_show(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit(SkyEvent::OutputShow.AsStr(), json!({ "channel": ChannelName }));
	Ok(Value::Null)
}
