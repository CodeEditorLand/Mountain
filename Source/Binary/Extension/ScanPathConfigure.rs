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

	// Skip all built-in extensions when either the legacy
	// `LAND_SKIP_BUILTIN_EXTENSIONS` or the `.env.Land.Extensions` flag
	// `LAND_DISABLE_BUILTIN_EXTENSIONS` is set. Both accepted so kernel /
	// minimal profiles and the skill-file env stay in sync. User scan path
	// still runs so VSIX-installed extensions remain visible.
	let SkipBuiltins = matches!(std::env::var("LAND_SKIP_BUILTIN_EXTENSIONS").as_deref(), Ok("1") | Ok("true"))
		|| matches!(std::env::var("LAND_DISABLE_BUILTIN_EXTENSIONS").as_deref(), Ok("1") | Ok("true"));

	if SkipBuiltins {
		dev_log!(
			"extensions",
			"[Extensions] [ScanPaths] LAND_SKIP_BUILTIN_EXTENSIONS=true - skipping all built-in paths, keeping user \
			 path"
		);
	} else {
		dev_log!("extensions", "[Extensions] [ScanPaths] Adding default scan paths...");
	}

	// `LAND_BUILTIN_EXTENSIONS_DIR` takes precedence over the executable-
	// relative probing chain. Useful for CI builds where the bundle layout
	// differs from both the `.app` convention and the repo layout.
	if !SkipBuiltins {
		if let Ok(Override) = std::env::var("LAND_BUILTIN_EXTENSIONS_DIR") {
			let OverridePath = ExpandUserPath(&Override);
			if OverridePath.exists() {
				dev_log!(
					"extensions",
					"[Extensions] [ScanPaths] + {} (LAND_BUILTIN_EXTENSIONS_DIR)",
					OverridePath.display()
				);
				ScanPathsGuard.push(OverridePath);
			} else {
				dev_log!(
					"extensions",
					"warn: [Extensions] [ScanPaths] LAND_BUILTIN_EXTENSIONS_DIR={} does not exist; ignoring",
					Override
				);
			}
		}
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
				// `.exists()` guard and silently skip - no need for a
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
				// Code source checkout - avoids requiring a copy step.
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
	//
	// Atom U1: `LAND_USER_EXTENSIONS_DIR` overrides the default
	// `~/.land/extensions`. Useful for per-workspace sandboxes, shared
	// caches on CI, or running against a test extensions set without
	// polluting the user's real profile.
	if let Ok(UserOverride) = std::env::var("LAND_USER_EXTENSIONS_DIR") {
		let OverridePath = ExpandUserPath(&UserOverride);
		dev_log!(
			"extensions",
			"[Extensions] [ScanPaths] + {} (LAND_USER_EXTENSIONS_DIR)",
			OverridePath.display()
		);
		ScanPathsGuard.push(OverridePath);
	} else if let Some(HomeDirectory) = dirs::home_dir() {
		let UserExtensionPath = HomeDirectory.join(".land/extensions");
		dev_log!(
			"extensions",
			"[Extensions] [ScanPaths] + {} (User)",
			UserExtensionPath.display()
		);
		ScanPathsGuard.push(UserExtensionPath);
	}

	// Atom U1: additional paths via `LAND_EXTRA_EXTENSIONS_DIRS`. Mirrors
	// VS Code's `--extensions-dir=<a>:<b>:<c>` CLI. Platform-separator:
	// semicolon on Windows (matches PATHEXT), colon elsewhere.
	if let Ok(Extras) = std::env::var("LAND_EXTRA_EXTENSIONS_DIRS") {
		let Separator = if cfg!(target_os = "windows") { ';' } else { ':' };
		for Candidate in Extras.split(Separator) {
			let Trimmed = Candidate.trim();
			if Trimmed.is_empty() {
				continue;
			}
			let ExtraPath = ExpandUserPath(Trimmed);
			dev_log!(
				"extensions",
				"[Extensions] [ScanPaths] + {} (LAND_EXTRA_EXTENSIONS_DIRS)",
				ExtraPath.display()
			);
			ScanPathsGuard.push(ExtraPath);
		}
	}

	// Atom U1: development extensions path - the VS Code equivalent of
	// `--extensionDevelopmentPath=<dir>`. Extensions here always load
	// regardless of enablement state; kept separate from user-scope so a
	// broken dev extension doesn't persist into the user's profile.
	if let Ok(DevExtensions) = std::env::var("LAND_DEV_EXTENSIONS_DIR") {
		let DevPath = ExpandUserPath(&DevExtensions);
		dev_log!(
			"extensions",
			"[Extensions] [ScanPaths] + {} (LAND_DEV_EXTENSIONS_DIR)",
			DevPath.display()
		);
		ScanPathsGuard.push(DevPath);
	}

	let ScanPaths = ScanPathsGuard.clone();

	dev_log!("extensions", "[Extensions] [ScanPaths] Configured: {:?}", ScanPaths);

	Ok(ScanPaths)
}

/// Expand a leading `~/` to `$HOME/` for user-provided paths. Env-var
/// overrides frequently come from operators typing `~/.vscode/extensions`
/// without shell expansion (e.g. in `.env` files, GUI launchers, sidecar
/// manifests). Leaves absolute and relative paths untouched.
fn ExpandUserPath(Raw:&str) -> PathBuf {
	if let Some(Stripped) = Raw.strip_prefix("~/") {
		if let Some(Home) = dirs::home_dir() {
			return Home.join(Stripped);
		}
	}
	PathBuf::from(Raw)
}
