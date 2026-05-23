#![allow(unused_variables, dead_code, unused_imports)]

//! Generic-request command handlers for `process_mountain_request`.
//! Handles `commands.execute`, `executeCommand`, and `unregisterCommand`
//! using Cocoon's `MountainGRPCClient` method name conventions.

use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};
use super::FileSystem::{ErrResponse, OkResponse};

pub async fn HandleCommandsExecute(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let CommandId = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Arg = Params.get("arg").cloned().unwrap_or(Value::Null);

	match Env.ExecuteCommand(CommandId, Arg).await {
		Ok(V) => OkResponse(RequestId, &V),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub async fn HandleExecuteCommand(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Arg = Params
		.get("arguments")
		.and_then(|A| A.as_array())
		.and_then(|A| A.first())
		.cloned()
		.unwrap_or(Value::Null);

	match Env.ExecuteCommand(CommandId, Arg).await {
		Ok(V) => OkResponse(RequestId, &json!({ "result": V })),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub async fn HandleUnregisterCommand(
	RequestId:u64,

	Params:Value,

	Env:&MountainEnvironment,
) -> Response<GenericResponse> {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	match Env.UnregisterCommand(ExtensionId, CommandId).await {
		Ok(()) => OkResponse(RequestId, &json!({ "success": true })),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
