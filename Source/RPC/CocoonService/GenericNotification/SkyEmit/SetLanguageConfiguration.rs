
use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Language = Params.get("language").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://language/configure", json!({ "language": Language }));
}
