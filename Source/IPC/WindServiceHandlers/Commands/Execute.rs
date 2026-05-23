//! Wire method: `commands:execute`.
//! Dispatches to Mountain's CommandExecutor and emits
//! `sky://commands/executed` for `vscode.commands.onDidExecuteCommand`.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Command::CommandExecutor::CommandExecutor;

	let CommandId = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "commands:execute requires string command_id as first argument".to_string())?
		.to_string();

	let CommandArgs:Vec<Value> = Arguments.into_iter().skip(1).collect();

	let Argument = CommandArgs.first().cloned().unwrap_or(Value::Null);

	dev_log!("ipc", "commands:execute id={}", CommandId);

	let Result = RunTime
		.Environment
		.ExecuteCommand(CommandId.clone(), Argument)
		.await
		.map_err(|Error| format!("commands:execute failed: {}", Error));

	let _ = RunTime.Environment.ApplicationHandle.emit(
		"sky://commands/executed",
		json!({ "command": CommandId, "arguments": CommandArgs }),
	);

	Result
}
