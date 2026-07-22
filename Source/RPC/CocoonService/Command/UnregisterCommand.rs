//! Remove a previously-registered Cocoon command from the executor.
use CommonLibrary::Command::CommandExecutor::CommandExecutor;
use tonic::{Response, Status};
use ::Vine::Generated::{Empty, UnregisterCommandRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:UnregisterCommandRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Unregistering command '{}'", Request.command_id);

	match Service
		.environment
		.UnregisterCommand(String::new(), Request.command_id.clone())
		.await
	{
		Ok(()) => dev_log!("cocoon", "[CocoonService] Command removed: {}", Request.command_id),

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonService] Failed to unregister command '{}': {:?}",
				Request.command_id,
				Error
			)
		},
	}

	Ok(Response::new(Empty {}))
}
