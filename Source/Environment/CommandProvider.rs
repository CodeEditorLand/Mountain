// @module CommandProvider (Environment)
// @description Implements the `CommandExecutor` trait for the
// `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{command::CommandExecutor, Environment::Requires, error::CommonError};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::command as CommandHandler;

#[async_trait]
impl CommandExecutor for MountainEnvironment {
	// Executes a registered command, delegating to the `ExecuteCommandLogic`
	// handler.
	async fn ExecuteCommand(&self, command_identifier:String, argument:Value) -> Result<Value, CommonError> {
		CommandHandler::ExecuteCommandLogic(&self.ApplicationHandle, command_identifier, argument).await
	}

	// Registers a command from a sidecar, delegating to the
	// `RegisterCommandLogic` handler.
	async fn RegisterCommand(&self, sidecar_identifier:String, command_identifier:String) -> Result<(), CommonError> {
		CommandHandler::RegisterCommandLogic(&self.ApplicationHandle, sidecar_identifier, command_identifier).await
	}

	// Unregisters a command from a sidecar, delegating to the
	// `UnregisterCommandLogic` handler.
	async fn UnregisterCommand(&self, sidecar_identifier:String, command_identifier:String) -> Result<(), CommonError> {
		CommandHandler::UnregisterCommandLogic(&self.ApplicationHandle, sidecar_identifier, command_identifier).await
	}

	// Retrieves a list of all registered command IDs, delegating to the
	// `GetAllCommandLogic` handler.
	async fn GetAllCommand(&self) -> Result<Vec<String>, CommonError> {
		CommandHandler::GetAllCommandLogic(&self.ApplicationHandle).await
	}
}

impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	// Fulfills the dependency injection requirement for `CommandExecutor`.
	fn Require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}
