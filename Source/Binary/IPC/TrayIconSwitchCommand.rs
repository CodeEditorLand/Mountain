//! # TrayIconSwitchCommand
//!
//! Dynamically switches the system tray icon based on the application theme.
//!
//! ## RESPONSIBILITIES
//!
//! ### Icon Management
//! - Handle requests to switch tray icon based on theme (Light/Dark)
//! - Load appropriate icon bytes from embedded resources
//! - Update the system tray icon with proper error handling
//! - Validate application handle and tray availability
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC command in Binary subsystem
//! - UI integration point for theme-based icon switching
//!
//! ### Dependencies
//! - tauri: Application handle and tray management
//! - log: Logging framework
//!
//! ### Dependents
//! - Sky frontend: Calls this command when theme changes
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Icon files are embedded at compile time, no runtime file access
//! - No user input validation needed for boolean theme parameter
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Icon bytes are embedded at compile time, no disk I/O
//! - Icon update is synchronous but very fast
//! - Tray icon updates are cached by the OS

use log::{debug, error, warn};
use tauri::{AppHandle, image::Image};

/// Dynamically switches the tray icon based on the theme (Light/Dark).
///
/// This command is invoked from the frontend when the application theme changes,
//! allowing the system tray icon to match the current visual theme for consistency.
///
/// # Arguments
///
/// * `App` - Tauri application handle for accessing the system tray
/// * `IsDarkMode` - Boolean flag indicating if dark mode is active
///
/// # Returns
///
/// Returns nothing. Errors are logged but do not propagate to the frontend
/// since icon switching is not critical functionality.
///
/// # Note
///
/// Currently both dark and light icons point to the same 32x32.png file.
/// In the future, separate icon files should be provided for each theme.
#[tauri::command]
pub fn SwitchTrayIcon(App: AppHandle, IsDarkMode: bool) {
	debug!("[UI] [Tray] Switching icon. IsDarkMode: {}", IsDarkMode);

	// Icon bytes embedded at compile time for both themes
	// TODO: Provide separate icon files for dark/light themes
	const DARK_ICON_BYTES: &[u8] = include_bytes!("../../icons/32x32.png");
	const LIGHT_ICON_BYTES: &[u8] = include_bytes!("../../icons/32x32.png");

	// Select appropriate icon bytes based on theme
	let IconBytes = if IsDarkMode { DARK_ICON_BYTES } else { LIGHT_ICON_BYTES };

	// Retrieve the tray by its ID and update the icon
	if let Some(Tray) = App.tray_by_id("tray") {
		match Image::from_bytes(IconBytes) {
			Ok(IconImage) => {
				if let Err(e) = Tray.set_icon(Some(IconImage)) {
					error!("[UI] [Tray] Failed to set icon: {}", e);
				}
			},
			Err(e) => error!("[UI] [Tray] Failed to load icon bytes: {}", e),
		}
	} else {
		warn!("[UI] [Tray] Tray with ID 'tray' not found.");
	}
}
