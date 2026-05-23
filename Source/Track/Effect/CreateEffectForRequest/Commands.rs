use std::sync::Arc;

use CommonLibrary::{Command::CommandExecutor::CommandExecutor, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{string_at, val_at},
	MappedEffectType::MappedEffect,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"executeCommand" => {
			crate::effect!(run_time, {
				let command_executor:Arc<dyn CommandExecutor> = run_time.Environment.Require();
				let (command_id, args) = if let Some(Object) = Parameters.as_object() {
					let Id = Object
						.get("command")
						.or_else(|| Object.get("commandId"))
						.and_then(Value::as_str)
						.unwrap_or("")
						.to_string();
					let A = Object
						.get("args")
						.cloned()
						.unwrap_or_else(|| Object.get("arguments").cloned().unwrap_or_default());
					(Id, A)
				} else {
					let Id = string_at(&Parameters, 0);
					let A = val_at(&Parameters, 1);
					(Id, A)
				};
				command_executor
					.ExecuteCommand(command_id, args)
					.await
					.map_err(|e| e.to_string())
			})
		},

		"Command.Execute" => {
			crate::effect!(run_time, {
				let command_executor:Arc<dyn CommandExecutor> = run_time.Environment.Require();
				let command_id = string_at(&Parameters, 0);
				let args = val_at(&Parameters, 1);
				command_executor
					.ExecuteCommand(command_id, args)
					.await
					.map_err(|e| e.to_string())
			})
		},

		"Command.GetAll" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn CommandExecutor> = run_time.Environment.Require();
				provider
					.GetAllCommands()
					.await
					.map(|cmds| json!(cmds))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
