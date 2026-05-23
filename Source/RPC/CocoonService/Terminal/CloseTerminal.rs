
//! Dispose a PTY via `TerminalProvider::DisposeTerminal`.

use tonic::{Response, Status};
use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{CloseTerminalRequest, Empty},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:CloseTerminalRequest) -> Result<Response<Empty>, Status> {
	let TerminalIdentifier = Request.terminal_id as u64;

	dev_log!("cocoon", "[CocoonService] close_terminal: id={}", TerminalIdentifier);

	match Service.environment.DisposeTerminal(TerminalIdentifier).await {
		Ok(()) => Ok(Response::new(Empty {})),

		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonService] close_terminal failed id={}: {}",
				TerminalIdentifier,
				Error
			);

			Err(Status::internal(format!("close_terminal: {}", Error)))
		},
	}
}
