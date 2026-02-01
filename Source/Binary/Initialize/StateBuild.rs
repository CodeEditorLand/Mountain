//! # StateBuild
//!
//! Builds the ApplicationState for the application.
//!
//! ## RESPONSIBILITIES
//!
//! ### State Construction
//! - Create ApplicationState with workspace folders
//! - Initialize workspace configuration path
//! - Set default state values
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Early initialization component in Binary subsystem
//! - Provides global application state
//!
//! ### Dependencies
//! - crate::ApplicationState: Application state type
//!
//! ### Dependents
//! - Fn() main entry point: Uses built state
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate workspace paths before storing in state
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - State construction is fast, in-memory only

use std::sync::Arc;
use crate::ApplicationState::ApplicationState;

/// Build ApplicationState with initial workspace folders.
///
/// Creates the ApplicationState with the provided workspace folders
/// and workspace configuration path.
///
/// # Arguments
///
/// * `Folders` - Initial workspace folders
/// * `ConfigPath` - Workspace configuration file path
///
/// # Returns
///
/// Returns an Arc wrapping the constructed ApplicationState.
pub fn Build(Folders: Vec<String>, ConfigPath: Option<std::path::PathBuf>) -> Arc<ApplicationState> {
	Arc::new(ApplicationState {
		WorkSpaceFoldersValue: Arc::new(std::sync::Mutex::new(Folders)),
		WorkSpaceConfigurationPathValue: Arc::new(std::sync::Mutex::new(ConfigPath)),
		..ApplicationState::default()
	})
}
