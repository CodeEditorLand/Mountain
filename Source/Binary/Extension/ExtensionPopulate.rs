//! # Extension Populate Module
//!
//! Scans and populates extensions from configured scan paths.

use crate::{ApplicationState::Struct::ApplicationState::ApplicationState, dev_log};

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
pub async fn Fn(ApplicationHandle:tauri::AppHandle, AppState:&std::sync::Arc<ApplicationState>) -> Result<(), String> {
	match crate::ApplicationState::Internal::ExtensionScanner::ScanAndPopulateExtensions::Fn(
		ApplicationHandle.clone(),
		&AppState.Extension,
	)
	.await
	{
		Ok(()) => {
			dev_log!(
				"extensions",
				"[Extensions] [Populate] Extensions scanned and populated successfully."
			);

			Ok(())
		},

		Err(e) => {
			dev_log!("extensions", "error: [Extensions] [Populate] Failed: {}", e);

			Err(format!("Failed to scan and populate extensions: {}", e))
		},
	}
}
