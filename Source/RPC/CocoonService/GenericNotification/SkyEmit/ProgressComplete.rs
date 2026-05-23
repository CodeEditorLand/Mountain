#![allow(unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit("sky://progress/complete", json!({ "id": Id }));
}
