use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let IncludeUntitled = Params.get("includeUntitled").and_then(|V| V.as_bool()).unwrap_or(false);

	let _ = Env
		.ApplicationHandle
		.emit("sky://editor/saveAll", json!({ "includeUntitled": IncludeUntitled }));

	super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true }))
}
