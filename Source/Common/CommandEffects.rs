
// Defines the CommandExecutor trait and associated effects for command
// management. This provides a standardized way to execute, register, and query
// commands within the application's environment.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
}; // For the bound on the effect's runtime accessor

/// A trait for environments that can execute commands.
#[async_trait]
pub trait CommandExecutor: Environment {
	/// Executes a command by its identifier with the given arguments.
	async fn ExecuteCommand(&self, CommandIdentifier:String, Argument:Value) -> Result<Value, CommonError>;
	/// Registers a command from a sidecar process.
	async fn RegisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError>;
	/// Unregisters a command from a sidecar process.
	async fn UnregisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError>;
	/// Retrieves a list of all currently registered command identifiers.
	async fn GetAllCommands(&self) -> Result<Vec<String>, CommonError>;
}

/// Creates an effect to execute a command.
pub fn ExecuteCommand<RuntimeAccessType>(
	CommandIdentifier:String,
	Argument:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Value>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn CommandExecutor>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let CommandIdentifierClone = CommandIdentifier.clone();
		let ArgumentClone = Argument.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Executor:Arc<dyn CommandExecutor> = Environment.require();
			Executor.ExecuteCommand(CommandIdentifierClone, ArgumentClone).await
		})
	}))
}

/// Creates an effect to register a command.
pub fn RegisterCommand<RuntimeAccessType>(
	SidecarIdentifier:String,
	CommandIdentifier:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn CommandExecutor>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let SidecarIdentifierClone = SidecarIdentifier.clone();
		let CommandIdentifierClone = CommandIdentifier.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Executor:Arc<dyn CommandExecutor> = Environment.require();
			Executor.RegisterCommand(SidecarIdentifierClone, CommandIdentifierClone).await
		})
	}))
}

/// Creates an effect to unregister a command.
pub fn UnregisterCommand<RuntimeAccessType>(
	SidecarIdentifier:String,
	CommandIdentifier:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn CommandExecutor>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let SidecarIdentifierClone = SidecarIdentifier.clone();
		let CommandIdentifierClone = CommandIdentifier.clone();
		Box::pin(async move {
			let Environment = Accessor.get_environment();
			let Executor:Arc<dyn CommandExecutor> = Environment.require();
			Executor.UnregisterCommand(SidecarIdentifierClone, CommandIdentifierClone).await
		})
	}))
}

/// Creates an effect to get a list of all registered commands.
pub fn GetAllCommands<RuntimeAccessType>() -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Vec<String>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn CommandExecutor>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Executor:Arc<dyn CommandExecutor> = Environment.require();
			Executor.GetAllCommands().await
		})
	}))
}
