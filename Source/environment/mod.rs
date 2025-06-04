// src/environment/mod.rs

// Declare all the sub-modules
mod config_provider;
mod documents_provider;
mod fs_provider;
// ... other provider modules ...
mod language_features_provider;
mod utils; // For shared helpers

// Re-export the main MountainEnvironment struct and other key items if needed.
// The MountainEnvironment struct itself can be defined here or in one of the
// sub-modules and then re-exported. For simplicity, let's define it here and
// have specific trait implementations in the sub-modules.

use std::sync::Arc;

use Land_Common::environment::Environment;
use log::info;
use tauri::{AppHandle, State as TauriState, Wry}; // Renamed to avoid conflict // Base trait

// Main struct definition
#[derive(Clone)]
pub struct MountainEnvironment {
	app_handle:AppHandle<Wry>,
}

impl MountainEnvironment {
	pub fn new(app_handle:AppHandle<Wry>) -> Self {
		info!("[Env Init] MountainEnvironment instance created (from environment/mod.rs).");
		Self { app_handle }
	}

	// Helper to get AppState, used by many provider impls
	pub(crate) fn get_app_state(&self) -> TauriState<'_, crate::app_state::AppState> {
		self.app_handle.state::<crate::app_state::AppState>()
	}

	// The get_file_provider_capabilities method can live here or in utils.rs
	// Let's put it here for now as it's closely tied to MountainEnvironment's
	// capabilities.
	pub fn get_file_provider_capabilities(&self) -> u32 {
		// ... (implementation from previous synthesis) ...
		use Land_Common::fs_effects::FileSystemProviderCapabilities;
		let mut capabilities = FileSystemProviderCapabilities::FileReadWrite as u32
			| FileSystemProviderCapabilities::FileOpenReadWriteLock as u32
			| FileSystemProviderCapabilities::FileFolderCopy as u32;
		if std::env::consts::OS != "windows" {
			capabilities |= FileSystemProviderCapabilities::PathCaseSensitive as u32;
		}
		log::debug!("[MountainEnv] File provider capabilities for 'file' scheme: {}", capabilities);
		capabilities
	}
}

impl Environment for MountainEnvironment {}

// Now, each sub-module (e.g., fs_provider.rs) will contain:
// use super::MountainEnvironment; // To access the struct
// use Land_Common::environment::Requires;
// use Land_Common::fs_effects::{FsReader, FsWriter};
// use async_trait::async_trait;
// ... other necessary imports ...
//
// #[async_trait]
// impl FsReader for MountainEnvironment { ... }
//
// impl Requires<Arc<dyn FsReader + Send + Sync>> for MountainEnvironment { ...
// }
