use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	Env.ApplicationState.Feature.OutputChannels.Remove(&Channel);

	let _ = Env
		.ApplicationHandle
		.emit_to("main", "sky://output/dispose", json!({ "channel": Channel }));
}
