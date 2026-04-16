//! # Extension Scan Path Configure Module
//!
//! Configures extension scan paths from the executable directory.

use std::path::PathBuf;


use crate::ApplicationState::{ApplicationState, MapLockError};
use crate::dev_log;

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
	dev_log!("extensions", "[Extensions] [ScanPaths] Locking ExtensionScanPaths...");

	let mut ScanPathsGuard = AppState
		.Extension
		.Registry
		.ExtensionScanPaths
		.lock()
		.map_err(MapLockError)
		.map_err(|e| format!("Failed to lock ExtensionScanPaths: {}", e))?;

	dev_log!("extensions", "[Extensions] [ScanPaths] Adding default scan paths...");

	// Resolve paths from executable directory
	if let Ok(ExecutableDirectory) = std::env::current_exe() {
		if let Some(Parent) = ExecutableDirectory.parent() {
			// Standard Tauri bundle path: ../Resources/extensions
			let ResourcesPath = Parent.join("../Resources/extensions");
			dev_log!("extensions", "[Extensions] [ScanPaths] + {}", ResourcesPath.display());
			ScanPathsGuard.push(ResourcesPath);

			// Debug/dev path: Target/debug/extensions
			let LocalPath = Parent.join("extensions");
			dev_log!("extensions", "[Extensions] [ScanPaths] + {}", LocalPath.display());
			ScanPathsGuard.push(LocalPath);

			// Sky Target path: where CopyVSCodeAssets copies built-in
			// extensions during the build.
			let SkyTargetPath = Parent.join("../../../Element/Sky/Target/Static/Application/extensions");
			if SkyTargetPath.exists() {
				dev_log!("extensions", "[Extensions] [ScanPaths] + {} (Sky Target)", SkyTargetPath.display());
				ScanPathsGuard.push(SkyTargetPath);
			}

			// VS Code dependency path: built-in extensions from the VS Code
			// source checkout. Primary source in dev — avoids requiring a copy
			// step. Production builds use Sky Target or Resources instead.
			let DependencyPath = Parent.join("../../../../Dependency/Microsoft/Dependency/Editor/extensions");
			if DependencyPath.exists() {
				dev_log!("extensions", "[Extensions] [ScanPaths] + {} (VS Code Dependency)", DependencyPath.display());
				ScanPathsGuard.push(DependencyPath);
			}
		}
	}

	let ScanPaths = ScanPathsGuard.clone();

	dev_log!("extensions", "[Extensions] [ScanPaths] Configured: {:?}", ScanPaths);

	Ok(ScanPaths)
}
