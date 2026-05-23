
//! Append a line (text + `\n`) to an output channel. Twin of
//! `OutputAppend` with the newline pre-applied so the renderer
//! doesn't need its own line-mode toggle.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Line = format!("{}\n", Text);

	let _ = ApplicationHandle.emit(SkyEvent::OutputAppend.AsStr(), json!({ "channel": ChannelName, "text": Line }));

	Ok(Value::Null)
}
