#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Command registry handlers - execute and list all registered commands.

use std::sync::Arc;

use CommonLibrary::Command::CommandExecutor::CommandExecutor;
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Execute a command by ID, dispatching to Mountain's CommandExecutor.
pub async fn CommandsExecute(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let CommandId = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "commands:execute requires string command_id as first argument".to_string())?
		.to_string();

	let Argument = Arguments.get(1).cloned().unwrap_or(Value::Null);

	dev_log!("ipc", "commands:execute id={}", CommandId);

	RunTime
		.Environment
		.ExecuteCommand(CommandId, Argument)
		.await
		.map_err(|Error| format!("commands:execute failed: {}", Error))
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
