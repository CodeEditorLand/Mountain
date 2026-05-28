use serde_json::Value;
use tonic::Response;
use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use crate::{::Vine::Generated::GenericResponse, Environment::MountainEnvironment::MountainEnvironment};

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let CommandId = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Arg = Params.get("arg").cloned().unwrap_or(Value::Null);

	match Env.ExecuteCommand(CommandId, Arg).await {
		Ok(V) => super::super::FileSystem::OkResponse(RequestId, &V),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
