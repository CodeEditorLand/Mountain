#![allow(non_snake_case)]

//! Forward the resolved PID for a terminal to Sky on
//! `sky://terminal/processId`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, TerminalProcessIdNotification},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:TerminalProcessIdNotification) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Terminal PID: {} for terminal {}",
		Request.process_id,
		Request.terminal_id
	);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/processId",
		json!({ "id": Request.terminal_id, "pid": Request.process_id }),
	);

	Ok(Response::new(Empty {}))
}
