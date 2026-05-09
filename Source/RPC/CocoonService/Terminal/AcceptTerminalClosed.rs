#![allow(non_snake_case)]

//! Forward a terminal-closed notification to Sky on `sky://terminal/exit`.
//! (`/closed` had no consumer.)

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, TerminalClosedNotification},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:TerminalClosedNotification) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Terminal closed: {}", Request.terminal_id);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://terminal/exit", json!({ "id": Request.terminal_id }));

	Ok(Response::new(Empty {}))
}
