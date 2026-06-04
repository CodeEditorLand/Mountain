pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		executeCommand, Command.Execute, Command.GetAll => true,
		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{Command::CommandExecutor::CommandExecutor, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::{Emitter, Runtime};

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{string_at, val_at},
	MappedEffectType::MappedEffect,
};
pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:&Value) -> Option<Result<MappedEffect, String>> {
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

				// Capture before the move so the tier-gated dual-emit can
				// reuse them. The executor consumes both by value.
				let BroadcastId = command_id.clone();

				let BroadcastArgs = args.clone();

				let ExecResult = command_executor
					.ExecuteCommand(command_id, args)
					.await
					.map_err(|e| e.to_string());

				// `vscode.commands.onDidExecuteCommand` symmetry. The
				// renderer-originated `commands:execute` Tauri-IPC arm
				// (see WindServiceHandlers/Commands/Execute.rs) already
				// dual-emits `$acceptCommandExecuted`. This arm is hit
				// when an extension running in the Node.js host calls
				// `vscode.commands.executeCommand(...)` and the command
				// is NOT locally registered in Cocoon - the call goes
				// through Mountain's gRPC `Command.Execute` arm instead.
				//
				// Off by default because every executeCommand from the
				// extension host adds an extra Vine notification roundtrip.
				// Flip `TierCommandEventBroadcast=On` to enable.
				let BroadcastEnabled = std::env::var("TierCommandEventBroadcast")
					.unwrap_or_else(|_| env!("TierCommandEventBroadcast", "Off").to_string());

				if BroadcastEnabled == "On" {
					let _ = ::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptCommandExecuted".to_string(),
						json!({
							"command": BroadcastId,
							"arguments": [BroadcastArgs],
						}),
					)
					.await;
				}

				ExecResult
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
