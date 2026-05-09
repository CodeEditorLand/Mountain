#![allow(non_snake_case)]

//! Hide a terminal panel without disposing the underlying PTY.
//! The child process keeps running; subsequent `TerminalShow`
//! reopens the same session. Mirrors
//! `vscode.Terminal.hide()`.

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn TerminalHide(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let TerminalId = Arguments.first().and_then(|V| V.as_u64()).unwrap_or(0);

	RunTime
		.Environment
		.HideTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:hide failed: {}", Error))
}
