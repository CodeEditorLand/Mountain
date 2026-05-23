#![allow(unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Arg = Params
		.get("arguments")
		.and_then(|A| A.as_array())
		.and_then(|A| A.first())
		.cloned()
		.unwrap_or(Value::Null);

	match Env.ExecuteCommand(CommandId, Arg).await {
		Ok(V) => super::super::FileSystem::OkResponse(RequestId, &json!({ "result": V })),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
