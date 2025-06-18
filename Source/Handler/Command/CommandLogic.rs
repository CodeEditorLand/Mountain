// @module CommandLogic
// @description Contains the core logic for managing the command registry and
// dispatching command execution to either native Rust handlers or proxied
// sidecar handlers.

use std::{collections::HashMap, sync::Arc};

use Common::{error::CommonError, IPC::DTO::ProxyTarget};
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State, Window};

use super::CommandHandler::CommandHandler;
use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Vine::client,
};

// Logic to execute a command, dispatching to the appropriate handler.
pub async fn ExecuteCommandLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	command_identifier:String,
	argument:Value,
) -> Result<Value, CommonError> {
	let handler_info_option = app_handle
		.state::<ApplicationState>()
		.CommandRegistry
		.lock()
		.unwrap()
		.get(&command_identifier)
		.cloned();

	match handler_info_option {
		Some(CommandHandler::Native(function)) => {
			debug!("[CommandLogic] Executing NATIVE command '{}'.", command_identifier);
			let runtime_state:State<'_, Arc<ApplicationRunTime>> = app_handle.state();
			let window = app_handle.get_window("main").ok_or_else(|| {
				CommonError::UiInteraction { Reason:"Main window not found for command execution".into() }
			})?;
			function(app_handle.clone(), window, runtime_state.inner().clone(), argument)
				.await
				.map_err(|e| CommonError::CommandExecution { CommandIdentifier:command_identifier.clone(), Reason:e })
		},
		Some(CommandHandler::Proxied { SidecarIdentifier, CommandIdentifier: proxied_command_identifier }) => {
			debug!(
				"[CommandLogic] Executing PROXIED command '{}' on sidecar '{}'.",
				command_identifier, SidecarIdentifier
			);
			let rpc_parameters = json!([proxied_command_identifier, argument]);
			let rpc_method = format!("{}$executeContributedCommand", ProxyTarget::ExtHostCommand.GetTargetPrefix());
			client::SendRequest(SidecarIdentifier, rpc_method, rpc_parameters, 30000).await
		},
		None => {
			error!("[CommandLogic] Command '{}' not found in registry.", command_identifier);
			Err(CommonError::CommandNotFound { Feature:"command".into(), Identifier:command_identifier })
		},
	}
}

// Logic to register a command contributed by a sidecar process.
pub async fn RegisterCommandLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	sidecar_identifier:String,
	command_identifier:String,
) -> Result<(), CommonError> {
	info!(
		"[CommandLogic] Registering PROXY command '{}' from sidecar '{}'",
		command_identifier, sidecar_identifier
	);
	let app_state = app_handle.state::<ApplicationState>();
	let mut registry = app_state.CommandRegistry.lock().unwrap();
	registry.insert(
		command_identifier.clone(),
		CommandHandler::Proxied { SidecarIdentifier:sidecar_identifier, CommandIdentifier:command_identifier },
	);
	Ok(())
}

// Logic to unregister a previously registered command.
pub async fn UnregisterCommandLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	_sidecar_identifier:String,
	command_identifier:String,
) -> Result<(), CommonError> {
	info!("[CommandLogic] Unregistering command '{}'", command_identifier);
	let app_state = app_handle.state::<ApplicationState>();
	app_state.CommandRegistry.lock().unwrap().remove(&command_identifier);
	Ok(())
}

// Logic to get a list of all currently registered command IDs.
pub async fn GetAllCommandLogic<R:Runtime>(app_handle:&AppHandle<R>) -> Result<Vec<String>, CommonError> {
	debug!("[CommandLogic] Getting all command identifiers.");
	let app_state = app_handle.state::<ApplicationState>();
	let registry = app_state.CommandRegistry.lock().unwrap();
	Ok(registry.keys().cloned().collect())
}

// Initializes the command registry with all native Mountain commands at
// startup.
pub fn RegisterNativeCommand<R:Runtime + 'static>() -> HashMap<String, CommandHandler<R>> {
	let registry = HashMap::new();
	// Example of registering a native command:
	// registry.insert(
	//     "mountain.ShowWorkspaceInfo".into(),
	//     CommandHandler::Native(crate::Handler::workspace::ShowWorkspaceInfoCommand),
	// );
	info!("[CommandLogic] Native command registry initialized (currently empty).");
	registry
}
