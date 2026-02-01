//! # Status Reporter Register Module
//!
//! Initializes the IPC status reporting functionality.

use crate::IPC::initialize_status_reporter;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use log::{error, info};
use std::sync::Arc;

/// Initializes the IPC status reporting with the ApplicationRunTime.
///
/// # Arguments
///
/// * `ApplicationHandle` - The Tauri application handle
/// * `RunTime` - The ApplicationRunTime instance
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Status Reporter Functionality
///
/// The status reporter provides:
/// - IPC connection status tracking
/// - Historical status data collection
/// - Real-time status reporting to frontend
/// - Subscription-based status updates
///
/// # Errors
///
/// Returns an error if status reporter initialization fails.
pub fn StatusReporterRegister(
	ApplicationHandle: &tauri::AppHandle,
	RunTime: Arc<ApplicationRunTime>,
) -> Result<(), String> {
	match initialize_status_reporter(ApplicationHandle, RunTime) {
		Ok(()) => {
			info!("[IPC] [StatusReporter] Status reporter initialized successfully.");
			Ok(())
		},
		Err(e) => {
			error!("[IPC] [StatusReporter] Failed to initialize: {}", e);
			Err(format!("Failed to initialize status reporter: {}", e))
		},
	}
}
