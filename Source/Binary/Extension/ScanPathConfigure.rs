//! # Extension Scan Path Configure Module
//!
//! Configures extension scan paths from the executable directory.

use std::path::PathBuf;

use log::{debug, info};

use crate::ApplicationState::{ApplicationState, MapLockError};

/// Configures extension scan paths by resolving paths from the executable
/// directory.
///
/// # Arguments
///
/// * `AppState` - The application state containing ExtensionScanPaths
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Scan Path Configuration
///
/// This function adds the following default scan paths:
/// - `../Resources/extensions` - Bundled extensions in app resources directory
/// - `extensions` - Local extensions directory relative to executable
///
/// # Errors
///
/// Returns an error if ExtensionScanPaths mutex lock fails.
pub fn ScanPathConfigure(AppState:&std::sync::Arc<ApplicationState>) -> Result<Vec<PathBuf>, String> {
	debug!("[Extensions] [ScanPaths] Locking ExtensionScanPaths...");

	let mut ScanPathsGuard = AppState
		.Extension
		.Registry
		.ExtensionScanPaths
		.lock()
		.map_err(MapLockError)
		.map_err(|e| format!("Failed to lock ExtensionScanPaths: {}", e))?;

	debug!("[Extensions] [ScanPaths] Adding default scan paths...");

	// Resolve paths from executable directory
	if let Ok(ExecutableDirectory) = std::env::current_exe() {
		if let Some(Parent) = ExecutableDirectory.parent() {
			// Standard Tauri bundle path: ../Resources/extensions
			let ResourcesPath = Parent.join("../Resources/extensions");
			debug!("[Extensions] [ScanPaths] + {}", ResourcesPath.display());
			ScanPathsGuard.push(ResourcesPath);

			// Debug/dev path: Target/debug/extensions
			let LocalPath = Parent.join("extensions");
			debug!("[Extensions] [ScanPaths] + {}", LocalPath.display());
			ScanPathsGuard.push(LocalPath);

			// Sky Target path: where CopyVSCodeAssets copies built-in
			// extensions during the build.
			let SkyTargetPath = Parent.join("../../../Element/Sky/Target/Static/Application/extensions");
			if SkyTargetPath.exists() {
				debug!("[Extensions] [ScanPaths] + {} (Sky Target)", SkyTargetPath.display());
				ScanPathsGuard.push(SkyTargetPath);
			}

			// VS Code dependency path: built-in extensions from the VS Code
			// source checkout. Primary source in dev — avoids requiring a copy
			// step. Production builds use Sky Target or Resources instead.
			let DependencyPath = Parent.join("../../../Dependency/Microsoft/Dependency/Editor/extensions");
			if DependencyPath.exists() {
				debug!("[Extensions] [ScanPaths] + {} (VS Code Dependency)", DependencyPath.display());
				ScanPathsGuard.push(DependencyPath);
			}
		}
	}

	let ScanPaths = ScanPathsGuard.clone();

	info!("[Extensions] [ScanPaths] Configured: {:?}", ScanPaths);

	Ok(ScanPaths)
}
