// ---------------------------------------------------------------------------------------------
// Mountain Application Runtime (runtime.rs)
// --------------------------------------------------------------------------------------------
// Defines the `AppRuntime` which is responsible for executing `ActionEffect`s
// within the Mountain application. It serves as a specialized wrapper around
// the generic `DefaultRuntime` provided by `Land_Common`, injecting the
// specific `MountainEnvironment`. The `MountainEnvironment` provides the
// concrete, native implementations for all effects defined in
// `Land_Common::effects`.
//
// This `AppRuntime` instance is a critical component, managed by Tauri as
// shared state, and is passed to command/request dispatchers (`track.rs`,
// `rpc.rs`) to enable them to run effects.
//
// Responsibilities:
// - Holding an `Arc` (atomic reference counted pointer) to an instance of
//   `MountainEnvironment`. This allows the runtime to access the concrete
//   effect implementations.
// - Providing the core `run` method. This method:
//   - Takes an `ActionEffect<E, Err, Out>` as input, where `E` is the required
//     environment trait (e.g., `FsReader`, `ConfigProvider`), `Err` is the
//     error type (usually `CommonError`), and `Out` is the output type of the
//     effect.
//   - Executes the effect by calling the appropriate method on the underlying
//     `MountainEnvironment` (which implements the required trait `E` via the
//     `Requires<E>` mechanism).
// - Leveraging the `DefaultRuntime` from `Land_Common` for the actual execution
//   logic, which might involve spawning tasks via Tokio or other concurrency
//   primitives depending on its implementation.
//
// Key Interactions:
// - Instantiated in `main.rs` during application setup, where it's provided
//   with an `Arc<MountainEnvironment>`.
// - Managed by Tauri as shared state, typically accessed as `State<'_,
//   Arc<AppRuntime>>`.
// - Its `run` method is the primary way `ActionEffect`s are executed. This
//   method is called by:
//   - `track::dispatch_command` (for effects originating from frontend
//     commands).
//   - `track::dispatch_sidecar_request` (for effects originating from sidecar
//     RPCs).
//   - RPC handler methods in `rpc.rs` if they need to run effects (e.g., for UI
//     interactions).
// - Delegates the actual implementation of effect logic to the
//   `MountainEnvironment` instance it holds.
// --------------------------------------------------------------------------------------------

// For Arc<MountainEnvironment>
use std::sync::Arc;

// Import DefaultRuntime and the core Runtime trait from Land_Common.
use Land_Common::{
	// For trait bounds on `run`
	environment::{Environment, Requires},

	// Renamed to avoid conflict
	runtime::{ActionEffect, DefaultRuntime, Runtime as CommonRuntimeTrait},
};
// For logging runtime creation
use log::info;

// The concrete environment for Mountain
use crate::environment::MountainEnvironment;

/// The application runtime for Mountain, responsible for executing
/// `ActionEffect`s.
///
/// It wraps a `DefaultRuntime` from `Land_Common`, configured with Mountain's
/// specific `MountainEnvironment`.
pub struct AppRuntime {
	// The inner runtime from Land_Common that handles the core effect execution logic.
	inner_runtime:DefaultRuntime<MountainEnvironment>,
}

impl AppRuntime {
	/// Creates a new `AppRuntime`.
	///
	/// # Argument
	/// * `environment` - An `Arc<MountainEnvironment>` that provides the
	///   concrete implementations for all effects.
	///
	/// # Returns
	/// A new `AppRuntime` instance.
	pub fn new(environment:Arc<MountainEnvironment>) -> Self {
		info!("[AppRuntime Init] Creating new AppRuntime instance.");

		// Initialize the `DefaultRuntime` from `Land_Common`, providing it with
		// the Mountain-specific environment. This `DefaultRuntime` might internally
		// use mechanisms like `tokio::spawn` or a custom execution queue for effects.
		Self { inner_runtime:DefaultRuntime::new(environment) }
	}

	/// Executes an `ActionEffect`.
	///
	/// This method takes an `ActionEffect` and runs it using the configured
	/// `MountainEnvironment`. The `ActionEffect` itself defines what
	/// environment trait (`E`) it requires. The `MountainEnvironment` must
	/// implement `Requires<E>` for the effect to be runnable.
	///
	/// # Type Parameters
	/// * `E`: The specific environment trait (e.g., `FsReader`,
	///
	///   `ConfigProvider`) that the `effect` requires. Must implement
	///   `Environment + Send + Sync + 'static`.
	/// * `Err`: The error type that the `effect` can return (typically
	///   `CommonError`). Must be `Send + Sync + 'static`.
	/// * `Out`: The output type that the `effect` produces on success. Must be
	///   `Send + Sync + 'static`.
	///
	/// # Argument
	/// * `effect` - The `ActionEffect` to execute.
	///
	/// # Returns
	/// * `Result<Out, Err>`: The outcome of the effect's execution.
	pub async fn run<E, Err, Out>(&self, effect:ActionEffect<E, Err, Out>) -> Result<Out, Err>
	where
		// Effect's required environment trait
		E: Environment + Send + Sync + 'static,
		// Effect's error type
		Err: Send + Sync + 'static,
		// Effect's output type
		Out: Send + Sync + 'static,
		// Constraint: MountainEnvironment must be able to provide the required trait `E`.
		MountainEnvironment: Requires<E>, {
		// Delegate the execution to the `run` method of the inner `DefaultRuntime`.
		// The `DefaultRuntime` will use its `MountainEnvironment` instance to satisfy
		// the effect's `Requires<E>` constraint and call the appropriate trait method.
		self.inner_runtime.run(effect).await
	}

	// Expose a way to get the underlying environment if direct access is needed
	// by components that already have the AppRuntime. This is used by RPC handlers
	// and `track.rs` to get FsReader/FsWriter etc.
	// Renamed from `get_environment` in a previous iteration for clarity, but
	// `get_environment` is fine. Sticking with `get_environment` as it's more
	// descriptive of what it returns.
	pub fn get_environment(&self) -> Arc<MountainEnvironment> { self.inner_runtime.get_environment() }
}

// Implement the common Runtime trait for AppRuntime if DefaultRuntime also
// does. This allows AppRuntime to be used more generically if needed.
// This assumes `DefaultRuntime<T>` implements `CommonRuntimeTrait<T>`.
impl CommonRuntimeTrait<MountainEnvironment> for AppRuntime {
	fn get_environment(&self) -> Arc<MountainEnvironment> { self.inner_runtime.get_environment() }
}
