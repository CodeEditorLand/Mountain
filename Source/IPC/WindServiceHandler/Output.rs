#![allow(non_snake_case)]

//! Output Channel domain handlers for Wind IPC.

use serde_json::{Value, json};
use tauri::AppHandle;

use crate::dev_log;

/// Create a named output channel. Returns the channel name as its handle.
pub async fn handle_output_create(_AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	let ChannelName = Args.first().and_then(|V| V.as_str()).unwrap_or("Output").to_string();
	dev_log!("ipc", "output:create channel='{}'", ChannelName);
	Ok(json!({ "channelName": ChannelName }))
}

/// Append text to an output channel.
pub async fn handle_output_append(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = Args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = AppHandle.emit("sky://output/append", json!({ "channel": ChannelName, "text": Text }));
	Ok(Value::Null)
}

/// Append a line to an output channel (text + newline).
pub async fn handle_output_append_line(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = Args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Line = format!("{}\n", Text);

	let _ = AppHandle.emit("sky://output/append", json!({ "channel": ChannelName, "text": Line }));
	Ok(Value::Null)
}

/// Clear an output channel.
pub async fn handle_output_clear(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = AppHandle.emit("sky://output/clear", json!({ "channel": ChannelName }));
	Ok(Value::Null)
}

/// Show an output channel panel.
pub async fn handle_output_show(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = AppHandle.emit("sky://output/show", json!({ "channel": ChannelName }));
	Ok(Value::Null)
}
