//! # Extension Scan Path Configure Module
//!
//! Configures extension scan paths from the executable directory.

use std::path::PathBuf;

use crate::{
	ApplicationState::{ApplicationState, MapLockError},
	dev_log,
};

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

	// Atom J3: kernel / minimal profiles set LAND_SKIP_BUILTIN_EXTENSIONS=true
	// to ship without any bundled extensions. The user-extensions path
	// (`~/.land/extensions`) still scans so VSIX-installed extensions work.
	let SkipBuiltins = matches!(std::env::var("LAND_SKIP_BUILTIN_EXTENSIONS").as_deref(), Ok("1") | Ok("true"));

	if SkipBuiltins {
		dev_log!(
			"extensions",
			"[Extensions] [ScanPaths] LAND_SKIP_BUILTIN_EXTENSIONS=true — skipping all built-in paths, keeping user \
			 path"
		);
	} else {
		dev_log!("extensions", "[Extensions] [ScanPaths] Adding default scan paths...");
	}

	// Resolve paths from executable directory
	if !SkipBuiltins {
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

				// Monorepo-layout fallback paths: resolved relative to
				// `Element/Mountain/Target/{debug,release}/`, so they only
				// materialise when the binary runs from inside the repo.
				// Shipped `.app`s launched from `/Applications/` hit the
				// `.exists()` guard and silently skip — no need for a
				// `cfg(debug_assertions)` gate. Keeping these live in release
				// lets a raw `Target/release/<name>` launch find the same 98
				// built-in extensions a debug build does.
				//
				// Sky Target path: where CopyVSCodeAssets copies built-in
				// extensions during the Sky build.
				let SkyTargetPath = Parent.join("../../../Sky/Target/Static/Application/extensions");
				if SkyTargetPath.exists() {
					dev_log!(
						"extensions",
						"[Extensions] [ScanPaths] + {} (Sky Target, repo-layout)",
						SkyTargetPath.display()
					);
					ScanPathsGuard.push(SkyTargetPath);
				}

				// VS Code dependency path: built-in extensions from the VS
				// Code source checkout — avoids requiring a copy step.
				let DependencyPath = Parent.join("../../../../Dependency/Microsoft/Dependency/Editor/extensions");
				if DependencyPath.exists() {
					dev_log!(
						"extensions",
						"[Extensions] [ScanPaths] + {} (VS Code Dependency, repo-layout)",
						DependencyPath.display()
					);
					ScanPathsGuard.push(DependencyPath);
				}
			}
		}
	} // end !SkipBuiltins

	// User-scope paths: always scanned, independent of whether the binary
	// was launched from the repo, a `.app`, or a symlink on the Desktop.
	// Mirrors VS Code's `~/.vscode-oss/extensions` convention.
	if let Some(HomeDirectory) = dirs::home_dir() {
		let UserExtensionPath = HomeDirectory.join(".land/extensions");
		dev_log!(
			"extensions",
			"[Extensions] [ScanPaths] + {} (User)",
			UserExtensionPath.display()
		);
		ScanPathsGuard.push(UserExtensionPath);
	}

	let ScanPaths = ScanPathsGuard.clone();

	dev_log!("extensions", "[Extensions] [ScanPaths] Configured: {:?}", ScanPaths);

	Ok(ScanPaths)
}
