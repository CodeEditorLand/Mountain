//! # Tauri Build Module
//!
//! Configures and creates the Tauri Builder with platform-specific settings.

use tauri::Wry;

/// Creates and configures the Tauri Builder with platform-specific
/// configurations.
///
/// # Returns
///
/// A configured `tauri::Builder<Wry>` ready for plugin and window
/// configuration.
///
/// # Platform-Specific Behavior
///
/// - Windows/Linux: Enables any_thread configuration
/// - macOS: Uses default threading (no special configuration)
pub fn TauriBuild() -> tauri::Builder<Wry> {
	// Initialize the builder with default configuration
	let Builder = tauri::Builder::default();

	// Disable Tauri's default macOS main menu so it doesn't pre-install
	// Edit > Undo/Redo entries that compete with Monaco's undo stack and
	// cause the WKWebView NSUndoManager to intercept Cmd+Z before the
	// workbench sees it. The app menu is set explicitly in AppMenu.rs
	// (SetAppMenu call in AppLifecycle.rs) without Undo/Redo items.
	#[cfg(target_os = "macos")]
	let Builder = Builder.enable_macos_default_menu(false);

	// Apply platform-specific configurations
	#[cfg(any(windows, target_os = "linux"))]
	{
		let Builder = Builder.any_thread();

		return Builder;
	}

	Builder
}
