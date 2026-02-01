//! # Tauri Build Module
//!
//! Configures and creates the Tauri Builder with platform-specific settings.

use tauri::Wry;

/// Creates and configures the Tauri Builder with platform-specific configurations.
///
/// # Returns
///
/// A configured `tauri::Builder<Wry>` ready for plugin and window configuration.
///
/// # Platform-Specific Behavior
///
/// - Windows/Linux: Enables any_thread configuration
/// - macOS: Uses default threading (no special configuration)
pub fn TauriBuild() -> tauri::Builder<Wry> {
	// Initialize the builder with default configuration
	let mut Builder = tauri::Builder::default();

	// Apply platform-specific configurations
	#[cfg(any(windows, target_os = "linux"))]
	{
		Builder = Builder.any_thread();
	}

	Builder
}
