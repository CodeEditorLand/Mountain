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
			// Standard Tauri bundle path: ../Resources/extensions.
			// When launched from a `.app`, Parent is `Contents/MacOS/` and
			// this resolves to `Contents/Resources/extensions`.
			let ResourcesPath = Parent.join("../Resources/extensions");
			dev_log!("extensions", "[Extensions] [ScanPaths] + {}", ResourcesPath.display());
			ScanPathsGuard.push(ResourcesPath);

			// VS Code-style bundle layout: `.app/Contents/Resources/app/extensions`.
			// Some tooling copies built-ins here; probe both conventions so a
			// single bundle works regardless of which copy step placed them.
			let ResourcesAppPath = Parent.join("../Resources/app/extensions");
			dev_log!("extensions", "[Extensions] [ScanPaths] + {}", ResourcesAppPath.display());
			ScanPathsGuard.push(ResourcesAppPath);

			// Debug/dev path: Target/debug/extensions
			let LocalPath = Parent.join("extensions");
			dev_log!("extensions", "[Extensions] [ScanPaths] + {}", LocalPath.display());
			ScanPathsGuard.push(LocalPath);

			// Dev-only fallback paths: the monorepo layout
			// (Element/Mountain/Target/debug/) is not present in shipped
			// bundles. In production, the ../Resources/extensions path above
			// is authoritative. Gate with cfg to keep release builds lean.
			#[cfg(debug_assertions)]
			{
				// Sky Target path: where CopyVSCodeAssets copies built-in
				// extensions during the build.
				let SkyTargetPath = Parent.join("../../../Sky/Target/Static/Application/extensions");
				if SkyTargetPath.exists() {
					dev_log!("extensions", "[Extensions] [ScanPaths] + {} (Sky Target, dev)", SkyTargetPath.display());
					ScanPathsGuard.push(SkyTargetPath);
				}

				// VS Code dependency path: built-in extensions from the VS
				// Code source checkout — avoids requiring a copy step in dev.
				let DependencyPath = Parent.join("../../../../Dependency/Microsoft/Dependency/Editor/extensions");
				if DependencyPath.exists() {
					dev_log!("extensions", "[Extensions] [ScanPaths] + {} (VS Code Dependency, dev)", DependencyPath.display());
					ScanPathsGuard.push(DependencyPath);
				}
			}
		}
	}

	// User-scope paths: always scanned, independent of whether the binary
	// was launched from the repo, a `.app`, or a symlink on the Desktop.
	// Mirrors VS Code's `~/.vscode-oss/extensions` convention.
	if let Some(HomeDirectory) = dirs::home_dir() {
		for Suffix in [
			".codeeditorland/extensions",
			".land/extensions",
		] {
			let UserExtensionPath = HomeDirectory.join(Suffix);
			dev_log!("extensions", "[Extensions] [ScanPaths] + {} (User)", UserExtensionPath.display());
			ScanPathsGuard.push(UserExtensionPath);
		}
	}

	let ScanPaths = ScanPathsGuard.clone();

	dev_log!("extensions", "[Extensions] [ScanPaths] Configured: {:?}", ScanPaths);

	Ok(ScanPaths)
}
