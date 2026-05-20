//! macOS application menu - Edit submenu without Undo/Redo.
//!
//! Tauri's `Builder::default()` on macOS installs the standard AppKit menu
//! bar, which includes `Edit → Undo (Cmd+Z)` and `Edit → Redo (Cmd+Shift+Z)`.
//! These native items intercept Cmd+Z at the macOS responder-chain level
//! *before* the WKWebView's JavaScript keydown handler fires, so VS Code's
//! Monaco editor never sees the event - the native WKWebView text-buffer undo
//! runs instead, and nothing happens visually (Monaco's undo stack is
//! untouched). Ctrl+Z reaches the JS layer because no native menu binds it.
//!
//! Removing Undo/Redo from the native Edit menu lets Cmd+Z pass through to
//! the WKWebView's keydown handler where VS Code registers `meta+z` → undo.
//! Cut/Copy/Paste/SelectAll stay as predefined items so native text fields
//! (e.g., address bar inputs) keep working correctly.
//!
//! On Windows / Linux, no menu override is applied - those platforms do not
//! have the same WKWebView responder-chain interception.

use crate::dev_log;

/// Install a custom app menu on `App`, removing Undo/Redo from the Edit
/// submenu so Cmd+Z reaches VS Code's Monaco keybinding handler.
///
/// Called once from `AppLifecycleSetup` immediately after the main window
/// is built. A failure here is non-fatal (logs a warning and skips the
/// override so the default menu remains).
#[cfg(target_os = "macos")]
pub fn SetAppMenu(App: &tauri::App) {
	use tauri::menu::{MenuBuilder, SubmenuBuilder};

	let Result = (|| -> Result<(), Box<dyn std::error::Error>> {
		// Build Edit submenu: Cut / Copy / Paste / ── / Select All.
		// Undo and Redo are intentionally absent so Cmd+Z and Cmd+Shift+Z
		// reach the WKWebView JS layer where Monaco handles them.
		// `.cut()/.copy()/.paste()/.separator()/.select_all()` are the
		// infallible convenience methods on SubmenuBuilder (return Self).
		let EditSubmenu = SubmenuBuilder::new(App, "Edit")
			.cut()
			.copy()
			.paste()
			.separator()
			.select_all()
			.build()?;

		// MenuBuilder automatically prepends the macOS App menu (About,
		// Services, Hide, Quit etc.) as the first entry when running on
		// macOS, so we only need to append the submenus we care about.
		// `.item()` on MenuBuilder also returns Self (infallible).
		let Menu = MenuBuilder::new(App).item(&EditSubmenu).build()?;

		App.set_menu(Menu)?;

		dev_log!("lifecycle", "[UI] [Menu] macOS Edit menu set (Undo/Redo removed; Cmd+Z routes to Monaco).");

		Ok(())
	})();

	if let Err(Error) = Result {
		dev_log!(
			"lifecycle",
			"warn: [UI] [Menu] Failed to override macOS app menu ({}); default menu retained - Cmd+Z may trigger \
			 native undo instead of Monaco undo.",
			Error
		);
	}
}

/// No-op on non-macOS platforms - the Edit menu interception is macOS-specific.
#[cfg(not(target_os = "macos"))]
pub fn SetAppMenu(_App: &tauri::App) {}
