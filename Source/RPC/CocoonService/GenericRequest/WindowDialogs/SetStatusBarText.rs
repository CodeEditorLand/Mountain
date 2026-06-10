use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;
use ::Vine::Generated::GenericResponse;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));

	super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true }))
}
