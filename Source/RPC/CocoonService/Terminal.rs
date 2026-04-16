#![allow(non_snake_case)]
//! Terminal domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: open_terminal, terminal_input, close_terminal,
//! accept_terminal_opened, accept_terminal_closed,
//! accept_terminal_process_id, accept_terminal_process_data,
//! resize_terminal.

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;
use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::dev_log;
use crate::Vine::Generated::{
	CloseTerminalRequest, Empty, OpenTerminalRequest, ResizeTerminalRequest,
	TerminalClosedNotification, TerminalDataNotification, TerminalInputRequest,
	TerminalOpenedNotification, TerminalProcessIdNotification,
};

pub async fn OpenTerminal(
	Service:&CocoonServiceImpl,
	req:OpenTerminalRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Opening terminal: {}", req.name);

	// Build options JSON matching TerminalStateDTO::Create expectations
	let Options = json!({
		"name": req.name,
		"shellPath": if req.shell_path.is_empty() { serde_json::Value::Null } else { json!(req.shell_path) },
		"shellArgs": req.shell_args,
		"cwd": if req.cwd.is_empty() { serde_json::Value::Null } else { json!(req.cwd) },
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

pub async fn TerminalInput(
	Service:&CocoonServiceImpl,
	req:TerminalInputRequest,
) -> Result<Response<Empty>, Status> {
	let TerminalId = req.terminal_id as u64;
	dev_log!("cocoon", "[CocoonService] terminal_input: id={} bytes={}", TerminalId, req.data.len());

	let Text = String::from_utf8_lossy(&req.data).into_owned();

	match Service.environment.SendTextToTerminal(TerminalId, Text).await {
		Ok(()) => Ok(Response::new(Empty {})),
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] terminal_input failed id={}: {}", TerminalId, Error);
			Err(Status::not_found(format!("terminal_input: {}", Error)))
		},
	}
}

pub async fn CloseTerminal(
	Service:&CocoonServiceImpl,
	req:CloseTerminalRequest,
) -> Result<Response<Empty>, Status> {
	let TerminalId = req.terminal_id as u64;
	dev_log!("cocoon", "[CocoonService] close_terminal: id={}", TerminalId);

	match Service.environment.DisposeTerminal(TerminalId).await {
		Ok(()) => Ok(Response::new(Empty {})),
		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] close_terminal failed id={}: {}", TerminalId, Error);
			Err(Status::internal(format!("close_terminal: {}", Error)))
		},
	}
}

pub async fn AcceptTerminalOpened(
	Service:&CocoonServiceImpl,
	req:TerminalOpenedNotification,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon",
		"[CocoonService] Terminal opened notification: {} (ID: {})",
		req.name, req.terminal_id
	);

	// Forward terminal opened event to Sky for UI update
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/opened",
		json!({ "id": req.terminal_id, "name": req.name }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn AcceptTerminalClosed(
	Service:&CocoonServiceImpl,
	req:TerminalClosedNotification,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Terminal closed: {}", req.terminal_id);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/closed",
		json!({ "id": req.terminal_id }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn AcceptTerminalProcessId(
	Service:&CocoonServiceImpl,
	req:TerminalProcessIdNotification,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Terminal PID: {} for terminal {}", req.process_id, req.terminal_id);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/processId",
		json!({ "id": req.terminal_id, "pid": req.process_id }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn AcceptTerminalProcessData(
	Service:&CocoonServiceImpl,
	req:TerminalDataNotification,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Terminal data for {}: {} bytes", req.terminal_id, req.data.len());

	let DataString = String::from_utf8_lossy(&req.data).to_string();
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/data",
		json!({ "id": req.terminal_id, "data": DataString }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn ResizeTerminal(
	Service:&CocoonServiceImpl,
	req:ResizeTerminalRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon",
		"[CocoonService] resize_terminal: id={} cols={} rows={}",
		req.terminal_id, req.cols, req.rows
	);

	// Notify Sky/Wind of the new dimensions for UI resize
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://terminal/resize",
		json!({ "id": req.terminal_id, "cols": req.cols, "rows": req.rows }),
	);

	// TODO(P1): Call portable_pty::MasterPty::resize once PtyMaster handle
	// is stored in TerminalStateDTO (requires wrapping MasterPty in Arc<Mutex>)

	Ok(Response::new(Empty {}))
}
