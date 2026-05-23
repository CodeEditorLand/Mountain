#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ViewType = Params.get("viewType").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Title = Params.get("title").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Handle = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map(|D| D.as_millis() as u64)
		.unwrap_or(0);

	let _ = Env.ApplicationHandle.emit(
		"sky://webview/create",
		json!({
			"handle": Handle,
			"viewType": ViewType,
			"title": Title,
			"viewColumn": Params.get("viewColumn"),
			"preserveFocus": Params.get("preserveFocus").and_then(|V| V.as_bool()).unwrap_or(false),
		}),
	);

	super::super::FileSystem::OkResponse(RequestId, &json!({ "handle": Handle }))
}
