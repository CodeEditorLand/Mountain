//! # Cocoon Start Module
//!
//! Initializes and starts the Cocoon sidecar process.

use log::{error, info};

use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	ProcessManagement::CocoonManagement::InitializeCocoon,
};

/// Starts the Cocoon sidecar process for build tool support.
///
/// # Arguments
///
/// * `ApplicationHandle` - The Tauri application handle
/// * `Environment` - The Mountain environment instance
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Cocoon Sidecar Functionality
///
/// The Cocoon sidecar provides:
/// - Build tool integration
/// - Process management for external tools
/// - Communication bridge with external build processes
///
/// # Errors
///
/// Returns an error if Cocoon initialization fails.
pub async fn CocoonStart(
	ApplicationHandle:&tauri::AppHandle,
	Environment:&std::sync::Arc<MountainEnvironment>,
) -> Result<(), String> {
	match InitializeCocoon(ApplicationHandle, Environment).await {
		Ok(()) => {
			info!("[Cocoon] [Start] Cocoon sidecar started successfully.");
			Ok(())
		},
		Err(e) => {
			error!("[Cocoon] [Start] Failed to start: {}", e);
			Err(format!("Failed to start Cocoon sidecar: {}", e))
		},
	}
}
