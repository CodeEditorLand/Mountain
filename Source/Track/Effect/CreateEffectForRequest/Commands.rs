use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Command::CommandExecutor::CommandExecutor, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"executeCommand" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
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
							let Id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
							let A = Parameters.get(1).cloned().unwrap_or_default();
							(Id, A)
						};
						command_executor
							.ExecuteCommand(command_id, args)
							.await
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Command.Execute" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let command_executor:Arc<dyn CommandExecutor> = run_time.Environment.Require();
						let command_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let args = Parameters.get(1).cloned().unwrap_or_default();
						command_executor
							.ExecuteCommand(command_id, args)
							.await
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Command.GetAll" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn CommandExecutor> = run_time.Environment.Require();
						provider
							.GetAllCommands()
							.await
							.map(|cmds| json!(cmds))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
