use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;

use crate::{Environment::MountainEnvironment::MountainEnvironment};

use ::Vine::Generated::GenericResponse;

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Url = Params
		.as_str()
		.or_else(|| Params.get("url").and_then(|V| V.as_str()))
		.unwrap_or("")
		.to_string();

	let _ = Env.ApplicationHandle.emit("sky://native/openExternal", json!({ "url": Url }));

	super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true }))
}
