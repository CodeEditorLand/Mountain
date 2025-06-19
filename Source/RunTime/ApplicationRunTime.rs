//! # ApplicationRunTime
//!
//! Defines the concrete, `Echo`-based `ApplicationRunTime` for the Mountain
//! application. This is the core execution engine that bridges the declarative
//! `ActionEffect` system with the high-performance `Echo` task scheduler.

use std::sync::Arc;

use Common::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
};
use Echo::Scheduler::Scheduler::Scheduler;
use async_trait::async_trait;
use log::{error, info};
use tokio::sync::oneshot;

use crate::Environment::MountainEnvironment::MountainEnvironment;

/// A `RunTime` that uses a high-performance, work-stealing scheduler (`Echo`)
/// to execute all `ActionEffect`s. This struct is managed by Tauri and is
/// cloneable so it can be passed into different contexts.
pub struct ApplicationRunTime {
	/// A shared handle to the application's central scheduler.
	pub Scheduler:Arc<Scheduler>,
	/// A shared handle to the application's `Environment`, providing all
	/// necessary capabilities.
	pub Environment:Arc<MountainEnvironment>,
}

impl ApplicationRunTime {
	/// Creates a new `ApplicationRunTime` that is powered by an `Echo`
	/// scheduler.
	///
	/// # Arguments
	/// * `Scheduler`: A shared pointer to the application's main scheduler.
	/// * `Environment`: A shared pointer to the application's `Environment`.
	pub fn Create(Scheduler:Arc<Scheduler>, Environment:Arc<MountainEnvironment>) -> Self {
		info!("[ApplicationRunTime] New Echo-based instance created.");
		Self { Scheduler, Environment }
	}
}

#[async_trait]
impl ApplicationRunTimeTrait for ApplicationRunTime {
	type EnvironmentType = MountainEnvironment;

	/// Gets the `Environment` associated with this `RunTime`.
	fn GetEnvironment(&self) -> Arc<Self::EnvironmentType> { self.Environment.clone() }

	/// The core integration logic between `Common::ActionEffect` and
	/// `Echo::Scheduler`.
	///
	/// This method takes an `ActionEffect`, wraps its execution in a new
	/// future, submits that future to the scheduler, and then awaits the result
	/// via a `oneshot` channel. This decouples the *request* of an effect from
	/// its *execution* on a worker thread, enabling true concurrent processing.
	async fn Run<TCapability, TError, TOutput>(
		&self,
		Effect:ActionEffect<Arc<TCapability>, TError, TOutput>,
	) -> Result<TOutput, TError>
	where
		TCapability: ?Sized + Send + Sync,
		Self::EnvironmentType: Requires<Arc<TCapability>>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static, {
		// 1. Create the single-use channel to receive the result from the worker.
		let (ResultSender, ResultReceiver) = oneshot::channel::<Result<TOutput, TError>>();

		// 2. Get the specific capability the effect needs from the Environment.
		let CapabilityProvider:Arc<TCapability> = self.Environment.Require();

		// 3. Create the future that will be executed by a worker thread.
		// This future captures the Effect, its Capability, and the Sender.
		let Task = async move {
			let Result = Effect.Apply(CapabilityProvider).await;
			if ResultSender.send(Result).is_err() {
				// This occurs if the caller stops awaiting the receiver, e.g., due to a
				// timeout. The task has already completed, so we just log the failure.
				error!("[ApplicationRunTime] Failed to send effect result; receiver was dropped.");
			}
		};

		// 4. Submit the raw future to the scheduler with normal priority.
		self.Scheduler.Submit(Task, Echo::Task::Priority::Normal);

		// 5. Await the result from the oneshot channel.
		match ResultReceiver.await {
			Ok(Result) => Result,
			Err(RecvError) => {
				let Message = format!("Effect execution canceled; oneshot channel closed. Error: {}", RecvError);
				error!("{}", Message);
				// Convert the channel receive error into the effect's error type.
				Err(CommonError::IPCError { Description:Message }.into())
			},
		}
	}
}

impl Clone for ApplicationRunTime {
	/// Implements `Clone` to satisfy Tauri's `State` management requirements.
	fn clone(&self) -> Self { Self { Scheduler:self.Scheduler.clone(), Environment:self.Environment.clone() } }
}
