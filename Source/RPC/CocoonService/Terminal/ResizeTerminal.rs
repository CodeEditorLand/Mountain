#![allow(non_snake_case)]

//! Notify Sky/Wind of new terminal dimensions on `sky://terminal/resize`.
//! TODO(P1): also call `portable_pty::MasterPty::resize` once the master
//! handle is stored in `TerminalStateDTO` (requires wrapping in
//! `Arc<Mutex>`).

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, ResizeTerminalRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:ResizeTerminalRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] resize_terminal: id={} cols={} rows={}",
		Request.terminal_id,
		Request.cols,
		Request.rows
	);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/resize",
		json!({ "id": Request.terminal_id, "cols": Request.cols, "rows": Request.rows }),
	);

	Ok(Response::new(Empty {}))
}
