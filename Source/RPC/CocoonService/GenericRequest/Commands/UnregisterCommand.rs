use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use crate::{Environment::MountainEnvironment::MountainEnvironment};

use ::Vine::Generated::GenericResponse;

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	match Env.UnregisterCommand(ExtensionId, CommandId).await {
		Ok(()) => super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true })),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
