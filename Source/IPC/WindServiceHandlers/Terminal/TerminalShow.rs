
//! Bring a terminal to the foreground in the panel. When
//! `PreserveFocus` is `true`, the active editor keeps keyboard
//! focus (mirrors `vscode.Terminal.show(preserveFocus)`).

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;
use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let TerminalId = Arguments.first().and_then(|V| V.as_u64()).unwrap_or(0);

	let PreserveFocus = Arguments.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	RunTime
		.Environment
		.ShowTerminal(TerminalId, PreserveFocus)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:show failed: {}", Error))
}
