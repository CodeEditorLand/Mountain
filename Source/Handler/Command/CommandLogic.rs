use std::{collections::HashMap, sync::Arc};

use Common::{error::CommonError, ipc::dto::ProxyTarget};
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{ApplicationHandle, Manager, RunTime, State, Window};

// @module CommandsLogic
// @description Contains the core logic for managing the command registry and
// dispatching command execution to either native Rust Handler or proxied
// sidecar Handler.
use super::CommandHandler::CommandHandler;
use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	Handler::error_utils,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	vine::{self, client},
};

// Logic to execute a command, dispatching to the appropriate handler.
pub async fn ExecuteCommandLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	CommandIdentifier:String,
	Argument:Value,
) -> Result<Value, CommonError> {
	let HandlerInfoOption = ApplicationHandle
		.state::<ApplicationState>()
		.CommandRegistry
		.lock()
		.unwrap()
		.get(&CommandIdentifier)
		.cloned();

	match HandlerInfoOption {
		Some(CommandHandler::Native(Function)) => {
			debug!("[CommandsLogic] Executing NATIVE command '{}'.", CommandIdentifier);
			let RunTimeState:State<'_, Arc<ApplicationRunTime>> = ApplicationHandle.state();
			let Window = ApplicationHandle
				.get_webview_window("main")
				.ok_or_else(|| CommonError::UiInteraction { Reason:"Main window not found".into() })?;
			Function(ApplicationHandle.clone(), Window, RunTimeState.inner().clone(), Argument)
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

// Logic to register a command contributed by a sidecar process.
pub async fn RegisterCommandLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	SidecarIdentifier:String,
	CommandIdentifier:String,
) -> Result<(), CommonError> {
	info!(
		"[CommandsLogic] Registering PROXY command '{}' from sidecar '{}'",
		CommandIdentifier, SidecarIdentifier
	);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let mut Registry = AppStateInstance.CommandRegistry.lock().unwrap();
	Registry.insert(
		CommandIdentifier.clone(),
		CommandHandler::Proxied { SidecarIdentifier, CommandIdentifier },
	);
	Ok(())
}

// Logic to unregister a previously registered command.
pub async fn UnregisterCommandLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	_SidecarIdentifier:String,
	CommandIdentifier:String,
) -> Result<(), CommonError> {
	info!("[CommandsLogic] Unregistering command '{}'", CommandIdentifier);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	AppStateInstance.CommandRegistry.lock().unwrap().remove(&CommandIdentifier);
	Ok(())
}

// Logic to get a list of all currently registered command IDs.
pub async fn GetAllCommandsLogic<R:RunTime>(ApplicationHandle:&ApplicationHandle<R>) -> Result<Vec<String>, CommonError> {
	debug!("[CommandsLogic] Getting all command identifiers.");
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let Registry = AppStateInstance.CommandRegistry.lock().unwrap();
	Ok(Registry.keys().cloned().collect())
}

// Initializes the command registry with all native Mountain commands at
// startup.
pub fn RegisterNativeCommands<R:RunTime + 'static>() -> HashMap<String, CommandHandler<R>> {
	let mut Registry = HashMap::new();
	// Example of registering a native command:
	// Registry.insert(
	//     "mountain.ShowWorkspaceInfo".into(),
	//     CommandHandler::Native(crate::Handler::workspace::ShowWorkspaceInfoCommand),
	// );
	info!("[CommandsLogic] Native commands registered.");
	Registry
}
