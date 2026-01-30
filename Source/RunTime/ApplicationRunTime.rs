// File: Mountain/Source/RunTime/ApplicationRunTime.rs
// Role: Defines the concrete, `Echo`-based `ApplicationRunTime`.
// Responsibilities:
//   - The core execution engine bridging `ActionEffect`s with the `Echo`
//     scheduler.
//   - Provides the `Run` method to execute any effect, supplying the required
//     capability.
//   - Orchestrates the graceful shutdown of application services.

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # ApplicationRunTime
//!
//! Defines the concrete, `Echo`-based `ApplicationRunTime` for the Mountain
//! application. This is the core execution engine that bridges the declarative
//! `ActionEffect` system with the high-performance `Echo` task scheduler.

#![allow(non_snake_case, non_camel_case_types)]

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
use log::{error, info, warn, debug};
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

		let shutdown_result = self.ShutdownWithRecovery().await;

		match shutdown_result {
			Ok(()) => info!("[ApplicationRunTime] Service shutdown tasks completed successfully."),
			Err(error) => error!("[ApplicationRunTime] Service shutdown completed with errors: {}", error),
		}
	}

	/// Enhanced shutdown with comprehensive error handling and recovery
	pub async fn ShutdownWithRecovery(&self) -> Result<(), CommonError> {
		info!("[ApplicationRunTime] Initiating robust shutdown with recovery...");

		let mut shutdown_errors: Vec<String> = Vec::new();

		// 1. Shutdown Cocoon with retry mechanism
		match self.ShutdownCocoonWithRetry().await {
			Ok(()) => debug!("[ApplicationRunTime] Cocoon shutdown successful"),
			Err(error) => {
				shutdown_errors.push(format!("Cocoon shutdown failed: {}", error));
				warn!("[ApplicationRunTime] Cocoon shutdown failed, continuing with other services...");
			},
		}

		// 2. Dispose of all active terminals with error handling
		match self.DisposeTerminalsSafely().await {
			Ok(()) => debug!("[ApplicationRunTime] Terminal disposal successful"),
			Err(error) => {
				shutdown_errors.push(format!("Terminal disposal failed: {}", error));
				warn!("[ApplicationRunTime] Terminal disposal failed, continuing...");
			},
		}

		// 3. Save application state
		match self.SaveApplicationState().await {
			Ok(()) => debug!("[ApplicationRunTime] Application state saved"),
			Err(error) => {
				shutdown_errors.push(format!("State save failed: {}", error));
				warn!("[ApplicationRunTime] Failed to save application state, continuing...");
			},
		}

		// 4. Flush any pending operations
		self.FlushPendingOperations().await;

		if !shutdown_errors.is_empty() {
			Err(CommonError::Unknown {
				Description: format!("Shutdown completed with {} errors: {:?}", shutdown_errors.len(), shutdown_errors),
			})
		} else {
			Ok(())
		}
	}

	/// Shutdown Cocoon with retry mechanism
	async fn ShutdownCocoonWithRetry(&self) -> Result<(), CommonError> {
		let IPCProvider:Arc<dyn IPCProvider> = self.Environment.Require();

		let mut attempts = 0;
		let max_attempts = 3;

		while attempts < max_attempts {
			match IPCProvider
				.SendNotificationToSideCar("cocoon-main".to_string(), "$shutdown".to_string(), serde_json::Value::Null)
				.await
			{
				Ok(()) => {
					// Give Cocoon a moment to process the shutdown
					tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
					return Ok(());
				},
				Err(error) => {
					attempts += 1;
					if attempts == max_attempts {
						return Err(error);
					}

					warn!(
						"[ApplicationRunTime] Cocoon shutdown attempt {} failed: {}. Retrying...",
						attempts,
						error
					);

					tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
				},
			}
		}

		Err(CommonError::Unknown {
			Description: "Failed to shutdown Cocoon after maximum retries".to_string(),
		})
	}

	/// Safely dispose of all active terminals
	async fn DisposeTerminalsSafely(&self) -> Result<(), CommonError> {
		let TerminalProvider:Arc<dyn TerminalProviderTrait> = self.Environment.Require();

		let TerminalIds:Vec<u64> = {
			let TerminalsGuard = self.Environment.ApplicationState.ActiveTerminals.lock()
				.map_err(|e| CommonError::StateLockPoisoned { Context: e.to_string() })?;

			TerminalsGuard.keys().cloned().collect()
		};

		let mut disposal_errors: Vec<String> = Vec::new();

		for id in TerminalIds {
			match TerminalProvider.DisposeTerminal(id).await {
				Ok(()) => debug!("[ApplicationRunTime] Terminal {} disposed successfully", id),
				Err(error) => {
					disposal_errors.push(format!("Terminal {}: {}", id, error));
					warn!("[ApplicationRunTime] Failed to dispose terminal {}: {}", id, error);
				},
			}
		}

		if !disposal_errors.is_empty() {
			Err(CommonError::Unknown {
				Description: format!("Terminal disposal completed with {} errors: {:?}", disposal_errors.len(), disposal_errors),
			})
		} else {
			Ok(())
		}
	}

	/// Save application state before shutdown
	async fn SaveApplicationState(&self) -> Result<(), CommonError> {
		debug!("[ApplicationRunTime] Saving application state...");

		// Save global memento
		let global_memento_guard = self.Environment.ApplicationState.GlobalMemento.lock()
			.map_err(|e| CommonError::StateLockPoisoned { Context: e.to_string() })?;

		let global_memento_path = &self.Environment.ApplicationState.GlobalMementoPath;

		if let Some(parent) = global_memento_path.parent() {
			if !parent.exists() {
				std::fs::create_dir_all(parent).map_err(|e| {
					CommonError::FileSystemIO {
						Path: parent.to_path_buf(),
						Description: e.to_string(),
					}
				})?;
			}
		}

		let memento_json = serde_json::to_string_pretty(&*global_memento_guard)
			.map_err(|e| CommonError::SerializationError { Description: e.to_string() })?;

		std::fs::write(global_memento_path, memento_json)
			.map_err(|e| CommonError::FileSystemIO {
				Path: global_memento_path.clone(),
				Description: e.to_string(),
			})
	}

	/// Flush any pending operations
	async fn FlushPendingOperations(&self) {
		debug!("[ApplicationRunTime] Flushing pending operations...");

		// Flush pending UI requests
		let mut pending_requests_guard = self.Environment.ApplicationState.PendingUserInterfaceRequests.lock()
			.unwrap_or_else(|e| {
				error!("[ApplicationRunTime] Failed to lock pending UI requests: {}", e);
				e.into_inner()
			});

for (_request_id, sender) in pending_requests_guard.drain() {
			let _ = sender.send(Err(CommonError::Unknown {
				Description: "Application shutting down".to_string(),
			}));
		}

		debug!("[ApplicationRunTime] Pending operations flushed");
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

impl ApplicationRunTime {
	/// Enhanced effect execution with timeout and recovery
	pub async fn RunWithTimeout<TCapabilityProvider, TError, TOutput>(
		&self,
		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
		timeout: std::time::Duration,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		Self::EnvironmentType: Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static,
	{
		tokio::time::timeout(timeout, self.Run(Effect))
			.await
			.map_err(|_| {
				CommonError::Unknown {
					Description: format!("Effect execution timed out after {:?}", timeout),
				}.into()
			})?
	}

	/// Execute effect with retry mechanism
	pub async fn RunWithRetry<TCapabilityProvider, TError, TOutput>(
		&self,
		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
		max_retries: u32,
		initial_delay: std::time::Duration,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		Self::EnvironmentType: Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static,
	{
		let mut retry_count = 0;
		let mut current_delay = initial_delay;

		while retry_count <= max_retries {
			match self.Run(Effect.clone()).await {
				Ok(result) => return Ok(result),
				Err(error) => {
					if retry_count == max_retries {
						return Err(error);
					}

					retry_count += 1;
					warn!(
						"[ApplicationRunTime] Effect execution failed (attempt {}): {}. Retrying in {:?}...",
						retry_count,
						error,
						current_delay
					);

					tokio::time::sleep(current_delay).await;
					current_delay *= 2; // Exponential backoff
				},
			}
		}

		Err(CommonError::Unknown {
			Description: format!("Effect execution failed after {} retries", max_retries),
		}.into())
	}
}
