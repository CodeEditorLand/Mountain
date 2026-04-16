//! # Shutdown (RunTime::Shutdown)
//!
//! ## RESPONSIBILITIES
//!
//! Service shutdown and lifecycle management for graceful application
//! termination. Coordinates cleanup of all application services with error
//! recovery.
//!
//! ## ARCHITECTURAL ROLE
//!
//! The lifecycle manager in Mountain's architecture that ensures clean
//! application termination and state persistence.
//!
//! ## KEY COMPONENTS
//!
//! - **Shutdown**: Main shutdown orchestration
//! - **ShutdownWithRecovery**: Enhanced shutdown with error recovery
//! - **ShutdownCocoonWithRetry**: Cocoon sidecar shutdown with retry
//! - **DisposeTerminalsSafely**: Terminal cleanup
//! - **SaveApplicationState**: State persistence
//! - **FlushPendingOperations**: Cleanup pending operations
//!
//! ## ERROR HANDLING
//!
//! All shutdown operations use error recovery to continue cleanup even when
//! individual services fail. Errors are collected and reported without
//! crashing. Multi-attempt retry for critical operations like Cocoon shutdown.
//!
//! ## LOGGING
//!
//! Uses log crate with appropriate severity levels:
//! - `info`: Shutdown initiation and completion
//! - `debug`: Detailed operation steps
//! - `warn`: Recoverable errors during shutdown
//! - `error`: Failed operations (but continues shutdown)
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Shutdown operations are optimized to complete quickly
//! - Sequential cleanup to avoid race conditions
//! - Minimal blocking during state persistence
//! - Uses timeout recovery to prevent hanging
//!
//! ## TODO
//!
//! None

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
	Terminal::TerminalProvider::TerminalProvider as TerminalProviderTrait,
};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use crate::dev_log;

impl ApplicationRunTime {
	/// Orchestrates the graceful shutdown of all services.
	pub async fn Shutdown(&self) {
		dev_log!("lifecycle", "[ApplicationRunTime] Initiating graceful shutdown of services...");

		let shutdown_result = self.ShutdownWithRecovery().await;

		match shutdown_result {
			Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Service shutdown tasks completed successfully."),
			Err(error) => dev_log!("lifecycle", "error: [ApplicationRunTime] Service shutdown completed with errors: {}", error),
		}
	}

	/// Enhanced shutdown with comprehensive error handling and recovery.
	pub async fn ShutdownWithRecovery(&self) -> Result<(), CommonError> {
		dev_log!("lifecycle", "[ApplicationRunTime] Initiating robust shutdown with recovery...");

		let mut shutdown_errors:Vec<String> = Vec::new();

		// 1. Shutdown Cocoon with retry mechanism
		match self.ShutdownCocoonWithRetry().await {
			Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Cocoon shutdown successful"),
			Err(error) => {
				shutdown_errors.push(format!("Cocoon shutdown failed: {}", error));
				dev_log!("lifecycle", "warn: [ApplicationRunTime] Cocoon shutdown failed, continuing with other services...");
			},
		}

		// 2. Dispose of all active terminals with error handling
		match self.DisposeTerminalsSafely().await {
			Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Terminal disposal successful"),
			Err(error) => {
				shutdown_errors.push(format!("Terminal disposal failed: {}", error));
				dev_log!("lifecycle", "warn: [ApplicationRunTime] Terminal disposal failed, continuing...");
			},
		}

		// 3. Save application state
		match self.SaveApplicationState().await {
			Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Application state saved"),
			Err(error) => {
				shutdown_errors.push(format!("State save failed: {}", error));
				dev_log!("lifecycle", "warn: [ApplicationRunTime] Failed to save application state, continuing...");
			},
		}

		// 4. Flush any pending operations
		self.FlushPendingOperations().await;

		if !shutdown_errors.is_empty() {
			Err(CommonError::Unknown {
				Description:format!(
					"Shutdown completed with {} errors: {:?}",
					shutdown_errors.len(),
					shutdown_errors
				),
			})
		} else {
			Ok(())
		}
	}

	/// Shutdown Cocoon with retry mechanism.
	pub async fn ShutdownCocoonWithRetry(&self) -> Result<(), CommonError> {
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

					dev_log!("lifecycle", "warn: [ApplicationRunTime] Cocoon shutdown attempt {} failed: {}. Retrying...",
						attempts, error);

					tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
				},
			}
		}

		Err(CommonError::Unknown { Description:"Failed to shutdown Cocoon after maximum retries".to_string() })
	}

	/// Safely dispose of all active terminals.
	pub async fn DisposeTerminalsSafely(&self) -> Result<(), CommonError> {
		let TerminalProvider:Arc<dyn TerminalProviderTrait> = self.Environment.Require();

		let TerminalIds:Vec<u64> = {
			let TerminalsGuard = self
				.Environment
				.ApplicationState
				.Feature
				.Terminals
				.ActiveTerminals
				.lock()
				.map_err(|e| CommonError::StateLockPoisoned { Context:e.to_string() })?;

			TerminalsGuard.keys().cloned().collect()
		};

		let mut disposal_errors:Vec<String> = Vec::new();

		for id in TerminalIds {
			match TerminalProvider.DisposeTerminal(id).await {
				Ok(()) => dev_log!("lifecycle", "[ApplicationRunTime] Terminal {} disposed successfully", id),
				Err(error) => {
					disposal_errors.push(format!("Terminal {}: {}", id, error));
					dev_log!("lifecycle", "warn: [ApplicationRunTime] Failed to dispose terminal {}: {}", id, error);
				},
			}
		}

		if !disposal_errors.is_empty() {
			Err(CommonError::Unknown {
				Description:format!(
					"Terminal disposal completed with {} errors: {:?}",
					disposal_errors.len(),
					disposal_errors
				),
			})
		} else {
			Ok(())
		}
	}

	/// Save application state before shutdown.
	pub async fn SaveApplicationState(&self) -> Result<(), CommonError> {
		dev_log!("lifecycle", "[ApplicationRunTime] Saving application state...");

		// Save global memento
		let global_memento_guard = self
			.Environment
			.ApplicationState
			.Configuration
			.MementoGlobalStorage
			.lock()
			.map_err(|e| CommonError::StateLockPoisoned { Context:e.to_string() })?;

		let global_memento_path = self
			.Environment
			.ApplicationState
			.GlobalMementoPath
			.lock()
			.map_err(|e| CommonError::StateLockPoisoned { Context:e.to_string() })?
			.clone();

		if let Some(parent) = global_memento_path.parent() {
			if !parent.exists() {
				std::fs::create_dir_all(parent)
					.map_err(|e| CommonError::FileSystemIO { Path:parent.to_path_buf(), Description:e.to_string() })?;
			}
		}

		let memento_json = serde_json::to_string_pretty(&*global_memento_guard)
			.map_err(|e| CommonError::SerializationError { Description:e.to_string() })?;

		std::fs::write(&global_memento_path, memento_json)
			.map_err(|e| CommonError::FileSystemIO { Path:global_memento_path.clone(), Description:e.to_string() })
	}

	/// Flush any pending operations.
	pub async fn FlushPendingOperations(&self) {
		dev_log!("lifecycle", "[ApplicationRunTime] Flushing pending operations...");

		// Flush pending UI requests
		let mut pending_requests_guard = self
			.Environment
			.ApplicationState
			.UI
			.PendingUserInterfaceRequest
			.lock()
			.unwrap_or_else(|e| {
				dev_log!("lifecycle", "error: [ApplicationRunTime] Failed to lock pending UI requests: {}", e);
				e.into_inner()
			});

		for (_request_id, sender) in pending_requests_guard.drain() {
			let _ = sender.send(Err(CommonError::Unknown {
				Description:"Application shutting down".to_string(),
			}));
		}

		dev_log!("lifecycle", "[ApplicationRunTime] Pending operations flushed");
	}
}
