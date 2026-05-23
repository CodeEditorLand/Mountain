#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `commands:getAll`.
//! Returns all registered command IDs from Mountain's CommandRegistry.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Command::CommandExecutor::CommandExecutor;

	let Commands = RunTime
		.Environment
		.GetAllCommands()
		.await
		.map_err(|Error| format!("commands:getAll failed: {}", Error))?;

	Ok(json!(Commands))
}
