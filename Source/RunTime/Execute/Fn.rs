//! # Fn (RunTime::Execute)
//!
//! ## RESPONSIBILITIES
//!
//! Core effect execution functions that bridge the declarative ActionEffect
//! system with the Echo scheduler for high-performance task execution.
//!
//! ## ARCHITECTURAL ROLE
//!
//! The execution engine in Mountain's architecture that handles the "how"
//! of effect execution, while ActionEffect defines the "what".
//!
//! ## KEY COMPONENTS
//!
//! - **Run**: Basic effect execution through Echo scheduler
//! - **RunWithTimeout**: Timeout-based execution with cancellation
//! - **RunWithRetry**: Retry mechanisms with exponential backoff
//!
//! ## ERROR HANDLING
//!
//! All errors are propagated through Result<T, E> with detailed context.
//! Effect errors are converted to CommonError when appropriate.
//! Timeouts return timeout-specific errors.
//! Retry failures include attempt count and last error information.
//!
//! ## LOGGING
//!
//! Uses log crate with appropriate severity levels:
//! - `info`: Effect submission and completion
//! - `debug`: Detailed execution steps
//! - `warn`: Retry attempts and recoverable errors
//! - `error`: Failed operations and timeout occurrences
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Uses oneshot channels for result collection (minimal overhead)
//! - Tasks are submitted to Echo's work-stealing scheduler
//! - Timeout uses tokio::time::timeout for efficient cancellation
//! - Retry with exponential backoff prevents system overload
//!
//! ## TODO
//!
//! None

use std::sync::Arc;

use CommonLibrary::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
};
use Echo::Task::Priority::Priority;
use async_trait::async_trait;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use crate::dev_log;

/// The core integration logic between `Common::ActionEffect` and
/// `Echo::Scheduler`.
#[async_trait]
impl ApplicationRunTimeTrait for ApplicationRunTime {
	async fn Run<TCapabilityProvider, TError, TOutput>(
		&self,
		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		<Self as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
			Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static, {
		let (ResultSender, ResultReceiver) = tokio::sync::oneshot::channel::<Result<TOutput, TError>>();

		let CapabilityProvider:Arc<TCapabilityProvider> = self.Environment.Require();

		let Task = async move {
			let Result = Effect.Apply(CapabilityProvider).await;

			if ResultSender.send(Result).is_err() {
				dev_log!("lifecycle", "error: [ApplicationRunTime] Failed to send effect result; receiver was dropped.");
			}
		};

		self.Scheduler.Submit(Task, Priority::Normal);

		match ResultReceiver.await {
			Ok(Result) => Result,

			Err(_RecvError) => {
				let Message = "Effect execution canceled; oneshot channel closed.".to_string();

				dev_log!("lifecycle", "error: {}", Message);

				Err(CommonError::IPCError { Description:Message }.into())
			},
		}
	}
}

impl ApplicationRunTime {
	/// Enhanced effect execution with timeout and recovery.
	pub async fn RunWithTimeout<TCapabilityProvider, TError, TOutput>(
		&self,
		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
		timeout:std::time::Duration,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		<Self as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
			Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static,
		TOutput: Send + Sync + 'static, {
		tokio::time::timeout(timeout, ApplicationRunTimeTrait::Run(self, Effect))
			.await
			.map_err(|_| {
				CommonError::Unknown { Description:format!("Effect execution timed out after {:?}", timeout) }.into()
			})?
	}

	/// Execute effect with retry mechanism.
	pub async fn RunWithRetry<TCapabilityProvider, TError, TOutput>(
		&self,
		Effect:ActionEffect<Arc<TCapabilityProvider>, TError, TOutput>,
		max_retries:u32,
		initial_delay:std::time::Duration,
	) -> Result<TOutput, TError>
	where
		TCapabilityProvider: ?Sized + Send + Sync + 'static,
		<Self as CommonLibrary::Environment::HasEnvironment::HasEnvironment>::EnvironmentType:
			Requires<TCapabilityProvider>,
		TError: From<CommonError> + Send + Sync + 'static + std::fmt::Display,
		TOutput: Send + Sync + 'static, {
		let mut retry_count = 0;
		let mut current_delay = initial_delay;

		while retry_count <= max_retries {
			match ApplicationRunTimeTrait::Run(self, Effect.clone()).await {
				Ok(result) => return Ok(result),
				Err(error) => {
					if retry_count == max_retries {
						return Err(error);
					}

					retry_count += 1;
					dev_log!("lifecycle", "warn: [ApplicationRunTime] Effect execution failed (attempt {}): {}. Retrying in {:?}...",
						retry_count, error, current_delay);

					tokio::time::sleep(current_delay).await;

					// Apply exponential backoff by doubling the delay after each failure
					// to prevent overwhelming the system during recovery attempts.
					current_delay *= 2;
				},
			}
		}

		Err(
			CommonError::Unknown { Description:format!("Effect execution failed after {} retries", max_retries) }
				.into(),
		)
	}
}
