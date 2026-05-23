
//! `resize_terminal` gRPC endpoint. Resizes the PTY backing a terminal so
//! the shell receives SIGWINCH and readline/shells repaint correctly. Also
//! emits `sky://terminal/resize` so the Sky xterm.js panel reflows its
//! viewport to match.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, ResizeTerminalRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:ResizeTerminalRequest) -> Result<Response<Empty>, Status> {
	let TerminalId = Request.terminal_id;

	let Cols = Request.cols.max(1) as u16;

	let Rows = Request.rows.max(1) as u16;

	dev_log!(
		"cocoon",
		"[CocoonService] resize_terminal id={} cols={} rows={}",
		TerminalId,
		Cols,
		Rows
	);

	// Resize the actual PTY (sends SIGWINCH so readline/zsh repaint).
	let Provider:std::sync::Arc<dyn TerminalProvider> = Service.environment.Require();

	if let Err(Error) = Provider.ResizeTerminal(TerminalId.into(), Cols, Rows).await {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] resize_terminal id={} failed: {}",
			TerminalId,
			Error
		);
	}

	// Notify Sky so xterm.js reflows its viewport.
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://terminal/resize", json!({ "id": TerminalId, "cols": Cols, "rows": Rows }));

	Ok(Response::new(Empty {}))
}
