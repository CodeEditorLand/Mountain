#![allow(non_snake_case)]

//! Spawn a new PTY through `TerminalProvider::CreateTerminal`.
//! `Options` carries shell path, args, cwd, env, name. Returns a
//! provider-assigned terminal id (`u64`) which Wind uses for
//! every subsequent send/show/dispose call.

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;
use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn TerminalCreate(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Options = Arguments.first().cloned().unwrap_or(Value::Null);

	RunTime
		.Environment
		.CreateTerminal(Options)
		.await
		.map_err(|Error| format!("terminal:create failed: {}", Error))
}
