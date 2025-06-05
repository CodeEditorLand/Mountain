// ---------------------------------------------------------------------------------------------
// Mountain Environment - Main Module (environment/mod.rs)
// --------------------------------------------------------------------------------------------
// This module serves as the root for the `MountainEnvironment` and its various
// provider trait implementations. It defines the main `MountainEnvironment`
// struct and declares sub-modules where specific provider traits are
// implemented.
//
// The `MountainEnvironment` is the concrete realization of the abstract
// `Environment` concept from `Land_Common`, providing the actual "native"
// backend logic for `ActionEffect`s.
// --------------------------------------------------------------------------------------------

// Declare all the sub-modules where provider traits are implemented.
// These files will live in the `src/environment/` directory.
pub(crate) mod commands_provider;
pub(crate) mod config_provider;
pub(crate) mod diagnostics_provider;
pub(crate) mod documents_provider;
pub(crate) mod fs_provider;
pub(crate) mod ipc_provider;
pub(crate) mod language_features_provider;
pub(crate) mod output_provider;
pub(crate) mod secrets_provider;
pub(crate) mod storage_provider;
pub(crate) mod ui_provider;
pub(crate) mod workspace_provider;

// Shared utility functions for the environment module.
pub(crate) mod utils;

use std::sync::Arc;

// Import base Environment trait from Land_Common
use Land_Common::environment::Environment;
use Land_Common::fs_effects::FileSystemProviderCapabilities;
use log::{debug, info};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State as TauriState, Wry}; // For get_file_provider_capabilities

// --- Mountain Environment Struct Definition ---

/// Concrete implementation of the `Environment` and various provider traits.
///
/// This struct holds an `AppHandle` to interact with Tauri and access
/// `AppState`. It provides the "native" logic that backs `ActionEffect`s.
#[derive(Clone)]
pub struct MountainEnvironment {
	// Wry is the default Tauri webview runtime.
	// AppHandle provides access to Tauri's managed state (AppState),
	// path resolver, event system, window management, etc.
	app_handle:AppHandle<Wry>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment`.
	/// This is typically called once during application startup in `main.rs`.
	pub fn new(app_handle:AppHandle<Wry>) -> Self {
		info!("[Env Init] MountainEnvironment instance created.");
		Self { app_handle }
	}

	/// Helper to get a Tauri `State` wrapper for `AppState`.
	/// This is a common operation for many provider implementations.
	/// Marked `pub(crate)` as it's an internal detail of the `environment`
	/// module.
	pub(crate) fn get_app_state(&self) -> TauriState<'_, crate::app_state::AppState> {
		self.app_handle.state::<crate::app_state::AppState>()
	}

	/// Retrieves the filesystem capabilities for the 'file' scheme
	/// as implemented by this environment.
	/// Used during initialization to inform Cocoon about Mountain's FS
	/// capabilities.
	pub fn get_file_provider_capabilities(&self) -> u32 {
		// Start with basic read/write and locking capabilities.
		let mut capabilities = FileSystemProviderCapabilities::FileReadWrite as u32
			| FileSystemProviderCapabilities::FileOpenReadWriteLock as u32
			| FileSystemProviderCapabilities::FileFolderCopy as u32; // Assuming basic copy is supported

		// PathCaseSensitive is typically true for Linux/macOS and false for Windows.
		if std::env::consts::OS != "windows" {
			capabilities |= FileSystemProviderCapabilities::PathCaseSensitive as u32;
		}

		// Add other capabilities if Mountain's FsWriter/FsReader supports them:
		// if atomic_writes_supported { capabilities |=
		// FileSystemProviderCapabilities::FileAtomicWrite as u32; }
		// if atomic_reads_supported { capabilities |=
		// FileSystemProviderCapabilities::FileAtomicRead as u32; }
		// if atomic_directory_creation_supported { capabilities |=
		// FileSystemProviderCapabilities::FileAtomicReadDirectory as u32; } // This
		// seems misnamed in enum, likely meant for atomic dir create
		// if trash_supported { capabilities |= FileSystemProviderCapabilities::Trash as
		// u32; } if file_cloning_supported { capabilities |=
		// FileSystemProviderCapabilities::FileClone as u32; }

		debug!(
			"[MountainEnv] File provider capabilities for 'file' scheme determined: {}",
			capabilities
		);
		capabilities
	}
}

// Implement the base Environment trait (currently a marker trait).
impl Environment for MountainEnvironment {}

// Note: Specific provider trait implementations (e.g., `impl FsReader for
// MountainEnvironment`) and their corresponding `Requires` implementations will
// be in the sub-modules (e.g., `fs_provider.rs`). Each sub-module will use
// `super::MountainEnvironment`.
