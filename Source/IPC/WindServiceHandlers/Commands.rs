//! Command registry handlers - execute and list all registered commands.

use std::sync::Arc;

use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use serde_json::{Value, json};

use tauri::Emitter;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Execute a command by ID, dispatching to Mountain's CommandExecutor.
/// Emits `sky://commands/executed` after dispatch so subscribers of
/// `vscode.commands.onDidExecuteCommand` (telemetry collectors, vim,
/// gitlens) observe every command that runs through Mountain.
pub async fn CommandsExecute(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

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

	// Broadcast to `vscode.commands.onDidExecuteCommand` subscribers.
	// Fire-and-forget; failure is non-fatal.
	let _ = RunTime.Environment.ApplicationHandle.emit(
		"sky://commands/executed",

		json!({ "command": CommandId, "arguments": CommandArgs }),
	);

	Result
}

/// Return all registered command IDs from Mountain's CommandRegistry.
pub async fn CommandsGetAll(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {

	let Commands = RunTime
		.Environment
		.GetAllCommands()
		.await
		.map_err(|Error| format!("commands:getAll failed: {}", Error))?;

	Ok(json!(Commands))
}
