#![allow(non_snake_case)]

//! Command registry domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};
use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Execute a command by ID, dispatching to Mountain's CommandExecutor.
pub async fn handle_commands_execute(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let CommandId = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "commands:execute requires string command_id as first argument".to_string())?
		.to_string();

	let Argument = Args.get(1).cloned().unwrap_or(Value::Null);

	dev_log!("ipc", "commands:execute id={}", CommandId);

	Runtime
		.Environment
		.ExecuteCommand(CommandId, Argument)
		.await
		.map_err(|Error| format!("commands:execute failed: {}", Error))
}

/// Return all registered command IDs from Mountain's CommandRegistry.
pub async fn handle_commands_get_all(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Commands = Runtime
		.Environment
		.GetAllCommands()
		.await
		.map_err(|Error| format!("commands:getAll failed: {}", Error))?;

	Ok(json!(Commands))
}
