//! # Advanced Features Register Module
//!
//! Initializes the IPC advanced features functionality.

use crate::IPC::initialize_advanced_features;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use log::{error, info};
use std::sync::Arc;

/// Initializes the IPC advanced features with the ApplicationRunTime.
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
/// # Advanced Features Functionality
///
/// The advanced features module provides:
/// - Performance statistics collection
/// - Cache management and monitoring
/// - Collaboration session management
/// - Advanced IPC capabilities
///
/// # Errors
///
/// Returns an error if advanced features initialization fails.
pub fn AdvancedFeaturesRegister(
	ApplicationHandle: &tauri::AppHandle,
	RunTime: Arc<ApplicationRunTime>,
) -> Result<(), String> {
	match initialize_advanced_features(ApplicationHandle, RunTime) {
		Ok(()) => {
			info!("[IPC] [AdvancedFeatures] Advanced features initialized successfully.");
			Ok(())
		},
		Err(e) => {
			error!("[IPC] [AdvancedFeatures] Failed to initialize: {}", e);
			Err(format!("Failed to initialize advanced features: {}", e))
		},
	}
}
