//! Implements the `CommandExecutor` trait for the `MountainEnvironment`,
//! providing the concrete logic for command registration and execution.

use std::sync::Arc;

use Common::{command::CommandExecutor, environment::Requires, error::CommonError};
use async_trait::async_trait;
use serde_json::Value;

use super::MountainEnvironment;
use crate::handlers::commands as CommandHandler;

#[async_trait]
impl CommandExecutor for MountainEnvironment {
	/// Executes a registered command, delegating to the `ExecuteCommandLogic`
	/// handler.
	async fn ExecuteCommand(&self, CommandIdentifier:String, Argument:Value) -> Result<Value, CommonError> {
		CommandHandler::ExecuteCommandLogic(&self.AppHandle, CommandIdentifier, Argument).await
	}

	/// Registers a command from a sidecar, delegating to the
	/// `RegisterCommandLogic` handler.
	async fn RegisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		CommandHandler::RegisterCommandLogic(&self.AppHandle, SidecarIdentifier, CommandIdentifier).await
	}

	/// Unregisters a command from a sidecar, delegating to the
	/// `UnregisterCommandLogic` handler.
	async fn UnregisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		CommandHandler::UnregisterCommandLogic(&self.AppHandle, SidecarIdentifier, CommandIdentifier).await
	}

	/// Retrieves a list of all registered command IDs, delegating to the
	/// `GetAllCommandsLogic` handler.
	async fn GetAllCommands(&self) -> Result<Vec<String>, CommonError> {
		CommandHandler::GetAllCommandsLogic(&self.AppHandle).await
	}
}

impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	/// Fulfills the dependency injection requirement for `CommandExecutor`.
	fn Require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}
