//! Create a named output channel. Returns the channel name as
//! its handle. The Sky/frontend listens for `sky://output/create`
//! and instantiates the channel panel; we just acknowledge.

use serde_json::{Value, json};
use tauri::AppHandle;

use crate::{IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgStringOr, dev_log};

pub async fn Fn(_ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = ArgStringOr(&Arguments, 0, "Output");

	dev_log!("ipc", "output:create channel='{}'", ChannelName);

	Ok(json!({ "channelName": ChannelName }))
}
