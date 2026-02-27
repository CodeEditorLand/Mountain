//! # Window Build Module
//!
//! Creates and configures the main application window.

use tauri::{App, WebviewUrl, WebviewWindowBuilder, Wry};

/// Creates and configures the main application window.
///
/// # Arguments
///
/// * `Application` - The Tauri application instance
/// * `LocalhostUrl` - The localhost URL for the webview content
///
/// # Returns
///
/// A configured `WebviewWindow<Wry>` instance.
///
/// # Platform-Specific Behavior
///
/// - Windows/macOS/Linux: Sets title, maximized state, no decorations, and
///   shadow effect
/// - Debug builds: Automatically opens DevTools
pub fn WindowBuild(Application:&mut App, LocalhostUrl:String) -> tauri::WebviewWindow<Wry> {
	// Create the window URL pointing to the application
	let WindowUrl = WebviewUrl::External(
		format!("{}/index.html", LocalhostUrl)
			.parse()
			.expect("FATAL: Failed to parse localhost URL"),
	);

	// Configure window builder with base settings
	let mut WindowBuilder = WebviewWindowBuilder::new(Application, "main", WindowUrl)
		.use_https_scheme(false)
		.initialization_script("")
		.zoom_hotkeys_enabled(true)
		.browser_extensions_enabled(false);

	// Apply platform-specific window configurations
	#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
	{
		WindowBuilder = WindowBuilder.title("Mountain").maximized(true).decorations(false).shadow(true);
	}

	// Build the main window
	let MainWindow = WindowBuilder.build().expect("FATAL: Main window build failed");

	// Open DevTools in debug builds
	#[cfg(debug_assertions)]
	{
		MainWindow.open_devtools();
	}

	MainWindow
}
