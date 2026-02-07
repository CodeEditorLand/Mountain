//! # Command Register Module
//!
//! Registers native commands with the Tauri application.

use crate::{ApplicationState::ApplicationState, Command};

/// Registers all native commands with the Tauri application.
///
/// # Arguments
///
/// * `ApplicationHandle` - The Tauri application handle
/// * `AppState` - The application state structure
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Registered Commands
///
/// This function delegates to `Command::Bootstrap::RegisterNativeCommands`
/// which registers all native commands for frontend communication:
/// - TreeView commands (GetTreeViewChildren)
/// - Language features (ProvideHover, ProvideCompletions, ProvideDefinition,
///   ProvideReferences)
/// - Source Control Management commands
/// - Keybinding commands
/// - UI request dispatchers (DispatchFrontendCommand, ResolveUIRequest)
///
/// # Errors
///
/// Returns an error if command registration fails.
pub fn CommandRegister(
	ApplicationHandle:&tauri::AppHandle,
	AppState:&std::sync::Arc<ApplicationState>,
) -> Result<(), String> {
	Command::Bootstrap::RegisterNativeCommands(ApplicationHandle, AppState)
		.map_err(|Error| format!("Failed to register native commands: {}", Error))?;

	Ok(())
}
