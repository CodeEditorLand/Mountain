#![allow(non_snake_case)]
//! Command domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_command, execute_contributed_command,
//! unregister_command.

use CommonLibrary::Command::CommandExecutor::CommandExecutor;
use serde_json::json;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::dev_log;
use crate::Vine::Generated::{
	Empty, ExecuteCommandRequest, ExecuteCommandResponse,
	RegisterCommandRequest, UnregisterCommandRequest,
};

pub async fn RegisterCommand(
	Service:&CocoonServiceImpl,
	req:RegisterCommandRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon",
		"[CocoonService] Registering command '{}' from extension '{}'",
		req.command_id, req.extension_id
	);

	// Wire to CommandExecutor::RegisterCommand which stores a Proxied handler
	// pointing back to the Cocoon sidecar.
	if let Err(Error) = Service
		.environment
		.RegisterCommand(req.extension_id.clone(), req.command_id.clone())
		.await
	{
		dev_log!("cocoon", "warn: [CocoonService] Failed to register command '{}': {:?}", req.command_id, Error);
	} else {
		dev_log!("cocoon",
			"[CocoonService] Command registered: id={}, title={:?}",
			req.command_id, req.title
		);
	}

	Ok(Response::new(Empty {}))
}

pub async fn ExecuteContributedCommand(
	Service:&CocoonServiceImpl,
	req:ExecuteCommandRequest,
) -> Result<Response<ExecuteCommandResponse>, Status> {
	dev_log!("cocoon",
		"[CocoonService] Executing command '{}' with {} arguments",
		req.command_id,
		req.arguments.len()
	);

	// Look up command handler and execute with parameters
	for (i, arg) in req.arguments.iter().enumerate() {
		dev_log!("cocoon", "[CocoonService] Argument {}: {:?}", i, arg);
	}

	// Convert the first Argument oneof value to a serde_json::Value
	let Arg:serde_json::Value = req
		.arguments
		.first()
		.and_then(|A| A.value.as_ref())
		.map(|V| {
			match V {
				crate::Vine::Generated::argument::Value::StringValue(S) => json!(S),
				crate::Vine::Generated::argument::Value::IntValue(I) => json!(I),
				crate::Vine::Generated::argument::Value::BoolValue(B) => json!(B),
				crate::Vine::Generated::argument::Value::BytesValue(Bytes) => {
					serde_json::from_slice(Bytes).unwrap_or(serde_json::Value::Null)
				},
			}
		})
		.unwrap_or(serde_json::Value::Null);

	match Service.environment.ExecuteCommand(req.command_id, Arg).await {
		Ok(Value) => {
			let Bytes = serde_json::to_vec(&Value).unwrap_or_default();
			Ok(Response::new(ExecuteCommandResponse {
				result:Some(crate::Vine::Generated::execute_command_response::Result::Value(Bytes)),
			}))
		},
		Err(Error) => {
			let Bytes = serde_json::to_vec(&Error.to_string()).unwrap_or_default();
			Ok(Response::new(ExecuteCommandResponse {
				result:Some(crate::Vine::Generated::execute_command_response::Result::Error(
					crate::Vine::Generated::RpcError { code:-32000, message:Error.to_string(), data:Bytes },
				)),
			}))
		},
	}
}

pub async fn UnregisterCommand(
	Service:&CocoonServiceImpl,
	req:UnregisterCommandRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Unregistering command '{}'", req.command_id);

	// Wire to CommandExecutor::UnregisterCommand
	if let Err(Error) = Service.environment.UnregisterCommand(String::new(), req.command_id.clone()).await {
		dev_log!("cocoon", "warn: [CocoonService] Failed to unregister command '{}': {:?}", req.command_id, Error);
	} else {
		dev_log!("cocoon", "[CocoonService] Command removed: {}", req.command_id);
	}

	Ok(Response::new(Empty {}))
}
