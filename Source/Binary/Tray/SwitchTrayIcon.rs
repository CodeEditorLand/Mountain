// =============================================================================
// Binary / Tray / SwitchTrayIcon
// =============================================================================
//
//! # Switch Tray Icon Command
//!
//! Tauri command to dynamically switch the tray icon based on the theme.
//!
//! ## RESPONSIBILITIES
//!
//! ### Icon Switching
//! - Switch between light and dark theme tray icons
//! - Load appropriate icon bytes from embedded resources
//! - Handle icon loading errors gracefully
//!
//! ### Theme Integration
//! - Respond to theme changes from the frontend
//! - Provide smooth visual transition when switching themes
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Tauri command handler for tray icon switching
//! - Exposed to frontend for theme integration
//! - Part of the Binary/Tray subsystem
//!
//! ### Dependencies
//! - Tauri: AppHandle, image loading, and tray API
//! - log: Error logging
//!
//! ### Dependents
//! - Frontend (Sky): Invokes this command when theme changes
//!
//! ## Planned Work
//!
//! - Support for custom icon paths
//! - Icon caching to reduce memory usage
//! - Animated tray icons
//! - Icon transition effects
//! - Support for third-party icon themes
//! - Platform-specific icon format requirements
//! - Icon size optimization for different DPI settings
//! - Icon loading performance characteristics

use tauri::{AppHandle, image::Image};

use crate::dev_log;

/// Dynamically switches the tray icon based on the theme (Light/Dark).
/// Can be invoked from the frontend when the theme changes.
///
/// # Parameters
///
/// - `App`: Tauri application handle
/// - `IsDarkMode`: Whether dark mode is active
///
/// # Behavior
///
/// - Loads the appropriate icon bytes from embedded resources
/// - Updates the tray icon with the new image
/// - Logs errors if icon loading or setting fails
///
/// # Errors
///
/// - Logs warnings/errors but doesn't panic:
///   - If tray with ID 'tray' not found
///   - If icon bytes fail to load
///   - If setting the new icon fails
#[tauri::command]
pub fn SwitchTrayIcon(App:AppHandle, IsDarkMode:bool) {
	dev_log!("window", "[UI] [Tray] Switching icon. IsDarkMode: {}", IsDarkMode);

	const DARK_ICON_BYTES:&[u8] = include_bytes!("../../../icons/32x32.png");

	const LIGHT_ICON_BYTES:&[u8] = include_bytes!("../../../icons/32x32.png");

	let IconBytes = match IsDarkMode {
		true => DARK_ICON_BYTES,

		false => LIGHT_ICON_BYTES,
	};

	match App.tray_by_id("tray") {
		Some(Tray) => {
			match Image::from_bytes(IconBytes) {
				Ok(IconImage) => {
					match Tray.set_icon(Some(IconImage)) {
						Ok(()) => {},

						Err(e) => dev_log!("window", "error: [UI] [Tray] Failed to set icon: {}", e),
					}
				},

				Err(e) => dev_log!("window", "error: [UI] [Tray] Failed to load icon bytes: {}", e),
			}
		},

		None => dev_log!("window", "warn: [UI] [Tray] Tray with ID 'tray' not found."),
	}
}
