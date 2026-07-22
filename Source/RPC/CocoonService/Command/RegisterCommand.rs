//! Wire a Cocoon-contributed command into Mountain's `CommandExecutor` as
//! a Proxied handler that forwards back to the sidecar.
use CommonLibrary::Command::CommandExecutor::CommandExecutor;
use tonic::{Response, Status};
use ::Vine::Generated::{Empty, RegisterCommandRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterCommandRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering command '{}' from extension '{}'",
		Request.command_id,
		Request.extension_id
	);

	match Service
		.environment
		.RegisterCommand(Request.extension_id.clone(), Request.command_id.clone())
		.await
	{
		Ok(()) => {
			dev_log!(
				"cocoon",
				"[CocoonService] Command registered: id={}, title={:?}",
				Request.command_id,
				Request.title
			)
		},

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonService] Failed to register command '{}': {:?}",
				Request.command_id,
				Error
			)
		},
	}

	Ok(Response::new(Empty {}))
}
