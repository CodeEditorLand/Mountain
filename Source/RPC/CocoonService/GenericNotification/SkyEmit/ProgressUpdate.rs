#![allow(unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Message = Params.get("message").and_then(|V| V.as_str()).map(|S| S.to_string());

	let Increment = Params.get("increment").and_then(|V| V.as_f64());

	let _ = Env.ApplicationHandle.emit(
		"sky://progress/update",
		json!({ "id": Id, "message": Message, "increment": Increment }),
	);
}
