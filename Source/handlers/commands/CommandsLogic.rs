use std::{collections::HashMap, sync::Arc};

use Common::{error::CommonError, ipc::dto::ProxyTarget};
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State, Window};

/// @module CommandsLogic
/// @description Contains the core logic for managing the command registry and
/// dispatching command execution to either native Rust handlers or proxied
/// sidecar handlers.
use super::CommandHandler::CommandHandler;
use crate::{
	AppState::AppState::AppState,
	handlers::error_utils,
	runtime::AppRuntime::AppRuntime,
	vine::{self, client},
};

/// Logic to execute a command, dispatching to the appropriate handler.
pub async fn ExecuteCommandLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	CommandIdentifier:String,
	Argument:Value,
) -> Result<Value, CommonError> {
	let HandlerInfoOption = AppHandle
		.state::<AppState>()
		.CommandRegistry
		.lock()
		.unwrap()
		.get(&CommandIdentifier)
		.cloned();

	match HandlerInfoOption {
		Some(CommandHandler::Native(Function)) => {
			debug!("[CommandsLogic] Executing NATIVE command '{}'.", CommandIdentifier);
			let RuntimeState:State<'_, Arc<AppRuntime>> = AppHandle.state();
			let Window = AppHandle
				.get_webview_window("main")
				.ok_or_else(|| CommonError::UiInteraction { Reason:"Main window not found".into() })?;
			Function(AppHandle.clone(), Window, RuntimeState.inner().clone(), Argument)
				.await
				.map_err(|e| CommonError::CommandExecution { CommandIdentifier:CommandIdentifier.clone(), Reason:e })
		},
		Some(CommandHandler::Proxied { SidecarIdentifier, CommandIdentifier: ProxiedCommandIdentifier }) => {
			debug!(
				"[CommandsLogic] Executing PROXIED command '{}' on sidecar '{}'.",
				CommandIdentifier, SidecarIdentifier
			);
			let RpcParameters = json!([ProxiedCommandIdentifier, Argument]);
			let RpcMethod = format!("{}$executeContributedCommand", ProxyTarget::ExtHostCommands.GetTargetPrefix());
			client::SendRequest(&SidecarIdentifier, RpcMethod, RpcParameters, 30000).await
		},
		None => {
			error!("[CommandsLogic] Command '{}' not found in registry.", CommandIdentifier);
			Err(CommonError::CommandNotFound {
				Feature:"command".into(),
				DocumentUri:CommandIdentifier, // Using this field to store the command ID
			})
		},
	}
}

/// Logic to register a command contributed by a sidecar process.
pub async fn RegisterCommandLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	SidecarIdentifier:String,
	CommandIdentifier:String,
) -> Result<(), CommonError> {
	info!(
		"[CommandsLogic] Registering PROXY command '{}' from sidecar '{}'",
		CommandIdentifier, SidecarIdentifier
	);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut Registry = AppStateInstance.CommandRegistry.lock().unwrap();
	Registry.insert(
		CommandIdentifier.clone(),
		CommandHandler::Proxied { SidecarIdentifier, CommandIdentifier },
	);
	Ok(())
}

/// Logic to unregister a previously registered command.
pub async fn UnregisterCommandLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	_SidecarIdentifier:String,
	CommandIdentifier:String,
) -> Result<(), CommonError> {
	info!("[CommandsLogic] Unregistering command '{}'", CommandIdentifier);
	let AppStateInstance = AppHandle.state::<AppState>();
	AppStateInstance.CommandRegistry.lock().unwrap().remove(&CommandIdentifier);
	Ok(())
}

/// Logic to get a list of all currently registered command IDs.
pub async fn GetAllCommandsLogic<R:Runtime>(AppHandle:&AppHandle<R>) -> Result<Vec<String>, CommonError> {
	debug!("[CommandsLogic] Getting all command identifiers.");
	let AppStateInstance = AppHandle.state::<AppState>();
	let Registry = AppStateInstance.CommandRegistry.lock().unwrap();
	Ok(Registry.keys().cloned().collect())
}

/// Initializes the command registry with all native Mountain commands at
/// startup.
pub fn RegisterNativeCommands<R:Runtime + 'static>() -> HashMap<String, CommandHandler<R>> {
	let mut Registry = HashMap::new();
	// Example of registering a native command:
	// Registry.insert(
	//     "mountain.ShowWorkspaceInfo".into(),
	//     CommandHandler::Native(crate::handlers::workspace::ShowWorkspaceInfoCommand),
	// );
	info!("[CommandsLogic] Native commands registered.");
	Registry
}
