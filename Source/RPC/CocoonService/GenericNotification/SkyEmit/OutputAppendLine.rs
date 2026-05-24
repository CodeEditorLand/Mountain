use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::Struct;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Line = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit(
		"sky://output/append",
		json!({ "channel": Channel, "text": format!("{}\n", Line) }),
	);
}
