use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Title = Params.get("title").and_then(|V| V.as_str()).map(|S| S.to_string());

	let Location = Params.get("location").cloned();

	let Cancellable = Params.get("cancellable").and_then(|V| V.as_bool()).unwrap_or(false);

	let _ = Env.ApplicationHandle.emit(
		"sky://progress/start",
		json!({ "id": Id, "title": Title, "location": Location, "cancellable": Cancellable }),
	);
}
