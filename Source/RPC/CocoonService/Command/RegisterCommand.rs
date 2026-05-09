#![allow(non_snake_case)]

//! Wire a Cocoon-contributed command into Mountain's `CommandExecutor` as
//! a Proxied handler that forwards back to the sidecar.

use CommonLibrary::Command::CommandExecutor::CommandExecutor;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterCommandRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterCommandRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering command '{}' from extension '{}'",
		Request.command_id,
		Request.extension_id
	);

	if let Err(Error) = Service
		.environment
		.RegisterCommand(Request.extension_id.clone(), Request.command_id.clone())
		.await
	{
		dev_log!(
			"cocoon",
			"warn: [CocoonService] Failed to register command '{}': {:?}",
			Request.command_id,
			Error
		);
	} else {
		dev_log!(
			"cocoon",
			"[CocoonService] Command registered: id={}, title={:?}",
			Request.command_id,
			Request.title
		);
	}

	Ok(Response::new(Empty {}))
}
