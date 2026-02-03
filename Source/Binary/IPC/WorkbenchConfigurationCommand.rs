//! # WorkbenchConfigurationCommand
//!
//! Provides the initial workbench configuration to the Sky frontend via IPC.
//!
//! ## RESPONSIBILITIES
//!
//! ### Configuration Retrieval
//! - Handle IPC requests for workbench configuration
//! - Construct sandbox configuration from initialization data
//! - Validate configuration construction with proper error handling
//! - Return JSON configuration payload to frontend
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC bridge command in Binary subsystem
//! - Frontend-facing API for initial workspace setup
//!
//! ### Dependencies
//! - crate::ProcessManagement::InitializationData: Configuration construction
//! - crate::ApplicationState: Application state management
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//! - log: Logging framework
//!
//! ### Dependents
//! - Sky frontend: Requests workbench configuration on load
//! - Tauri IPC handler: Routes requests to this command
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Configuration data may contain workspace paths; ensure they are validated
//! - No user input is processed beyond initial workspace argument
//! - Error messages should not leak sensitive information
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Configuration construction involves file I/O; should be fast
//! - Consider caching if configuration becomes expensive to compute
//! - Async execution won't block main thread

use log::debug;
use std::sync::Arc;
use tauri::{AppHandle, State};
use serde_json::Value;

use crate::{ApplicationState::ApplicationState::ApplicationState, ProcessManagement::InitializationData};

/// Provides the initial workbench configuration to the Sky frontend.
///
/// This command is called by the frontend during initialization to receive
/// the sandbox configuration including workspace folders, settings, and
/// other application state needed to bootstrap the UI.
///
/// # Arguments
///
/// * `ApplicationHandle` - Tauri application handle for accessing system
///   resources
/// * `State` - Global application state containing workspace information
///
/// # Returns
///
/// Returns a JSON object containing the workbench configuration on success,
/// or a string error message on failure.
///
/// # Errors
///
/// Returns an error string if:
/// - Configuration construction fails (file system errors, JSON parsing)
/// - State locking fails (concurrent access issues)
#[tauri::command]
pub async fn MountainGetWorkbenchConfiguration(
	ApplicationHandle:AppHandle,
	State:State<'_, Arc<ApplicationState>>,
) -> Result<Value, String> {
	debug!("[IPC] [WorkbenchConfig] Request received.");

	debug!("[IPC] [WorkbenchConfig] Constructing sandbox configuration...");

	let Config = InitializationData::ConstructSandboxConfiguration(&ApplicationHandle, State.inner())
		.await
		.map_err(|Error| {
			debug!("[IPC] [WorkbenchConfig] Failed: {}", Error);
			Error.to_string()
		})?;

	debug!("[IPC] [WorkbenchConfig] Success. Returning payload.");

	Ok(Config)
}
