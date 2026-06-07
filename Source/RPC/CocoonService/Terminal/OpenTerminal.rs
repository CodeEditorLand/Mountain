//! Spawn a new PTY via `TerminalProvider::CreateTerminal`. Builds the
//! options JSON `TerminalStateDTO::Create` expects (name + shellPath +
//! shellArgs + cwd) and forwards through.

use serde_json::json;

use tonic::{Response, Status};

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use ::Vine::Generated::{Empty, OpenTerminalRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:OpenTerminalRequest) -> Result<Response<Empty>, Status> {

	dev_log!("cocoon", "[CocoonService] Opening terminal: {}", Request.name);

	let Options = json!({
		"name": Request.name,
		"shellPath": if Request.shell_path.is_empty() { serde_json::Value::Null } else { json!(Request.shell_path) },
		"shellArgs": Request.shell_args,
		"cwd": if Request.cwd.is_empty() { serde_json::Value::Null } else { json!(Request.cwd) },
	});

	match Service.environment.CreateTerminal(Options).await {
		Ok(Info) => {
			dev_log!("cocoon", "[CocoonService] Terminal created: {:?}", Info);

			Ok(Response::new(Empty {}))
		},

		Err(Error) => {
			dev_log!("cocoon", "error: [CocoonService] open_terminal failed: {}", Error);

			Err(Status::internal(format!("open_terminal: {}", Error)))
		},
	}
}
