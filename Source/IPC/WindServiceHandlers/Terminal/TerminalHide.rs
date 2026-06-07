//! Hide a terminal panel without disposing the underlying PTY.
//! The child process keeps running; subsequent `TerminalShow`
//! reopens the same session. Mirrors
//! `vscode.Terminal.hide()`.

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_u64,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let TerminalId = arg_u64(&Arguments, 0);

	RunTime
		.Environment
		.HideTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:hide failed: {}", Error))
}
