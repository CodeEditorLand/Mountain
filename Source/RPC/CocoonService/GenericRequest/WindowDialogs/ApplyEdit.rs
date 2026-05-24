use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Uri = Params
		.Get("uri")
		.and_then(|V| V.get("value").or(Some(V)))
		.and_then(|V| V.as_str())
		.unwrap_or("")
		.to_string();

	let Edits = Params.get("edits").cloned().unwrap_or(json!([]));

	let _ = Env
		.ApplicationHandle
		.emit("sky://editor/applyEdits", json!({ "uri": Uri, "edits": Edits }));

	super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true }))
}
