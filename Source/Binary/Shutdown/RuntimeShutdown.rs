//! # Runtime Shutdown Module
//!
//! Handles graceful shutdown of the ApplicationRunTime.

use std::sync::Arc;

use log::{debug, error, info};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Shuts down the ApplicationRunTime and its effect execution engine.
///
/// # Arguments
///
/// * `ApplicationHandle` - The Tauri application handle
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Shutdown Process
///
/// This function performs:
/// - Stops all running tasks and effects
/// - Cleans up internal resources
/// - Ensures graceful termination of the runtime
///
/// # Errors
///
/// Returns an error if ApplicationRunTime is not found or shutdown fails.
pub async fn RuntimeShutdown(ApplicationHandle:&tauri::AppHandle) -> Result<(), String> {
	debug!("[Shutdown] [Runtime] Shutting down ApplicationRunTime...");

	let RunTime = ApplicationHandle
		.try_state::<Arc<ApplicationRunTime>>()
		.ok_or("ApplicationRunTime not found in state")?;

	RunTime.inner().clone().Shutdown().await;

	info!("[Shutdown] [Runtime] ApplicationRunTime stopped.");

	Ok(())
}
