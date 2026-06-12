use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	if let Some(Entry) = Env
		.ApplicationState
		.Feature
		.OutputChannels
		.OutputChannels
		.lock()
		.get_mut(&Channel)
	{
		Entry.SetVisibility(true);
	}

	let _ = Env
		.ApplicationHandle
		.emit_to("main", "sky://output/show", json!({ "channel": Channel }));
}
