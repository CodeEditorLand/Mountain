//! Look up a contributed command and execute it. Marshals the first
//! protobuf `argument` oneof into `serde_json::Value` for the executor.

use CommonLibrary::Command::CommandExecutor::CommandExecutor;
use serde_json::json;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ExecuteCommandRequest, ExecuteCommandResponse, RpcError, argument, execute_command_response},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ExecuteCommandRequest,
) -> Result<Response<ExecuteCommandResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Executing command '{}' with {} arguments",
		Request.command_id,
		Request.arguments.len()
	);

	for (Index, Argument) in Request.arguments.iter().enumerate() {
		dev_log!("cocoon", "[CocoonService] Argument {}: {:?}", Index, Argument);
	}

	let Arg:serde_json::Value = Request
		.arguments
		.first()
		.and_then(|A| A.value.as_ref())
		.map(|V| {
			match V {
				argument::Value::StringValue(S) => json!(S),
				argument::Value::IntValue(I) => json!(I),
				argument::Value::BoolValue(B) => json!(B),
				argument::Value::BytesValue(Bytes) => serde_json::from_slice(Bytes).unwrap_or(serde_json::Value::Null),
			}
		})
		.unwrap_or(serde_json::Value::Null);

	match Service.environment.ExecuteCommand(Request.command_id, Arg).await {
		Ok(Value) => {
			let Bytes = serde_json::to_vec(&Value).unwrap_or_default();

			Ok(Response::new(ExecuteCommandResponse {
				result:Some(execute_command_response::Result::Value(Bytes)),
			}))
		},

		Err(Error) => {
			let Bytes = serde_json::to_vec(&Error.to_string()).unwrap_or_default();

			Ok(Response::new(ExecuteCommandResponse {
				result:Some(execute_command_response::Result::Error(RpcError {
					code:-32000,
					message:Error.to_string(),
					data:Bytes,
				})),
			}))
		},
	}
}
