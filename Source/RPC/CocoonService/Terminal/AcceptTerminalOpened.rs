//! Forward a terminal-opened notification to Sky on
//! `sky://terminal/create` (NOT `/opened` - `SkyBridge.ts:1736` listens on
//! `create` and destructures `{ id, name, pid }`; the `pid` is best-effort
//! 0 here until the real one lands via `AcceptTerminalProcessId`).

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, TerminalOpenedNotification},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:TerminalOpenedNotification) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Terminal opened notification: {} (ID: {})",
		Request.name,
		Request.TerminalId
	);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/create",
		json!({ "id": Request.TerminalId, "name": Request.name, "pid": 0 }),
	);

	Ok(Response::new(Empty {}))
}
