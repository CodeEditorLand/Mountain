use serde_json::{Value, json};

use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {

	let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));
}
