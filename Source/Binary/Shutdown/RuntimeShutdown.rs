//! # Runtime Shutdown Module
//!
//! Handles graceful shutdown of the ApplicationRunTime.

use std::sync::Arc;

use tauri::Manager;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

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

	dev_log!("lifecycle", "[Shutdown] [Runtime] Shutting down ApplicationRunTime...");

	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	RunTime.Shutdown().await;

	dev_log!("lifecycle", "[Shutdown] [Runtime] ApplicationRunTime stopped.");

	Ok(())
}
