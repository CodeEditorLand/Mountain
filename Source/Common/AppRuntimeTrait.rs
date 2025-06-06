// File: Common/AppRuntimeTrait.rs
// Defines the core trait for an application runtime, specifying the contract
// for executing effects within a given environment.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
};

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
