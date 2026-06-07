use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;
use ::Vine::Generated::GenericResponse;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0);

	let Html = Params.get("html").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://webview/set-html", json!({ "handle": Handle, "html": Html }));

	super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true }))
}
