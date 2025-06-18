// @module ApplicationRunTime
// @description Defines the concrete, Echo-based `ApplicationRunTime` for the
// Mountain application.
//
// This is the core execution engine that bridges the declarative
// `ActionEffect` system with the high-performance `Echo` task scheduler.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	effect::{ActionEffect, ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::{Environment, Requires},
	error::CommonError,
};
use Echo::scheduler::Scheduler;
use log::{error, info};
use tokio::sync::oneshot;

use crate::Environment::MountainEnvironment;

// A RunTime that uses a high-performance, work-stealing scheduler (`Echo`)
// to execute all `ActionEffect`s. This struct is managed by Tauri and is
// cloneable so it can be passed into different contexts.
pub struct ApplicationRunTime {
	// A shared handle to the application's central scheduler.
	pub Scheduler:Arc<Scheduler>,
	// A shared handle to the application's Environment, providing
	// capabilities.
	pub Environment:Arc<MountainEnvironment>,
}

impl ApplicationRunTime {
	// Creates a new `ApplicationRunTime` that is powered by an `Echo`
	// scheduler.
	//
	// # Arguments
	// * `scheduler` - A shared pointer to the application's main scheduler.
	// * `Environment` - A shared pointer to the application's Environment.
	pub fn New(scheduler:Arc<Scheduler>, Environment:Arc<MountainEnvironment>) -> Self {
		info!("[ApplicationRunTime] New Echo-based instance created.");
		Self { Scheduler:scheduler, Environment:Environment }
	}
}

#[async_trait]
impl ApplicationRunTimeTrait for ApplicationRunTime {
	type EnvironmentType = MountainEnvironment;

	// Gets the Environment associated with this RunTime.
	fn GetEnvironment(&self) -> Arc<Self::EnvironmentType> { self.Environment.clone() }

	// The core integration logic between `Common::ActionEffect` and
	// `Echo::Scheduler`.
	//
	// This method takes an `ActionEffect`, wraps its execution in a new
	// future, submits that future to the scheduler, and then awaits the result
	// via a `oneshot` channel. This decouples the request of an effect from
	// its execution on a worker thread, enabling true concurrent processing.
	async fn Run<Capability, Error, Output>(
		&self,
		effect:ActionEffect<Arc<Capability>, Error, Output>,
	) -> Result<Output, Error>
	where
		Capability: ?Sized + Send + Sync,
		Self::EnvironmentType: Requires<Arc<Capability>>,
		Error: From<CommonError> + Send + Sync + 'static, // Ensure we can represent a RecvError
		Output: Send + Sync + 'static, {
		// 1. Create the single-use channel to receive the result from the worker
		//    thread.
		let (result_sender, result_receiver) = oneshot::channel::<Result<Output, Error>>();

		// 2. Get the specific capability (e.g., `Arc<dyn FileSystemReader>`) the effect
		//    needs from the Environment.
		let capability_provider:Arc<Capability> = self.Environment.Require();

		// 3. Create the future that will be executed by a worker thread.
		// This future captures the Effect, its required Capability, and the Sender.
		let task = async move {
			let result = effect.Apply(capability_provider).await;
			if result_sender.send(result).is_err() {
				// This occurs if the caller stops awaiting the receiver, e.g., due to a
				// timeout. The task has already completed, so we just log the failure to
				// communicate the result.
				error!("[ApplicationRunTime] Failed to send effect result; receiver was dropped.");
			}
		};

		// 4. Submit the raw future to the scheduler with normal priority.
		self.Scheduler.Submit(task, Echo::task::Priority::Normal);

		// 5. Await the result from the oneshot channel.
		match result_receiver.await {
			Ok(result) => result,
			Err(recv_error) => {
				let message = format!("Effect execution canceled; oneshot channel closed. Error: {}", recv_error);
				error!("{}", message);
				// Convert the channel receive error into the effect's error type.
				Err(CommonError::IpcError { Description:message }.into())
			},
		}
	}
}

impl Clone for ApplicationRunTime {
	// Implements `Clone` to satisfy Tauri's `State` management requirements.
	fn clone(&self) -> Self { Self { Scheduler:self.Scheduler.clone(), Environment:self.Environment.clone() } }
}
