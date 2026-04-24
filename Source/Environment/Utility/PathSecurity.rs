//! # Path Security Utilities
//!
//! Functions for validating filesystem access and enforcing workspace trust.

use std::path::{Path, PathBuf};

use CommonLibrary::Error::CommonError::CommonError;

use crate::{ApplicationState::ApplicationState, dev_log};

/// A critical security helper that checks if a given filesystem path is
/// allowed for access.
///
/// The access model has two tiers:
///
/// 1. **Trusted system paths** - directories Land itself owns (user
///    extensions, agent plugins, app-support storage, bundled extension
///    roots). These are never "user content" and the extension scanner,
///    VSIX installer, and global-storage probes must be able to read/write
///    them regardless of which workspace folder is open. They bypass the
///    workspace-folder check entirely.
///
/// 2. **Workspace content** - everything else is only reachable when the
///    resolved path is a descendant of a currently registered, trusted
///    workspace folder. That's the sandbox boundary that keeps extensions
///    from rifling through `$HOME` via `vscode.workspace.fs`.
///
/// Without tier 1, the scanner's read of `~/.land/extensions` is
/// rejected as "Path is outside of the registered workspace folders", so
/// user-installed VSIXes never reach the Extensions sidebar even though
/// they are present on disk.
pub fn IsPathAllowedForAccess(ApplicationState:&ApplicationState, PathToCheck:&Path) -> Result<(), CommonError> {
	dev_log!("vfs", "[EnvironmentSecurity] Verifying path: {}", PathToCheck.display());

	// Tier 1: trusted system paths bypass workspace gating. See
	// `IsTrustedSystemPath` for the complete allow-list. Scanner reads,
	// VSIX installs, agent-plugin probes, and per-extension global-storage
	// stats hit this path on every boot.
	if IsTrustedSystemPath(PathToCheck) {
		return Ok(());
	}

	if !ApplicationState.Workspace.IsTrusted.load(std::sync::atomic::Ordering::Relaxed) {
		return Err(CommonError::FileSystemPermissionDenied {
			Path:PathToCheck.to_path_buf(),
			Reason:"Workspace is not trusted. File access is denied.".to_string(),
		});
	}

	let FoldersGuard = ApplicationState
		.Workspace
		.WorkspaceFolders
		.lock()
		.map_err(super::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

	if FoldersGuard.is_empty() {
		// Allow access if no folder is open, as operations are likely on user-chosen
		// files. A stricter model could deny this.
		return Ok(());
	}

	let IsAllowed = FoldersGuard.iter().any(|Folder| {
		match Folder.URI.to_file_path() {
			Ok(FolderPath) => PathToCheck.starts_with(FolderPath),
			Err(_) => false,
		}
	});

	if IsAllowed {
		Ok(())
	} else {
		Err(CommonError::FileSystemPermissionDenied {
			Path:PathToCheck.to_path_buf(),
			Reason:"Path is outside of the registered workspace folders.".to_string(),
		})
	}
}

/// Return `true` when `PathToCheck` falls under a directory that Land itself
/// manages and the sandbox should not gate.
///
/// Covered roots:
///
/// - `${LAND_USER_EXTENSION_DIRECTORY}` (explicit override, if set).
/// - `$HOME/.land/**` - the canonical namespace for user-installed
///   extensions, agent plugins, global storage, and any other Land-owned
///   state that lives outside the VS Code-style profile tree.
/// - The Mountain executable's own `extensions/`, `../Resources/extensions/`
///   and `../Resources/app/extensions/` neighbours - built-in extension
///   roots that ship inside the `.app` bundle.
/// - `$APPDATA`-equivalents: Tauri's resolved app-data / app-config /
///   app-local directories (via `$XDG_DATA_HOME`, `$XDG_CONFIG_HOME` if
///   set; on macOS the `Library/Application Support/land.editor.*` tree).
/// - `${TMPDIR}` - short-lived temp files the installer unpacks into.
///
/// Anything outside this list still flows through the workspace-folder
/// check. The set is intentionally narrow: it unblocks Land's *own*
/// bookkeeping reads without handing extensions an unbounded filesystem.
fn IsTrustedSystemPath(PathToCheck:&Path) -> bool {
	// Canonicalising is best-effort - when the path doesn't exist yet
	// (e.g. first-boot probes for `globalStorage/<extension>/state.json`)
	// `canonicalize` returns Err and we compare against the raw path.
	let Candidate = PathToCheck.canonicalize().unwrap_or_else(|_| PathToCheck.to_path_buf());

	if let Ok(Override) = std::env::var("LAND_USER_EXTENSION_DIRECTORY") {
		if !Override.is_empty() {
			let OverridePath = PathBuf::from(&Override);
			if Candidate.starts_with(&OverridePath) || PathToCheck.starts_with(&OverridePath) {
				return true;
			}
		}
	}

	if let Ok(Home) = std::env::var("HOME") {
		let LandRoot = PathBuf::from(&Home).join(".land");
		if Candidate.starts_with(&LandRoot) || PathToCheck.starts_with(&LandRoot) {
			return true;
		}

		// macOS / Linux Application-Support trees that host Land's per-profile
		// state. `land.editor.*` prefix matches every build profile variant.
		let MacAppSupport = PathBuf::from(&Home).join("Library/Application Support");
		if (Candidate.starts_with(&MacAppSupport) || PathToCheck.starts_with(&MacAppSupport))
			&& ContainsLandEditorSegment(PathToCheck)
		{
			return true;
		}

		let XdgConfig = std::env::var("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(&Home).join(".config"));
		if (Candidate.starts_with(&XdgConfig) || PathToCheck.starts_with(&XdgConfig))
			&& ContainsLandEditorSegment(PathToCheck)
		{
			return true;
		}

		let XdgData = std::env::var("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(&Home).join(".local/share"));
		if (Candidate.starts_with(&XdgData) || PathToCheck.starts_with(&XdgData))
			&& ContainsLandEditorSegment(PathToCheck)
		{
			return true;
		}
	}

	if let Ok(Exe) = std::env::current_exe() {
		if let Some(ExeParent) = Exe.parent() {
			let BundleRoots = [
				ExeParent.join("extensions"),
				ExeParent.join("../Resources/extensions"),
				ExeParent.join("../Resources/app/extensions"),
				// Sky's Static/Application/extensions root is reached via
				// `../../../Sky/Target/Static/Application/extensions` in the
				// debug profile - match the canonical `Sky/Target/Static/Application/extensions`
				// segment regardless of how many `..` hops the scan path used.
			];
			for Root in BundleRoots {
				let Normalised = Root.canonicalize().unwrap_or(Root.clone());
				if Candidate.starts_with(&Normalised) || PathToCheck.starts_with(&Root) {
					return true;
				}
			}
		}
	}

	// Sky / Dependency bundled extension trees. These are debug-profile
	// layouts where the scanner reaches the bundle root via relative hops
	// from the Mountain executable directory - canonicalising already
	// resolves that, but we also fall back to a path-segment match so a
	// missing file (first-boot probe) still clears the check.
	if ContainsPathSegments(PathToCheck, &["Sky", "Target", "Static", "Application", "extensions"])
		|| ContainsPathSegments(PathToCheck, &["Dependency", "Microsoft", "Dependency", "Editor", "extensions"])
	{
		return true;
	}

	if let Ok(TempDir) = std::env::var("TMPDIR") {
		let TempPath = PathBuf::from(&TempDir);
		if !TempPath.as_os_str().is_empty()
			&& (Candidate.starts_with(&TempPath) || PathToCheck.starts_with(&TempPath))
		{
			return true;
		}
	}

	false
}

/// True when `path` contains a directory segment whose name starts with
/// `land.editor.`. Used to tighten the Application-Support / XDG checks so
/// we only trust directories that Land itself provisioned, not every file
/// under `$HOME/Library/Application Support`.
fn ContainsLandEditorSegment(path:&Path) -> bool {
	path.components().any(|Component| {
		Component
			.as_os_str()
			.to_str()
			.map(|Name| Name.starts_with("land.editor."))
			.unwrap_or(false)
	})
}

/// True when every element of `segments` appears in order as consecutive
/// path components of `path`. Used to match Sky / Dependency extension
/// roots regardless of which relative-path prefix the scanner used.
fn ContainsPathSegments(path:&Path, segments:&[&str]) -> bool {
	let Names:Vec<&str> = path.components().filter_map(|C| C.as_os_str().to_str()).collect();
	if segments.is_empty() || Names.len() < segments.len() {
		return false;
	}
	Names.windows(segments.len()).any(|Window| Window.iter().zip(segments.iter()).all(|(A, B)| A == B))
}
