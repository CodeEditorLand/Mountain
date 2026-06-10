use serde_json::{Value, json};

use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {

	let Url = Params.get("url").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit("sky://native/openExternal", json!({ "url": Url }));
}
