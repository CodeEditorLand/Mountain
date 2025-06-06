// File: AppRuntime.rs
// Primary Focus: Defines the application's runtime environment and execution
// logic.

use std::sync::Arc;

use Common::Runtime::AppRuntimeTrait; /* Assuming AppRuntimeTrait is the PascalCase version of
                                             * CommonRuntimeTrait */
use Common::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
};
use async_trait::async_trait;
use log::{info, trace};

// Assuming MountainEnvironment is defined in the crate's Environment module
use crate::Environment::MountainEnvironment;

/// DefaultRuntime provides a basic, reusable runtime implementation.
/// It holds an environment and can execute effects that require capabilities
/// from that environment.
pub struct DefaultRuntime<ContextEnvironmentType:Environment> {
	EnvironmentInstance:Arc<ContextEnvironmentType>,
}

impl<ContextEnvironmentType:Environment> DefaultRuntime<ContextEnvironmentType> {
	/// Creates a new DefaultRuntime with the given environment.
	pub fn New(EnvironmentInstance:Arc<ContextEnvironmentType>) -> Self {
		trace!("[DefaultRuntime] New instance created.");
		Self { EnvironmentInstance }
	}

	/// Gets a clone of the Arc-wrapped environment instance.
	pub fn GetEnvironmentArc(&self) -> Arc<ContextEnvironmentType> { Arc::clone(&self.EnvironmentInstance) }
}

#[async_trait]
impl<ContextEnvironmentType:Environment + Send + Sync + 'static> AppRuntimeTrait<ContextEnvironmentType>
	for DefaultRuntime<ContextEnvironmentType>
{
	/// Gets the environment associated with this runtime.
	fn GetEnvironment(&self) -> Arc<ContextEnvironmentType> { Arc::clone(&self.EnvironmentInstance) }

	/// Runs an ActionEffect using the environment provided by this runtime.
	/// The ContextEnvironmentType must be able to provide the AccessorType
	/// required by the effect.
	async fn Run<AccessorType, ErrorType, OutputType>(
		&self,
		Effect:ActionEffect<AccessorType, ErrorType, OutputType>,
	) -> Result<OutputType, ErrorType>
	where
		AccessorType: Environment + Send + Sync + 'static,
		ContextEnvironmentType: Requires<AccessorType>,
		ErrorType: Send + Sync + 'static,
		OutputType: Send + Sync + 'static, {
		trace!("[DefaultRuntime] Running effect...");
		let AccessorInstance:AccessorType = self.EnvironmentInstance.require();
		Effect.Apply(AccessorInstance).await
	}
}

/// AppRuntime is the specific runtime for the Mountain application.
/// It utilizes a DefaultRuntime configured with MountainEnvironment.
pub struct AppRuntime {
	InnerRuntime:DefaultRuntime<MountainEnvironment>,
}

impl AppRuntime {
	/// Creates a new AppRuntime instance, taking an Arc-wrapped
	/// MountainEnvironment.
	pub fn New(EnvironmentInstance:Arc<MountainEnvironment>) -> Self {
		info!("[AppRuntime] New instance created.");
		Self { InnerRuntime:DefaultRuntime::New(EnvironmentInstance) }
	}

	/// Runs an ActionEffect.
	/// This is a convenience method specific to AppRuntime that constrains
	/// the effect's required environment (EffectEnvironmentType) to what
	/// MountainEnvironment can provide.
	pub async fn Run<EffectEnvironmentType, ErrorType, OutputType>(
		&self,
		Effect:ActionEffect<EffectEnvironmentType, ErrorType, OutputType>,
	) -> Result<OutputType, ErrorType>
	where
		EffectEnvironmentType: Environment + Send + Sync + 'static,
		ErrorType: Send + Sync + 'static,
		OutputType: Send + Sync + 'static,
		MountainEnvironment: Requires<EffectEnvironmentType>, {
		trace!("[AppRuntime] Delegating effect execution to InnerRuntime.");
		// This directly calls the `Run` method defined on `DefaultRuntime` via its
		// `AppRuntimeTrait` impl, as `AppRuntime::Run` has the same signature
		// constraints.
		self.InnerRuntime.Run(Effect).await
	}

	/// Gets the MountainEnvironment associated with this application runtime.
	pub fn GetEnvironment(&self) -> Arc<MountainEnvironment> { self.InnerRuntime.GetEnvironmentArc() }
}

#[async_trait]
impl AppRuntimeTrait<MountainEnvironment> for AppRuntime {
	/// Gets the MountainEnvironment.
	fn GetEnvironment(&self) -> Arc<MountainEnvironment> { self.InnerRuntime.GetEnvironmentArc() }

	/// Runs an ActionEffect using the MountainEnvironment.
	/// This implementation fulfills the AppRuntimeTrait for AppRuntime.
	async fn Run<AccessorType, ErrorType, OutputType>(
		&self,
		Effect:ActionEffect<AccessorType, ErrorType, OutputType>,
	) -> Result<OutputType, ErrorType>
	where
		AccessorType: Environment + Send + Sync + 'static,
		MountainEnvironment: Requires<AccessorType>, // Trait constraint from AppRuntimeTrait
		ErrorType: Send + Sync + 'static,
		OutputType: Send + Sync + 'static, {
		trace!("[AppRuntime Trait Impl] Delegating effect execution to InnerRuntime.");
		self.InnerRuntime.Run(Effect).await
	}
}
