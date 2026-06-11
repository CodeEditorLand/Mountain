//! Replace all content of an output channel. Emits `sky://output/replace`
//! with `{channel, text}` so Sky's `InstallEditorAndOutput.ts` can
//! clear the existing buffer and set the new content in one step.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let ChannelName = arg_string(&Arguments, 0);

	let Text = arg_string(&Arguments, 1);

	let _ = ApplicationHandle.emit(
		SkyEvent::OutputReplace.AsStr(),
		json!({ "channel": ChannelName, "text": Text }),
	);

	Ok(Value::Null)
}
