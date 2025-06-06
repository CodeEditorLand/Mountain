
// Implements the `CommandExecutor` trait for the `MountainEnvironment`.
// This file connects the abstract command effects to the concrete logic
// in the application's command handlers.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{CommandEffect::CommandExecutor, Environment::Requires, Errors::CommonError};
use async_trait::async_trait;
use log::{debug, error, info, trace};
use serde_json::{Value, json};
use tauri::Manager;

use crate::{Environment::MountainEnvironment, Handlers, Runtime::AppRuntime};

#[async_trait]
impl CommandExecutor for MountainEnvironment {
	/// Executes a command by its identifier with the given arguments.
	async fn ExecuteCommand(&self, CommandIdentifier:String, ArgumentValue:Value) -> Result<Value, CommonError> {
		info!(
			"[Environment CommandExecutor] ExecuteCommand: Identifier='{}'",
			CommandIdentifier
		);
		trace!("[Environment CommandExecutor] Argument: {:?}", ArgumentValue);

		let MainWindow = self.AppHandle.get_webview_window("main").ok_or_else(|| {
			let Message = "Main window not found for command execution effect.";
			error!("[Environment CommandExecutor] {}", Message);
			CommonError::UiInteraction { Reason:Message.to_string() }
		})?;

		let ApplicationRuntimeState = self.AppHandle.state::<Arc<AppRuntime>>();
		if ApplicationRuntimeState.inner().is_none() {
			let Message = "AppRuntime not managed or found in Tauri state for command execution.";
			error!("[Environment CommandExecutor] {}", Message);
			return Err(CommonError::StateLock { Context:Message.to_string() });
		}

		// The handler expects a JSON object with specific keys.
		let HandlerParameters = json!({
			"id": CommandIdentifier.clone(),
			"args": ArgumentValue
		});

		Handlers::Commands::HandleExecuteCommand(
			self.AppHandle.clone(),
			MainWindow,
			ApplicationRuntimeState.inner().clone(),
			HandlerParameters,
		)
		.await
		.map_err(|JsonRpcErrorString| CommonError::CommandExecution { CommandIdentifier, Reason:JsonRpcErrorString })
	}

	/// Registers a command from a sidecar process.
	async fn RegisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		info!(
			"[Environment CommandExecutor] RegisterCommand: Sidecar='{}', Identifier='{}'",
			SidecarIdentifier, CommandIdentifier
		);
		Handlers::Commands::HandleRegisterCommand(
			self.AppHandle.clone(),
			SidecarIdentifier,
			json!({ "id": CommandIdentifier.clone() }),
		)
		.await
		.map(|_ValueNull| ())
		.map_err(|ErrorString| CommonError::CommandRegistration { CommandIdentifier, Reason:ErrorString })
	}

	/// Unregisters a command from a sidecar process.
	async fn UnregisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		info!(
			"[Environment CommandExecutor] UnregisterCommand: Sidecar='{}', Identifier='{}'",
			SidecarIdentifier, CommandIdentifier
		);
		Handlers::Commands::HandleUnregisterCommand(
			self.AppHandle.clone(),
			SidecarIdentifier,
			json!({ "id": CommandIdentifier.clone() }),
		)
		.await
		.map(|_ValueNull| ())
		.map_err(|ErrorString| CommonError::CommandRegistration { CommandIdentifier, Reason:ErrorString })
	}

	/// Retrieves a list of all currently registered command identifiers.
	async fn GetAllCommands(&self) -> Result<Vec<String>, CommonError> {
		debug!("[Environment CommandExecutor] GetAllCommands");

		let ApplicationRuntimeState = self.AppHandle.state::<Arc<AppRuntime>>();
		if ApplicationRuntimeState.inner().is_none() {
			let Message = "AppRuntime not managed or found in Tauri state for GetAllCommands.";
			error!("[Environment CommandExecutor] {}", Message);
			return Err(CommonError::StateLock { Context:Message.to_string() });
		}

		Handlers::Commands::handle_get_commands(self.AppHandle.clone(), ApplicationRuntimeState.inner().clone())
			.await
			.and_then(|JsonValue| serde_json::from_value(JsonValue).map_err(|SerdeError| SerdeError.to_string()))
			.map_err(|Reason| CommonError::CommandList { Reason })
	}
}

impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}
