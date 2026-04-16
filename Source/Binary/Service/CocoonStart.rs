//! # Cocoon Start Module
//!
//! Initializes and starts the Cocoon sidecar process.


use crate::{
use crate::dev_log;
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
			dev_log!("cocoon", "[Cocoon] [Start] Cocoon sidecar started successfully.");
			Ok(())
		},
		Err(e) => {
			dev_log!("cocoon", "warn: [Cocoon] [Start] Cocoon unavailable (degraded mode): {}", e);
			Ok(()) // Graceful degradation — workbench works without Cocoon
		},
	}
}
