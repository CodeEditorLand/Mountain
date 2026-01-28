// File: Mountain/Source/Environment/CommandProvider.rs
// Role: Implements the `CommandExecutor` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Contains the core logic for managing the command registry.
//   - Dispatches command execution to either native Rust handlers or proxied
//     sidecar handlers.

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # CommandProvider Implementation
//!
//! Implements the `CommandExecutor` trait for the `MountainEnvironment`. This
//! provider contains the core logic for managing the command registry and
//! dispatching command execution to either native Rust handlers or proxied
//! sidecar handlers.

#![allow(non_snake_case, non_camel_case_types)]

use std::{future::Future, pin::Pin, sync::Arc};

use Common::{
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
