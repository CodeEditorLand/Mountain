/// matches.
pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		"executeCommand" | "Command.Execute" | "Command.GetAll" => true,

		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Environment::Requires::Requires,
	IPC::DTO::ProxyTarget::ProxyTarget,
};
use serde_json::{Value, json};
use tauri::{Emitter, Runtime};

use crate::{
	Track::Effect::{
		CreateEffectForRequest::Utilities::{
			Params::{string_at, val_at},
			Proxy::proxy_cocoon,
		},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

/// True when the stringified `CommonError::CommandNotFound` shape is seen -
/// the registry dead-end where the command id is unknown to Mountain. Other
/// execution errors (handler threw, IPC failure) must NOT trigger the Cocoon
/// fallback: the command was found and already ran.
fn IsCommandNotFound(Error:&str) -> bool { Error.starts_with("Command '") && Error.ends_with("' not found") }

/// Creates effect.
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

				let FallbackId = command_id.clone();

				let FallbackArgs = args.clone();

				match command_executor.ExecuteCommand(command_id, args).await.map_err(|e| e.to_string()) {
					Err(E) if IsCommandNotFound(&E) => {
						dev_log!(
							"commands",
							"[executeCommand] '{}' missing from Mountain registry; forwarding to Cocoon extension \
							 host.",
							FallbackId
						);

						proxy_cocoon(
							&run_time,
							ProxyTarget::ExtHostCommands,
							"ExecuteContributedCommand",
							json!([FallbackId, FallbackArgs]),
							30_000,
						)
						.await
					},

					Other => Other,
				}
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

				// Palette-invoked extension commands: when the id never made
				// it into Mountain's registry (lost/raced `registerCommand`
				// notification and no `onCommand:` activation event), forward
				// to the Cocoon extension host over the same proxied-RPC
				// method `CommandProvider` uses for registered Proxied
				// commands, instead of dead-ending with CommandNotFound.
				let ExecResult = match ExecResult {
					Err(E) if IsCommandNotFound(&E) => {
						dev_log!(
							"commands",
							"[Command.Execute] '{}' missing from Mountain registry; forwarding to Cocoon extension \
							 host.",
							BroadcastId
						);

						proxy_cocoon(
							&run_time,
							ProxyTarget::ExtHostCommands,
							"ExecuteContributedCommand",
							json!([BroadcastId.clone(), BroadcastArgs.clone()]),
							30_000,
						)
						.await
					},

					Other => Other,
				};

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
					let _ = crate::Vine::Client::SendNotification::Fn(
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
