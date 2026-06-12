//! # Runtime Shutdown Module
//!
//! Handles graceful shutdown of the ApplicationRunTime.

use std::sync::Arc;

use tauri::Manager;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Shuts down the ApplicationRunTime and its effect execution engine.
///
/// # Parameters
///
/// - `ApplicationHandle` — The Tauri application handle
///
/// # Returns
///
/// `Ok(())` on success, or `Err(String)` if shutdown fails.
///
/// ## Behaviour
///
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
