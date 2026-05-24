//! Bring a terminal to the foreground in the panel. When
//! `PreserveFocus` is `true`, the active editor keeps keyboard
//! focus (mirrors `vscode.Terminal.show(preserveFocus)`).

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;
use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::{ArgBool, ArgU64},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let TerminalId = ArgU64(&Arguments, 0);

	let PreserveFocus = ArgBool(&Arguments, 1);

	RunTime
		.Environment
		.ShowTerminal(TerminalId, PreserveFocus)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:show failed: {}", Error))
}
