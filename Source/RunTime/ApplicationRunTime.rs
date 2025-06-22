// File: Mountain/Source/RunTime/ApplicationRunTime.rs
// Role: Defines the concrete, `Echo`-based `ApplicationRunTime`.
// Responsibilities:
//   - The core execution engine bridging `ActionEffect`s with the `Echo`
//     scheduler.
//   - Provides the `Run` method to execute any effect, supplying the required
//     capability.
//   - Orchestrates the graceful shutdown of application services.

//! # ApplicationRunTime
//!
//! Defines the concrete, `Echo`-based `ApplicationRunTime` for the Mountain
//! application. This is the core execution engine that bridges the declarative
//! `ActionEffect` system with the high-performance `Echo` task scheduler.

use std::sync::Arc;

use Common::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::{Environment::Environment, HasEnvironment::HasEnvironment, Requires::Requires},
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
	Terminal::TerminalProvider::TerminalProvider as TerminalProviderTrait,
};
use Echo::Scheduler::Scheduler::Scheduler;
use async_trait::async_trait;
use log::{error, info};
use tokio::sync::oneshot;

use crate::Environment::MountainEnvironment::MountainEnvironment;

/// A `RunTime` that uses a high-performance, work-stealing scheduler (`Echo`)
/// to execute all `ActionEffect`s.
#[derive(Clone)]
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
	pub fn Create(Scheduler:Arc<Scheduler>, Environment:Arc<MountainEnvironment>) -> Self {
		info!("[ApplicationRunTime] New Echo-based instance created.");

		Self { Scheduler, Environment }
	}

	/// Orchestrates the graceful shutdown of all services.
	pub async fn Shutdown(&self) {
		info!("[ApplicationRunTime] Initiating graceful shutdown of services...");

		// 1. Shutdown Cocoon
		let IPCProvider:Arc<dyn IPCProvider> = self.Environment.Require();

		if let Err(e) = IPCProvider
			.SendNotificationToSidecar("cocoon-main".to_string(), "$shutdown".to_string(), serde_json::Value::Null)
			.await
		{
			error!("[ApplicationRunTime] Failed to send shutdown signal to Cocoon: {}", e);
		}

		// Give Cocoon a moment to process the shutdown before we proceed.
		tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

		// 2. Dispose of all active terminals
		let TerminalProvider:Arc<dyn TerminalProviderTrait> = self.Environment.Require();

		let TerminalIds:Vec<u64> = {
			let TerminalsGuard = self.Environment.ApplicationState.ActiveTerminals.lock().unwrap();

			TerminalsGuard.keys().cloned().collect()
		};

		for id in TerminalIds {
			if let Err(e) = TerminalProvider.DisposeTerminal(id).await {
				error!("[ApplicationRunTime] Failed to dispose terminal {}: {}", id, e);
			}
		}

		// TODO: Add final flush for storage services if they become debounced.

		info!("[ApplicationRunTime] Service shutdown tasks complete.");
	}
}

// Implement the marker trait to satisfy the bounds on ApplicationRunTimeTrait
impl HasEnvironment for ApplicationRunTime {
	type EnvironmentType = MountainEnvironment;

	fn GetEnvironment(&self) -> Arc<Self::EnvironmentType> { self.Environment.clone() }
}

// The ApplicationRunTime is not an environment itself, but it needs this marker
// to satisfy some complex generic bounds in the effect system.
impl Environment for ApplicationRunTime {}

#[async_trait]
impl ApplicationRunTimeTrait for ApplicationRunTime {
	/// The core integration logic between `Common::ActionEffect` and
	/// `Echo::Scheduler`.
	async fn Run<TCapabilityProvider, TError, TOutput>(
		&self,

		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		Self::EnvironmentType: Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static, {
		let (ResultSender, ResultReceiver) = oneshot::channel::<Result<TOutput, TError>>();

		let CapabilityProvider:Arc<TCapabilityProvider> = self.Environment.Require();

		let Task = async move {
			let Result = Effect.Apply(CapabilityProvider).await;

			if ResultSender.send(Result).is_err() {
				error!("[ApplicationRunTime] Failed to send effect result; receiver was dropped.");
			}
		};

		self.Scheduler.Submit(Task, Echo::Task::Priority::Priority::Normal);

		match ResultReceiver.await {
			Ok(Result) => Result,

			Err(RecvError) => {
				let Message = format!("Effect execution canceled; oneshot channel closed. Error: {}", RecvError);

				error!("{}", Message);

				Err(CommonError::IPCError { Description:Message }.into())
			},
		}
	}
}
