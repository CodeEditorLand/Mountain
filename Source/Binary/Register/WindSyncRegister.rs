//! # Wind Sync Register Module
//!
//! Initializes the Wind advanced sync functionality.

use crate::IPC::initialize_wind_advanced_sync;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use log::{error, info};
use std::sync::Arc;

/// Initializes the Wind advanced sync with the ApplicationRunTime.
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
/// # Wind Advanced Sync Functionality
///
/// The Wind advanced sync module provides:
/// - Document synchronization across instances
/// - Sync status tracking and reporting
/// - Update subscription management
/// - Real-time document change propagation
///
/// # Errors
///
/// Returns an error if Wind advanced sync initialization fails.
pub fn WindSyncRegister(
	ApplicationHandle: &tauri::AppHandle,
	RunTime: Arc<ApplicationRunTime>,
) -> Result<(), String> {
	match initialize_wind_advanced_sync(ApplicationHandle, RunTime) {
		Ok(()) => {
			info!("[IPC] [WindSync] Wind advanced sync initialized successfully.");
			Ok(())
		},
		Err(e) => {
			error!("[IPC] [WindSync] Failed to initialize: {}", e);
			Err(format!("Failed to initialize Wind advanced sync: {}", e))
		},
	}
}
