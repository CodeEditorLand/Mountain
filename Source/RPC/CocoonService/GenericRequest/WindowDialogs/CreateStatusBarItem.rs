use serde_json::{Value, json};
use tauri::Emitter;
use tonic::Response;

use crate::{Environment::MountainEnvironment::MountainEnvironment};

use ::Vine::Generated::GenericResponse;

pub fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Tooltip = Params.get("tooltip").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit(
		"sky://statusbar/set-entry",
		json!({ "id": Id, "text": Text, "tooltip": Tooltip }),
	);

	super::super::FileSystem::OkResponse(RequestId, &json!({ "itemId": Id }))
}
