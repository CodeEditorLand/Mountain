use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Name = Params.get("name").and_then(|V| V.as_str()).unwrap_or("").to_string();

	if !Id.is_empty() {
		let mut Channels = Env.ApplicationState.Feature.OutputChannels.OutputChannels.lock();

		let Entry = Channels.entry(Id.clone()).or_default();

		if Entry.Name.is_empty() {
			Entry.Name = Name.clone();
		}
	}

	let _ = Env
		.ApplicationHandle
		.emit_to("main", "sky://output/create", json!({ "id": Id, "name": Name }));
}
