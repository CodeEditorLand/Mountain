//! # Cocoon Start Module
//!
//! Initializes and starts the Cocoon sidecar process.

use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	ProcessManagement::CocoonManagement::InitializeCocoon,
	dev_log,
};

/// Starts the Cocoon sidecar process for build tool support.
///
/// # Parameters
///
/// - `ApplicationHandle` — Tauri application handle for service registration.
/// - `Environment` — Mountain environment instance providing configuration.
///
/// # Returns
///
/// `Ok(())` on success, or `Err(String)` if initialization fails. Cocoon
/// failure is non-fatal — the workbench continues in degraded mode.
///
/// # Errors
///
/// Returns an error if Cocoon initialization fails (binary not found,
/// gRPC connection timeout, etc.).
pub async fn Fn(
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

			Ok(()) // Graceful degradation - workbench works without Cocoon
		},
	}
}
