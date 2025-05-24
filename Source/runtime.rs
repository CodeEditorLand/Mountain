// ---------------------------------------------------------------------------------------------
// Mountain Application Runtime (runtime.rs)
// --------------------------------------------------------------------------------------------
// Defines the `AppRuntime` which is responsible for executing `ActionEffect`s.
// It wraps the `DefaultRuntime` provided by `Land_Common`, injecting the
// specific `MountainEnvironment` which provides the concrete implementations
// for effects. This runtime instance is managed by Tauri and passed to
// command/request dispatchers.
//
// Responsibilities:
// - Holding an instance of the `MountainEnvironment`.
// - Providing the core `run` method that takes an `ActionEffect` and executes
//   it by calling the appropriate method on the `MountainEnvironment`.
// - Handling the execution context (e.g., potentially spawning tasks via
//   tokio).
//
// Key Interactions:
// - Instantiated in `main.rs` with a `MountainEnvironment`.
// - Managed by Tauri via `State<'_, Arc<AppRuntime>>`.
// - Its `run` method is called by `track::dispatch_command` and
//   `track::dispatch_sidecar_request`.
// - Delegates actual effect implementation to the `MountainEnvironment`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::runtime::{DefaultRuntime, Runtime}; // Assuming DefaultRuntime in Common

use crate::environment::MountainEnvironment;

// Wrapper struct if needed, or just use
// Arc<DefaultRuntime<MountainEnvironment>> directly
pub struct AppRuntime {
	inner:DefaultRuntime<MountainEnvironment>,
}

impl AppRuntime {
	pub fn new(env:Arc<MountainEnvironment>) -> Self {
		// Initialize the default runtime from Land/Common, providing the
		// Mountain-specific environment.
		// This runtime might internally use tokio::spawn or your Echo queue.
		Self { inner:DefaultRuntime::new(env) }
	}

	// Expose the run method
	pub async fn run<E, Err, Out>(&self, effect:ActionEffect<E, Err, Out>) -> Result<Out, Err>
	where
		E: Environment + Send + Sync + 'static, // Ensure Env constraints match Effect
		Err: Send + Sync + 'static,
		Out: Send + Sync + 'static,
		MountainEnvironment: Requires<E>, // MountainEnv must provide what Effect needs
	{
		// Delegate to the common runtime's run method
		self.inner.run(effect).await
	}
}
