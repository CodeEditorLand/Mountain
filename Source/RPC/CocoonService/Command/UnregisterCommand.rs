//! Remove a previously-registered Cocoon command from the executor.

use CommonLibrary::Command::CommandExecutor::CommandExecutor;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, UnregisterCommandRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:UnregisterCommandRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Unregistering command '{}'", Request.command_id);

	if let Err(Error) = Service
		.environment
		.UnregisterCommand(String::new(), Request.command_id.clone())
		.await
	{
		dev_log!(
			"cocoon",
			"warn: [CocoonService] Failed to unregister command '{}': {:?}",
			Request.command_id,
			Error
		);
	} else {
		dev_log!("cocoon", "[CocoonService] Command removed: {}", Request.command_id);
	}

	Ok(Response::new(Empty {}))
}
