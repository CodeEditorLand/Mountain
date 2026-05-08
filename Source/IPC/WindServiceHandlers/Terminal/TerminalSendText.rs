#![allow(non_snake_case)]

//! Pipe text into a terminal's PTY stdin. Used both for direct
//! key forwarding (xterm.js → Mountain → PTY) and for
//! programmatic input (`vscode.window.terminals[…].sendText`).

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;
use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn TerminalSendText(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let TerminalId = Arguments
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:sendText requires terminal_id as first argument".to_string())?;

	let Text = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	RunTime
		.Environment
		.SendTextToTerminal(TerminalId, Text)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:sendText failed: {}", Error))
}
