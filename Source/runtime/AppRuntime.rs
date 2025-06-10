//! Defines the concrete, Echo-based `AppRuntime` for the Mountain application.
//!
//! This is the core execution engine that bridges the declarative
//! `ActionEffect` system with the high-performance `Echo` task scheduler.

use std::sync::Arc;

use Common::{
	effect::{ActionEffect, AppRuntime as AppRuntimeTrait},
	environment::{Environment, Requires},
	error::CommonError, // Assuming this will be used for error mapping.
};
use async_trait::async_trait;
use log::{error, info};
use tokio::sync::oneshot;

use crate::{environment::MountainEnvironment, scheduler::Scheduler};

/// A runtime that uses a high-performance, work-stealing scheduler (`Echo`)
/// to execute all `ActionEffect`s.
pub struct AppRuntime {
	Scheduler:Arc<Scheduler>,
	Environment:Arc<MountainEnvironment>,
}

impl AppRuntime {
	/// Creates a new `AppRuntime` that is powered by an `Echo` scheduler.
	///
	/// # Arguments
	/// * `Scheduler` - A shared pointer to the application's main scheduler.
	/// * `Environment` - A shared pointer to the application's environment.
	pub fn New(Scheduler:Arc<Scheduler>, Environment:Arc<MountainEnvironment>) -> Self {
		info!("[AppRuntime] New Echo-based instance created.");
		Self { Scheduler, Environment }
	}
}

#[async_trait]
impl AppRuntimeTrait for AppRuntime {
	type EnvironmentType = MountainEnvironment;

	/// Gets the environment associated with this runtime.
	fn GetEnvironment(&self) -> Arc<Self::EnvironmentType> { self.Environment.clone() }

	/// The core integration logic.
	///
	/// This method takes an `ActionEffect`, wraps its execution in a new
	/// future, submits that future to the scheduler, and then awaits the result
	/// via a `oneshot` channel. This decouples the request of an effect from
	/// its execution on a worker thread.
	async fn Run<Capability, Error, Output>(
		&self,
		Effect:ActionEffect<Arc<Capability>, Error, Output>,
	) -> Result<Output, Error>
	where
		Capability: ?Sized + Send + Sync,
		Self::EnvironmentType: Requires<Arc<Capability>>,
		Error: Send + Sync + 'static,
		Output: Send + Sync + 'static, {
		// 1. Create the single-use channel to receive the result.
		let (ResultSender, ResultReceiver) = oneshot::channel::<Result<Output, Error>>();

		// 2. Get the specific capability (e.g., `Arc<dyn FsReader>`) the effect needs.
		let CapabilityProvider:Arc<Capability> = self.Environment.Require();

		// 3. Create the future that will be executed by a worker thread.
		// This future captures the Effect, its required Capability, and the Sender.
		let Task = async move {
			let Result = Effect.Apply(CapabilityProvider).await;
			if ResultSender.send(Result).is_err() {
				// This occurs if the caller stops awaiting the receiver, e.g., due to a
				// timeout.
				error!("[AppRuntime] Failed to send effect result; receiver was dropped.");
			}
		};

		// 4. Submit the raw future to the scheduler with normal priority.
		self.Scheduler.Submit(Task, Echo::task::Priority::Normal);

		// 5. Await the result from the oneshot channel.
		match ResultReceiver.await {
			Ok(Result) => Result,
			Err(RecvError) => {
				let Message = format!("Effect execution canceled; oneshot channel closed. Error: {}", RecvError);
				error!("{}", Message);
				// TODO: Map this RecvError to a specific CommonError variant instead of
				// panicking.
				panic!("{}", Message);
			},
		}
	}
}

impl Clone for AppRuntime {
	/// Implements `Clone` to satisfy Tauri's `State` management requirements.
	fn clone(&self) -> Self { Self { Scheduler:self.Scheduler.clone(), Environment:self.Environment.clone() } }
}
