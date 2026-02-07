//! # Extension Populate Module
//!
//! Scans and populates extensions from configured scan paths.

use log::{error, info};

use crate::ApplicationState::{ApplicationState, Internal::ScanAndPopulateExtensions};

/// Scans and populates extensions from the configured scan paths.
///
/// # Arguments
///
/// * `ApplicationHandle` - The Tauri application handle
/// * `AppState` - The application state containing extension information
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Extension Scanning Process
///
/// This function performs:
/// - Scanning all configured extension directories
/// - Parsing extension metadata and manifests
/// - Loading extension capabilities
/// - Registering extensions with the application
///
/// # Errors
///
/// Returns an error if extension scanning or population fails.
pub async fn ExtensionPopulate(
	ApplicationHandle:tauri::AppHandle,
	AppState:&std::sync::Arc<ApplicationState>,
) -> Result<(), String> {
	match ScanAndPopulateExtensions(ApplicationHandle.clone(), &AppState.Extension).await {
		Ok(()) => {
			info!("[Extensions] [Populate] Extensions scanned and populated successfully.");
			Ok(())
		},
		Err(e) => {
			error!("[Extensions] [Populate] Failed: {}", e);
			Err(format!("Failed to scan and populate extensions: {}", e))
		},
	}
}
