//! Close a PTY, kill its child, and drop the entry from the
//! provider's terminal registry. Idempotent - disposing an
//! already-disposed id surfaces as a logged warning, not an
//! error.

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let TerminalId = Arguments
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:dispose requires terminal_id as first argument".to_string())?;

	RunTime
		.Environment
		.DisposeTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:dispose failed: {}", Error))
}
