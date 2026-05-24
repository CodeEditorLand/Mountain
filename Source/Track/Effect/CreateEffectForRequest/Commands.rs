use std::sync::Arc;

use CommonLibrary::{Command::CommandExecutor::CommandExecutor, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{StringAt, ValAt},
	MappedEffectType::MappedEffect,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"executeCommand" => {
			crate::effect!(RunTime, {
				let command_executor:Arc<dyn CommandExecutor> = RunTime.Environment.Require();
				let (command_id, args) = if let Some(Object) = Parameters.as_object() {
					let Id = Object
						.Get("command")
						.or_else(|| Object.get("commandId"))
						.and_then(Value::as_str)
						.unwrap_or("")
						.to_string();
					let A = Object
						.Get("args")
						.cloned()
						.unwrap_or_else(|| Object.get("arguments").cloned().unwrap_or_default());
					(Id, A)
				} else {
					let Id = StringAt(&Parameters, 0);
					let A = ValAt(&Parameters, 1);
					(Id, A)
				};
				command_executor
					.ExecuteCommand(command_id, args)
					.await
					.map_err(|E| e.to_string())
			})
		},

		"Command.Execute" => {
			crate::effect!(RunTime, {
				let command_executor:Arc<dyn CommandExecutor> = RunTime.Environment.Require();
				let command_id = StringAt(&Parameters, 0);
				let Args = ValAt(&Parameters, 1);
				command_executor
					.ExecuteCommand(command_id, args)
					.await
					.map_err(|E| e.to_string())
			})
		},

		"Command.GetAll" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn CommandExecutor> = RunTime.Environment.Require();
				provider
					.GetAllCommands()
					.await
					.map(|cmds| json!(cmds))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
