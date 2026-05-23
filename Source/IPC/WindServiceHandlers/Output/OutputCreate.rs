#![allow(non_snake_case, unused_variables)]

//! Create a named output channel. Returns the channel name as
//! its handle. The Sky/frontend listens for `sky://output/create`
//! and instantiates the channel panel; we just acknowledge.

use serde_json::{Value, json};
use tauri::AppHandle;

use crate::dev_log;

pub async fn Fn(_ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = Arguments.first().and_then(|V| V.as_str()).unwrap_or("Output").to_string();

	dev_log!("ipc", "output:create channel='{}'", ChannelName);

	Ok(json!({ "channelName": ChannelName }))
}
