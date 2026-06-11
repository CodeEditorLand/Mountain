//! Create a named output channel. Returns `{ channelName }` as the handle
//! and emits `sky://output/create` so Sky's `InstallEditorAndOutput.ts`
//! can add the channel to its in-memory `OutputChannels` map and make
//! it selectable in the Output panel drop-down.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager};

use crate::{IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string_or, dev_log};

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = arg_string_or(&Arguments, 0, "Output");

	dev_log!("output", "output:create channel='{}'", ChannelName);

	// Notify Sky to register the channel in the Output panel.
	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let _ = Window.emit(
			"sky://output/create",
			json!({ "id": ChannelName, "name": ChannelName }),
		);
	}

	Ok(json!({ "channelName": ChannelName }))
}
