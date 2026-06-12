//! Forward a terminal-opened notification to Sky on
//! `sky://terminal/create` (NOT `/opened` - `SkyBridge.ts:1736` listens on
//! `create` and destructures `{ id, name, pid }`; the `pid` is best-effort
//! 0 here until the real one lands via `AcceptTerminalProcessId`).
use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use ::Vine::Generated::{Empty, TerminalOpenedNotification};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:TerminalOpenedNotification) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Terminal opened notification: {} (ID: {})",
		Request.name,
		Request.terminal_id
	);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/create",
		json!({ "id": Request.terminal_id, "name": Request.name, "pid": 0 }),
	);

	Ok(Response::new(Empty {}))
}
