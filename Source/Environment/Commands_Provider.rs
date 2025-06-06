// ---------------------------------------------------------------------------------------------
// Mountain Environment - Command Executor Provider
// (environment/commands_provider.rs)
// --------------------------------------------------------------------------------------------
// This module implements the `CommandExecutor` trait for `MountainEnvironment`.
// It handles the execution and management of commands within the application,
// which can be native Mountain commands or commands registered by sidecars.
// Operations are delegated to handler functions in `handlers::commands`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	command_effects::CommandExecutor, // The trait being implemented
	environment::Requires,
	errors::CommonError,
};
use async_trait::async_trait;
use log::{error, info, trace}; // For logging
use serde_json::{Value, json};
use tauri::Manager; // For app_handle.state() and app_handle.get_webview_window()

use crate::{
	environment::MountainEnvironment,
	handlers,            // For delegating to command handlers
	runtime::AppRuntime, // For AppRuntime state access
};

// --- CommandExecutor Implementation ---
#[async_trait]
impl CommandExecutor for MountainEnvironment {
	async fn execute_command(&self, command_id:String, args_val:Value) -> Result<Value, CommonError> {
		info!("[Env CmdExec] Execute: command_id='{}'", command_id);
		trace!("[Env CmdExec] Argument: {:?}", args_val);

		// `handle_execute_command` expects AppHandle, Window, Arc<AppRuntime>, and
		// params. We have AppHandle from `self.app_handle`.
		// We can get Arc<AppRuntime> from AppHandle's managed state.
		// Getting a specific Window context from an effect can be tricky.
		// For now, we'll attempt to get the "main" window. If a command is
		// window-specific and needs a different window, the command execution
		// model or how effects are invoked might need adjustment.
		let main_window = self.app_handle.get_webview_window("main").ok_or_else(|| {
			let msg = "Main window not found for command execution effect. Command might be non-window specific or \
			           window management needs review.";
			error!("[Env CmdExec] {}", msg);
			CommonError::UiInteraction(msg.to_string())
		})?;

		let app_runtime_state = self.app_handle.state::<Arc<AppRuntime>>();
		if app_runtime_state.inner().is_none() {
			// Check if Arc itself is a valid pointer, then check Option
			let msg = "AppRuntime not managed or found in Tauri state for command execution.";
			error!("[Env CmdExec] {}", msg);
			return Err(CommonError::StateLock(msg.to_string()));
		}

		// Delegate to the handler function.
		// The params for `handle_execute_command` are `json!({ "id": command_id,
		// "args": args_val })`
		handlers::commands::handle_execute_command(
			self.app_handle.clone(),
			main_window,
			app_runtime_state.inner().clone(), // Clone the Arc<AppRuntime>
			json!({ "id": command_id, "args": args_val }),
		)
		.await
		.map_err(|json_rpc_err_str| {
			// `handle_execute_command` returns String, convert to CommonError
			CommonError::CommandExecution(command_id, json_rpc_err_str)
		})
	}

	async fn register_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!(
			"[Env CmdExec] Register: sidecar_id='{}', command_id='{}'",
			sidecar_id, command_id
		);

		// Params for `handle_register_command` are `sidecar_id` and `json!({ "id":
		// command_id })`
		handlers::commands::handle_register_command(
            self.app_handle.clone(),
            sidecar_id,
            json!({ "id": command_id }),
        )
        .await
        .map(|_value_null| ()) // Discard Value::Null, handler returns String for error
        .map_err(|err_str| CommonError::CommandRegistration(command_id, err_str))
	}

	async fn unregister_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!(
			"[Env CmdExec] Unregister: sidecar_id='{}', command_id='{}'",
			sidecar_id, command_id
		);

		handlers::commands::handle_unregister_command(self.app_handle.clone(), sidecar_id, json!({ "id": command_id }))
			.await
			.map(|_value_null| ())
			.map_err(|err_str| CommonError::CommandRegistration(command_id, err_str))
	}

	async fn get_all_commands(&self) -> Result<Vec<String>, CommonError> {
		debug!("[Env CmdExec] GetAllCommands");

		let app_runtime_state = self.app_handle.state::<Arc<AppRuntime>>();
		if app_runtime_state.inner().is_none() {
			let msg = "AppRuntime not managed or found in Tauri state for get_all_commands.";
			error!("[Env CmdExec] {}", msg);
			return Err(CommonError::StateLock(msg.to_string()));
		}

		handlers::commands::handle_get_commands(self.app_handle.clone(), app_runtime_state.inner().clone())
			.await
			.and_then(|json_value| {
				// `handle_get_commands` returns Result<Value, String> where Value is
				// Vec<String>
				serde_json::from_value(json_value).map_err(|serde_err| serde_err.to_string()) // Convert serde error to String
			})
			.map_err(CommonError::CommandList) // Convert String error to CommonError::CommandList
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}
