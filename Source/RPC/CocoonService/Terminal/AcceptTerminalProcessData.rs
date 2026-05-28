//! Forward terminal stdout bytes to Sky on `sky://terminal/data` as
//! lossy-decoded UTF-8.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{Empty, TerminalDataNotification};

pub async fn Fn(Service:&CocoonServiceImpl, Request:TerminalDataNotification) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Terminal data for {}: {} bytes",
		Request.terminal_id,
		Request.data.len()
	);

	let DataString = String::from_utf8_lossy(&Request.data).to_string();

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://terminal/data", json!({ "id": Request.terminal_id, "data": DataString }));

	Ok(Response::new(Empty {}))
}
