//! Forward bytes received from Cocoon to the PTY master writer.

use tonic::{Response, Status};
use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{Empty, TerminalInputRequest};

pub async fn Fn(Service:&CocoonServiceImpl, Request:TerminalInputRequest) -> Result<Response<Empty>, Status> {
	let TerminalIdentifier = Request.terminal_id as u64;

	dev_log!(
		"cocoon",
		"[CocoonService] terminal_input: id={} bytes={}",
		TerminalIdentifier,
		Request.data.len()
	);

	let Text = String::from_utf8_lossy(&Request.data).into_owned();

	match Service.environment.SendTextToTerminal(TerminalIdentifier, Text).await {
		Ok(()) => Ok(Response::new(Empty {})),

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonService] terminal_input failed id={}: {}",
				TerminalIdentifier,
				Error
			);

			Err(Status::not_found(format!("terminal_input: {}", Error)))
		},
	}
}
