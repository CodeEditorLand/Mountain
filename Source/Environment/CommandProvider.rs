//! # CommandProvider (Environment)
//!
//! Implements the `CommandExecutor` trait, serving as the central registry and
//! dispatcher for all commands in the Mountain application. Commands can be
//! handled either by native Rust handlers or proxied to extension sidecar
//! processes.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Command Registry
//! - Maintain centralized registry of all registered commands
//! - Store command metadata (id, title, category, flags)
//! - Track command source (native vs extension)
//! - Support command enablement/disable state
//!
//! ### 2. Command Dispatching
//! - Route command execution requests to appropriate handlers
//! - Execute native commands directly via function calls
//! - Proxy extension commands via IPC to sidecar processes
//! - Handle command result propagation and error translation
//!
//! ### 3. Command Execution Context
//! - Provide `CommandExecutor` capability to command handlers
//! - Manage command scope (text editor, file system, etc.)
//! - Track command invocation source and user context
//! - Support command cancellation for long-running operations
//!
//! ### 4. Command Discovery
//! - Enumerate all registered commands for UI display
//! - Support command palette and quick open
//! - Provide command categories and visibility rules
//! - Handle command contribution points from extensions
//!
//! ## ARCHITECTURAL ROLE
//!
//! CommandProvider is the **command execution hub** for Mountain:
//!
//! ```text
//! Command Caller ──► CommandProvider ──► Handler (Native or Extension)
//!       │                            │
//!       └─► ExecuteCommand() ───────► Execute
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: Core capability provider
//! - Implements `CommonLibrary::Command::CommandExecutor` trait
//! - Accessible via `Environment.Require<dyn CommandExecutor>()`
//!
//! ### Command Sources
//! - **Native Commands**: Defined in Rust, registered at startup
//! - **Extension Commands**: Defined in package.json, contributed by extensions
//! - **Built-in Commands**: Core Mountain functionality
//! - **User-defined Commands**: Custom macros and keybindings (future)
//!
//! ### Dependencies
//! - `ApplicationState`: Command registry storage
//! - `IPCProvider`: For proxying to extension sidecars
//! - `Log`: For command execution tracing
//!
//! ### Dependents
//! - All command handlers: Use `CommandExecutor` to execute other commands
//! - `DispatchLogic`::`DispatchFrontendCommand`: Main entry point
//! - Tauri command handlers: Many invoke `ExecuteCommand`
//! - Keybinding system: Trigger commands via keyboard shortcuts
//!
//! ## COMMAND EXECUTION FLOW
//!
//! 1. **Request**: Caller invokes `ExecuteCommand(command_id, args)`
//! 2. **Lookup**: Provider looks up command in
//!    `ApplicationState::CommandRegistry`
//! 3. **Handler**: Retrieves the associated handler (native function or
//!    extension RPC)
//! 4. **Execute**: Calls handler with arguments
//! 5. **Result**: Returns serialized JSON result or error
//!
//! ## NATIVE vs EXTENSION COMMANDS
//!
//! ### Native Commands
//! - Implemented directly in Rust
//! - Registered via `RegisterCommand` function
//! - Handler is a function pointer or Arc<Fn>
//! - Zero IPC overhead, direct call
//!
//! ### Extension Commands
//! - Defined in extension's `package.json` `contributes.commands`
//! - Registered when extension is activated
//! - Handler is RPC method to extension sidecar
//! - Goes through IPC layer with serialization
//!
//! ## ERROR HANDLING
//!
//! - Command not found: Returns `CommonError::InvalidArgument`
//! - Handler errors: Propagated as `CommonError`
//! - IPC failures: Converted to `CommonError::IPCError`
//! - Serialization failures: `CommonError::SerializationError`
//!
//! ## PERFORMANCE
//!
//! - Native commands: Near-zero overhead (direct function call)
//! - Extension commands: IPC serialization + network latency
//! - Command lookup: HashMap lookup by string ID (fast)
//! - Consider caching frequently used command results
//!
//! ## VS CODE REFERENCE
//!
//! Borrowed from VS Code's command system:
//! - `vs/platform/commands/common/commands.ts` - Command definitions
//! - `vs/workbench/services/commands/common/commandService.ts` - Command
//!   registry
//! - `vs/platform/commands/common/commandExecutor.ts` - Command execution
//!
//! ## TODO
//!
//! - [ ] Implement command contribution points from extensions
//! - [ ] Add command enablement/disable state management
//! - [ ] Support command categories and grouping
//! - [ ] Add command history and undo/redo stack
//! - [ ] Implement command keyboard shortcut resolution
//! - [ ] Add command telemetry (usage metrics)
//! - [ ] Support command aliases and deprecation
//! - [ ] Add command permission validation
//! - [ ] Implement command batching for related operations
//!
//! ## MODULE CONTENTS
//!
//! - [`CommandProvider`]: Main struct implementing `CommandExecutor`
//! - Command registration functions (to be added)
//! - Extension command proxy logic

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
