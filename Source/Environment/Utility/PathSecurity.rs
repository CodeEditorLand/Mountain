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
/// 1. **Trusted system paths** - directories Land itself owns (user extensions,
///    agent plugins, app-support storage, bundled extension roots). These are
///    never "user content" and the extension scanner, VSIX installer, and
///    global-storage probes must be able to read/write them regardless of which
///    workspace folder is open. They bypass the workspace-folder check
///    entirely.
///
/// 2. **Workspace content** - everything else is only reachable when the
///    resolved path is a descendant of a currently registered, trusted
///    workspace folder. That's the sandbox boundary that keeps extensions from
///    rifling through `$HOME` via `vscode.workspace.fs`.
///
/// Without tier 1, the scanner's read of `~/.land/extensions` is
/// rejected as "Path is outside of the registered workspace folders", so
/// user-installed VSIXes never reach the Extensions sidebar even though
/// they are present on disk.
pub fn IsPathAllowedForAccess(ApplicationState:&ApplicationState, PathToCheck:&Path) -> Result<(), CommonError> {
	// Per-call verification line is one of the highest-volume tags
	// (~15k hits per long session). The failure path below logs its own
	// line; the success path is auditable from IPC-side request logs.
	// Keep under `vfs-verbose` for deep debugging only.
	dev_log!("vfs-verbose", "[EnvironmentSecurity] Verifying path: {}", PathToCheck.display());

	// Defensive: empty path would slip through the trusted-system
	// check (no allow-list segment matches) AND the workspace-
	// descendant check (`Path::starts_with("")` returns true). Without
	// this guard, an extension probing `vscode.workspace.fs.stat("")`
	// would be authorised against ANY registered workspace folder.
	// Reject up front so the caller falls through to its not-found
	// handler.
	if PathToCheck.as_os_str().is_empty() {
		return Err(CommonError::FileSystemPermissionDenied {
			Path:PathToCheck.to_path_buf(),
			Reason:"Empty path: caller must supply an explicit filesystem path.".to_string(),
		});
	}

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

	// Use canonical paths on both sides so that prefix-matching survives
	// macOS's `/Volumes/<vol>/...` vs `/private/var/...` resolution and
	// any symlinked submodule roots. Cocoon's URI strip yields the user-
	// visible path (`/Volumes/CORSAIR/.../Land/Dependency/...`) while the
	// workspace folder URL stays as built from `from_directory_path` -
	// these can disagree on platforms where the resolved canonical path
	// differs from the URI-derived one (encoded mount-point indirection,
	// case-insensitive HFS+, etc.). Without this, a workspace with deep
	// submodule trees rejects every read that walks past the first level
	// even though the path is a literal descendant of the open folder.
	let CanonicalPathToCheck = crate::Cache::PathCanon::Canonicalize::Fn(PathToCheck)
		.unwrap_or_else(|_| PathToCheck.to_path_buf());
	let IsAllowed = FoldersGuard.iter().any(|Folder| {
		let FolderPath = match Folder.URI.to_file_path() {
			Ok(P) => P,
			Err(_) => return false,
		};
		let CanonicalFolderPath = crate::Cache::PathCanon::Canonicalize::Fn(&FolderPath)
			.unwrap_or_else(|_| FolderPath.clone());
		// Try both canonical-canonical AND raw-raw - either match wins.
		PathToCheck.starts_with(&FolderPath)
			|| PathToCheck.starts_with(&CanonicalFolderPath)
			|| CanonicalPathToCheck.starts_with(&FolderPath)
			|| CanonicalPathToCheck.starts_with(&CanonicalFolderPath)
	});

	if IsAllowed {
		Ok(())
	} else {
		// Surface the comparison details so a workspace-mismatch bug
		// (URL-to-path conversion, canonicalisation drift) is debuggable
		// without rebuilding. Tag is `vfs` so it appears under the
		// default `short` trace set.
		let FolderPaths:Vec<String> = FoldersGuard
			.iter()
			.map(|F| {
				F.URI
					.to_file_path()
					.map(|P| P.display().to_string())
					.unwrap_or_else(|_| format!("<bad-uri:{}>", F.URI))
			})
			.collect();
		dev_log!(
			"vfs",
			"[PathSecurity] reject path={} canonical={} folders=[{}]",
			PathToCheck.display(),
			CanonicalPathToCheck.display(),
			FolderPaths.join(", ")
		);
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
/// - `${Lodge}` (explicit override, if set).
/// - `$HOME/.land/**` - the canonical namespace for user-installed extensions,
///   agent plugins, global storage, and any other Land-owned state that lives
///   outside the VS Code-style profile tree.
/// - The Mountain executable's own `extensions/`, `../Resources/extensions/`
///   and `../Resources/app/extensions/` neighbours - built-in extension roots
///   that ship inside the `.app` bundle.
/// - `$APPDATA`-equivalents: Tauri's resolved app-data / app-config / app-local
///   directories (via `$XDG_DATA_HOME`, `$XDG_CONFIG_HOME` if set; on macOS the
///   `Library/Application Support/land.editor.*` tree).
/// - `${TMPDIR}` + `/tmp`, `/private/tmp`, `/var/tmp` - scratch dirs language
///   servers write their port-handoff / socket / lock files to. `TMPDIR` on
///   macOS points at `/var/folders/.../T/` but extensions hardcode
///   `/tmp/<tool>` directly.
/// - Third-party tool state under `$HOME/{.gitkraken,.gk,.copilot,
///   .config/git}` - probed by GitLens, copilot-chat, etc. Application state,
///   not user content.
///
/// Anything outside this list still flows through the workspace-folder
/// check. The set is intentionally narrow: it unblocks Land's *own*
/// bookkeeping reads + cooperating neighbour-tool probes without
/// handing extensions an unbounded filesystem.
fn IsTrustedSystemPath(PathToCheck:&Path) -> bool {
	// Canonicalising is best-effort - when the path doesn't exist yet
	// (e.g. first-boot probes for `globalStorage/<extension>/state.json`)
	// `canonicalize` returns Err and we compare against the raw path.
	let Candidate = crate::Cache::PathCanon::Canonicalize::Fn(PathToCheck)
		.unwrap_or_else(|_| PathToCheck.to_path_buf());

	if let Ok(Override) = std::env::var("Lodge") {
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

		let XdgConfig = std::env::var("XDG_CONFIG_HOME")
			.map(PathBuf::from)
			.unwrap_or_else(|_| PathBuf::from(&Home).join(".config"));
		if (Candidate.starts_with(&XdgConfig) || PathToCheck.starts_with(&XdgConfig))
			&& ContainsLandEditorSegment(PathToCheck)
		{
			return true;
		}

		let XdgData = std::env::var("XDG_DATA_HOME")
			.map(PathBuf::from)
			.unwrap_or_else(|_| PathBuf::from(&Home).join(".local/share"));
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
				let Normalised = crate::Cache::PathCanon::Canonicalize::Fn(&Root).unwrap_or(Root.clone());
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

	// Sky's Target tree as a whole is build output Land controls (product.json,
	// nls bundles, package.json, workbench bundle artifacts). gitlens reads
	// `Sky/Target/product.json` to detect the host product; the workbench reads
	// its own bundled metadata. None of these are user content - allowing the
	// whole `Sky/Target/` subtree mirrors the bundled-extension carve-out
	// above and keeps third-party probes from getting "outside workspace"
	// rejections for files Land itself shipped.
	if ContainsPathSegments(PathToCheck, &["Sky", "Target"])
		|| ContainsPathSegments(PathToCheck, &["Output", "Target"])
		|| ContainsPathSegments(PathToCheck, &["Dependency", "Microsoft", "Dependency", "Editor", "out"])
		|| ContainsPathSegments(
			PathToCheck,
			&["Dependency", "Microsoft", "Dependency", "Editor", "product.json"],
		) {
		return true;
	}

	if let Ok(TempDir) = std::env::var("TMPDIR") {
		let TempPath = PathBuf::from(&TempDir);
		if !TempPath.as_os_str().is_empty() && (Candidate.starts_with(&TempPath) || PathToCheck.starts_with(&TempPath))
		{
			return true;
		}
	}

	// Platform-conventional scratch roots that don't show up in `TMPDIR`
	// on macOS/Linux. Language servers (ruby-lsp, solargraph, jdtls,
	// pyright, …) write port-handoff / reporter / socket files under
	// `/tmp/<tool>/` as a matter of course. `/var/folders/.../T/` IS
	// covered by `TMPDIR` on macOS, but `/tmp` and `/private/tmp` are
	// the ones extensions actually target. Guarding these under the
	// system-trust tier is safe: extensions run inside Cocoon's Node
	// host, which already has unconstrained process-level filesystem
	// access - the sandbox only gates IPC round-trips through Mountain,
	// not the extension's own `fs.writeFileSync`.
	for Root in ["/tmp", "/private/tmp", "/var/tmp"] {
		let RootPath = PathBuf::from(Root);
		if Candidate.starts_with(&RootPath) || PathToCheck.starts_with(&RootPath) {
			return true;
		}
	}

	// Third-party tool state directories extensions commonly probe.
	// GitLens stats `~/.gitkraken/workspaces/workspaces.json` to offer a
	// "Open in GitKraken" menu; copilot-chat stats `~/.copilot/` for
	// cached completions. These live outside Land's namespace but are
	// not user-content either - they're application state from another
	// tool, safe to read/stat.
	if let Ok(Home) = std::env::var("HOME") {
		for Suffix in [".gitkraken", ".gk", ".copilot", ".config/git"] {
			let ToolRoot = PathBuf::from(&Home).join(Suffix);
			if Candidate.starts_with(&ToolRoot) || PathToCheck.starts_with(&ToolRoot) {
				return true;
			}
		}
	}

	// Read-only POSIX OS-info files. Many extensions (csharp, ruby-lsp,
	// rust-analyzer, debug adapters, telemetry SDKs) probe these to
	// branch on distro / kernel for spawning the correct binary. They
	// are world-readable system files - the workspace-folder check
	// rejects them as "outside workspace" but there's no plausible
	// abuse vector. Match by full equality to keep the carve-out tight.
	for SystemFile in [
		"/etc/os-release",
		"/etc/lsb-release",
		"/etc/system-release",
		"/etc/redhat-release",
		"/etc/SuSE-release",
		"/etc/debian_version",
		"/etc/alpine-release",
		"/etc/machine-id",
		"/etc/timezone",
		"/etc/localtime",
		"/proc/version",
		"/proc/cpuinfo",
		"/proc/meminfo",
		"/proc/self/status",
		"/proc/self/cgroup",
	] {
		let SysPath = PathBuf::from(SystemFile);
		if Candidate == SysPath || PathToCheck == SysPath {
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
	Names
		.windows(segments.len())
		.any(|Window| Window.iter().zip(segments.iter()).all(|(A, B)| A == B))
}
