// File: Common/Runtime.rs
// Defines the core traits for an application runtime, specifying the contract
// for executing effects and providing access to the environment.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
};

/// `AppRuntimeTrait` defines the essential capabilities of any application
/// runtime: providing an environment and executing effects within that
/// environment's context.
#[async_trait]
pub trait AppRuntimeTrait<ContextEnvironmentType:Environment>: Send + Sync {
	/// Returns the environment associated with the runtime.
	fn GetEnvironment(&self) -> Arc<ContextEnvironmentType>;

	/// Executes an `ActionEffect` that requires an `AccessorType`.
	/// The runtime's `ContextEnvironmentType` must be able to provide the
	/// required `AccessorType`.
	async fn Run<AccessorType, ErrorType, OutputType>(
		&self,
		Effect:ActionEffect<AccessorType, ErrorType, OutputType>,
	) -> Result<OutputType, ErrorType>
	where
		AccessorType: Environment + Send + Sync + 'static,
		ContextEnvironmentType: Requires<AccessorType>,
		ErrorType: Send + Sync + 'static,
		OutputType: Send + Sync + 'static;
}

/// `DefaultRuntime` provides a basic, reusable runtime implementation.
/// It holds an environment and can execute effects that require capabilities
/// from that environment.
pub struct DefaultRuntime<ContextEnvironmentType:Environment> {
	EnvironmentInstance:Arc<ContextEnvironmentType>,
}

impl<ContextEnvironmentType:Environment> DefaultRuntime<ContextEnvironmentType> {
	/// Creates a new `DefaultRuntime` with the given environment.
	pub fn New(EnvironmentInstance:Arc<ContextEnvironmentType>) -> Self { Self { EnvironmentInstance } }

	/// Gets a clone of the `Arc`-wrapped environment instance.
	pub fn GetEnvironmentArc(&self) -> Arc<ContextEnvironmentType> { Arc::clone(&self.EnvironmentInstance) }
}
