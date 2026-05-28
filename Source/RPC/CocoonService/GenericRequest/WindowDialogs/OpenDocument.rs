use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;

use crate::{Environment::MountainEnvironment::MountainEnvironment};

use ::Vine::Generated::GenericResponse;

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Uri = Params
		.get("uri")
		.and_then(|V| V.get("value").or(Some(V)))
		.and_then(|V| V.as_str())
		.unwrap_or("")
		.to_string();

	let ViewColumn = Params.get("viewColumn").and_then(|V| V.as_i64());

	let _ = Env
		.ApplicationHandle
		.emit("sky://editor/openDocument", json!({ "uri": Uri, "viewColumn": ViewColumn }));

	super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true }))
}
