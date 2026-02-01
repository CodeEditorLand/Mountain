// File: Mountain/Source/Environment/CommandProvider.rs
//
// # Architectural Role: Command Registration and Execution
//
// CommandProvider implements the CommandExecutor trait, serving as the central
// registry and dispatcher for all commands in the Mountain application.
// Commands can be handled either by native Rust handlers or proxied to
// extension sidecar processes.
//
// # Responsibilities
//
// 1. **Command Registry**: Maintains a centralized registry of all registered
//    commands and their corresponding handlers (native or proxied).
//
// 2. **Command Dispatching**: Routes command execution requests to the
//    appropriate handler based on the command identifier.
//
// 3. **Extension Command Proxying**: Enables extensions to contribute commands
//    that are executed in their sidecar processes via IPC.
//
// 4. **Command Lifecycle Management**: Handles registration, unregistration,
//    and querying of available commands.
//
// # Command Execution Flow
//
// 1. Extension or system calls ExecuteCommand(identifier, args)
// 2. CommandProvider looks up the command in ApplicationState.CommandRegistry
// 3. If native handler: executes Rust function directly with AppHandle and
//    arguments
// 4. If proxied handler: sends IPC request to the owning sidecar via Vine
//    client
// 5. Returns result or error to caller
//
// # Patterns Borrowed from VSCode
//
// - **Command Registry**: Follows VSCode's command registry pattern where
//   commands are identified by strings and can be contributed by extensions.
//
// - **Context Passing**: Like VSCode's execution context, Mountain passes the
//   AppHandle and Runtime to native handlers for context awareness.
//
// - **Conflict Resolution**: VSCode allows command overrides; Mountain
//   currently does not implement conflict resolution (TODO).
//
// # TODOs
//
// - [ ] Implement command conflict resolution strategy
// - [ ] Add command execution context (selection, active editor, etc.)
// - [ ] Implement command categories and metadata
// - [ ] Add command enablement/disablement based on context
// - [ ] Implement command execution metrics and telemetry
// - [ ] Add command keyboard shortcut registration lookup
// - [ ] Implement command execution timeout and cancellation
// - [ ] Add validation of command arguments
// - [ ] Consider adding command preconditions
// - [ ] Implement command history for undo/redo scenarios

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Error::CommonError::CommonError,
	IPC::DTO::ProxyTarget::ProxyTarget,
};
use async_trait::async_trait;
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use super::MountainEnvironment::MountainEnvironment;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Vine::Client};

/// An enum representing the different ways a command can be handled.
pub enum CommandHandler<R:Runtime + 'static> {
	/// A command handled by a native, asynchronous Rust function.
	Native(
		fn(
			AppHandle<R>,

			WebviewWindow<R>,

			Arc<ApplicationRunTime>,

			Value,
		) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>,
	),

	/// A command implemented in an extension and proxied to a sidecar.
	Proxied { SideCarIdentifier:String, CommandIdentifier:String },
}

impl<R:Runtime> Clone for CommandHandler<R> {
	fn clone(&self) -> Self {
		match self {
			Self::Native(Function) => Self::Native(*Function),

			Self::Proxied { SideCarIdentifier, CommandIdentifier } => {
				Self::Proxied {
					SideCarIdentifier:SideCarIdentifier.clone(),

					CommandIdentifier:CommandIdentifier.clone(),
				}
			},
		}
	}
}

#[async_trait]
impl CommandExecutor for MountainEnvironment {
	/// Executes a registered command by dispatching it to the appropriate
	/// handler.
	async fn ExecuteCommand(&self, CommandIdentifier:String, Argument:Value) -> Result<Value, CommonError> {
		let HandlerInfoOption = self
			.ApplicationState
			.CommandRegistry
			.lock()
			.map_err(super::Utility::MapApplicationStateLockErrorToCommonError)?
			.get(&CommandIdentifier)
			.cloned();

		match HandlerInfoOption {
			Some(CommandHandler::Native(Function)) => {
				debug!("[CommandProvider] Executing NATIVE command '{}'.", CommandIdentifier);

				let RunTime:Arc<ApplicationRunTime> =
					self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

				let MainWindow = self.ApplicationHandle.get_webview_window("main").ok_or_else(|| {
					CommonError::UserInterfaceInteraction {
						Reason:"Main window not found for command execution".into(),
					}
				})?;

				Function(self.ApplicationHandle.clone(), MainWindow, RunTime, Argument)
					.await
					.map_err(|Error| CommonError::CommandExecution { CommandIdentifier, Reason:Error })
			},

			Some(CommandHandler::Proxied { SideCarIdentifier, CommandIdentifier: ProxiedCommandIdentifier }) => {
				debug!(
					"[CommandProvider] Executing PROXIED command '{}' on sidecar '{}'.",
					CommandIdentifier, SideCarIdentifier
				);

				let RPCParameters = json!([ProxiedCommandIdentifier, Argument]);

				let RPCMethod = format!("{}$ExecuteContributedCommand", ProxyTarget::ExtHostCommands.GetTargetPrefix());

				Client::SendRequest(&SideCarIdentifier, RPCMethod, RPCParameters, 30000)
					.await
					.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })
			},

			None => {
				error!("[CommandProvider] Command '{}' not found in registry.", CommandIdentifier);

				Err(CommonError::CommandNotFound { Identifier:CommandIdentifier })
			},
		}
	}

	/// Registers a command contributed by a sidecar process.
	async fn RegisterCommand(&self, SideCarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		info!(
			"[CommandProvider] Registering PROXY command '{}' from sidecar '{}'",
			CommandIdentifier, SideCarIdentifier
		);

		let mut Registry = self
			.ApplicationState
			.CommandRegistry
			.lock()
			.map_err(super::Utility::MapApplicationStateLockErrorToCommonError)?;

		Registry.insert(
			CommandIdentifier.clone(),
			CommandHandler::Proxied { SideCarIdentifier, CommandIdentifier },
		);

		Ok(())
	}

	/// Unregisters a previously registered command.
	async fn UnregisterCommand(&self, _SideCarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		info!("[CommandProvider] Unregistering command '{}'", CommandIdentifier);

		self.ApplicationState
			.CommandRegistry
			.lock()
			.map_err(super::Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&CommandIdentifier);

		Ok(())
	}

	/// Gets a list of all currently registered command IDs.
	async fn GetAllCommands(&self) -> Result<Vec<String>, CommonError> {
		debug!("[CommandProvider] Getting all command identifiers.");

		let Registry = self
			.ApplicationState
			.CommandRegistry
			.lock()
			.map_err(super::Utility::MapApplicationStateLockErrorToCommonError)?;

		Ok(Registry.keys().cloned().collect())
	}
}
