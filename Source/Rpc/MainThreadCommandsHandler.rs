// File: Rpc/MainThreadCommandsHandler.rs
// Defines the RPC handler for commands originating from the sidecar (e.g.,
// Cocoon) and to be executed within the Mountain environment or proxied
// further.

use std::sync::Arc;

use log::{debug, info, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Window, Wry};

use crate::Handlers::{self, ErrorUtils}; // Assuming ErrorUtils will be PascalCased
use crate::{
	Rpc::Args::Commands::{ExecuteCommandArgument, RegisterArgument},
	Runtime::AppRuntime,
};

#[derive(Clone)]
pub struct MainThreadCommandsHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadCommandsHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Executes a command.
	/// This can be a command registered natively in Mountain or a command
	/// proxied from a sidecar.
	pub async fn ExecuteCommand(&self, Argument:ExecuteCommandArgument) -> Result<Value, String> {
		let CommandIdentifier = Argument.CommandIdentifier;
		let CommandArgumentsVec = Argument.CommandArguments;

		debug!(
			"[Rpc MainThreadCommands] ExecuteCommand (DTO): CommandIdentifier='{}', ArgumentCount={}",
			CommandIdentifier,
			CommandArgumentsVec.len()
		);

		let MainWindow = self.ApplicationHandle.get_webview_window("main").ok_or_else(|| {
			ErrorUtils::RpcErrorString(
				"Main window not found for command execution.".to_string(),
				Some("ENOWINDOW_CMDEXEC"),
			)
		})?;

		// The `handle_execute_command` function expects a specific JSON structure for
		// its `params` argument. We need to reconstruct this structure from our DTO.
		let HandlerParams = json!({
			"id": CommandIdentifier,
			"args": CommandArgumentsVec
		});

		// Assuming AppRuntime is managed correctly in Tauri's state
		let RuntimeState:State<'_, Arc<AppRuntime>> = self.ApplicationHandle.state();

		Handlers::Commands::HandleExecuteCommand(
			self.ApplicationHandle.clone(),
			MainWindow,
			RuntimeState,
			HandlerParams,
		)
		.await
	}

	/// Retrieves a list of all available commands.
	pub async fn GetCommands(&self, _Arguments:Value) -> Result<Value, String> {
		debug!("[Rpc MainThreadCommands] GetCommands (DTO flow)");
		// Assuming AppRuntime is managed correctly in Tauri's state
		let RuntimeState:State<'_, Arc<AppRuntime>> = self.ApplicationHandle.state();
		Handlers::Commands::HandleGetCommands(self.ApplicationHandle.clone(), RuntimeState).await
	}

	/// Registers a command from a sidecar.
	pub async fn RegisterCommand(&self, SidecarIdentifier:&str, Argument:RegisterArgument) -> Result<Value, String> {
		info!(
			"[Rpc MainThreadCommands] RegisterCommand (DTO): Identifier='{}' from Sidecar='{}'",
			Argument.Id, SidecarIdentifier
		);
		// The underlying handler expects a simple JSON object with an "id" field.
		let HandlerParams = json!({ "id": Argument.Id });
		Handlers::Commands::HandleRegisterCommand(
			self.ApplicationHandle.clone(),
			SidecarIdentifier.to_string(),
			HandlerParams,
		)
		.await
	}

	/// Unregisters a command previously registered by a sidecar.
	pub async fn UnregisterCommand(&self, SidecarIdentifier:&str, Argument:RegisterArgument) -> Result<Value, String> {
		info!(
			"[Rpc MainThreadCommands] UnregisterCommand (DTO): Identifier='{}' from Sidecar='{}'",
			Argument.Id, SidecarIdentifier
		);
		// The underlying handler expects a simple JSON object with an "id" field.
		let HandlerParams = json!({ "id": Argument.Id });
		Handlers::Commands::HandleUnregisterCommand(
			self.ApplicationHandle.clone(),
			SidecarIdentifier.to_string(),
			HandlerParams,
		)
		.await
	}
}
