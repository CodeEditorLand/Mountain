//! # CliParse
//!
//! Parses command-line arguments for workspace configuration.
//!
//! ## RESPONSIBILITIES
//!
//! ### Argument Parsing
//! - Parse CLI arguments
//! - Extract workspace file from arguments
//! - Validate workspace file extension
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Early initialization component in Binary subsystem
//! - Provides workspace configuration from CLI
//!
//! ### Dependencies
//! - std::env: Environment argument access
//!
//! ### Dependents
//! - Fn() main entry point: Uses parsed CLI args
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate workspace paths to prevent directory traversal
//! - Ensure only .code-workspace files are processed
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - CLI parsing is fast, minimal overhead

use std::path::{Path, PathBuf};

/// Parse CLI arguments and extract workspace path.
///
/// Looks for a .code-workspace file argument in the command-line
/// arguments and returns it if found.
///
/// # Returns
///
/// Returns the workspace file path if found, or None.
pub fn Parse() -> Option<PathBuf> {
	let CliArgs:Vec<String> = std::env::args().collect();

	let WorkspacePathArgument = CliArgs.iter().find(|Arg| Arg.ends_with(".code-workspace"));

	WorkspacePathArgument.map(|PathString| PathBuf::from(PathString))
}

/// Check if a workspace argument was provided.
///
/// Returns true if a workspace file path was found in CLI arguments.
pub fn HasWorkspaceArgument() -> bool { Parse().is_some() }

/// Parse workspace folder paths from CLI / env with the following precedence:
///
/// 1. Every `--folder <path>` pair on the command line (repeatable).
/// 2. Any non-flag positional argument that resolves to an existing directory
///    (convention used when the user drags a folder onto the app).
/// 3. `Open` env var (colon-separated on POSIX, `;`-separated on Windows to
///    match the platform's PATH delimiter).
/// 4. The current working directory, if no other source is available AND `Walk`
///    isn't set to `false`.
///
/// Returned paths are canonicalised; non-existent / non-directory entries
/// are dropped with a warning.
pub fn ParseWorkspaceFolders() -> Vec<PathBuf> {
	let mut Collected:Vec<PathBuf> = Vec::new();

	let CliArgs:Vec<String> = std::env::args().skip(1).collect();

	let mut Index = 0;

	while Index < CliArgs.len() {
		let Argument = &CliArgs[Index];

		if (Argument == "--folder" || Argument == "-F") && Index + 1 < CliArgs.len() {
			Collected.push(PathBuf::from(&CliArgs[Index + 1]));

			Index += 2;

			continue;
		}

		// Positional existing-directory argument. Skip flags + workspace files.
		if !Argument.starts_with('-') && !Argument.ends_with(".code-workspace") {
			let Candidate = PathBuf::from(Argument);

			if Candidate.is_dir() {
				Collected.push(Candidate);
			}
		}

		Index += 1;
	}

	if Collected.is_empty() {
		if let Ok(EnvValue) = std::env::var("Open") {
			let Separator = if cfg!(windows) { ';' } else { ':' };

			for Piece in EnvValue.split(Separator) {
				let Piece = Piece.trim();

				if Piece.is_empty() {
					continue;
				}

				Collected.push(PathBuf::from(Piece));
			}
		}
	}

	// Recently-opened fallback. The webview's initial URL is built from
	// `~/.land/workspaces/RecentlyOpened.json`'s top entry (see
	// `Binary/Build/WindowBuild.rs::BuildInitialUrl`), so when the user
	// picks a folder from the recent-list / "Open Folder" UI, the URL
	// loads with `?folder=<their-pick>` but Mountain's boot-time seeder
	// previously fell straight through to CWD walk-up. Result: webview
	// title says "Mountain" but Cocoon's init payload ships "FIDDEE",
	// vscode.git scans the wrong root, SCM panel reports zeros while
	// `git status` in the actual folder shows uncommitted changes.
	//
	// Probe the same source of truth as `BuildInitialUrl` so the seeded
	// workspace and the loaded URL agree. Slot this between env/CLI
	// (explicit user intent) and CWD walk-up (last resort).
	if Collected.is_empty() {
		if let Some(Path) = ResolveRecentlyOpenedTopFolder() {
			Collected.push(Path);
		}
	}

	if Collected.is_empty() {
		// CWD-autoload: ON in every profile. The earlier
		// debug-only default left release `.app` launches via Finder /
		// `open` with no workspace folder (cwd=`/` after `open`,
		// `RecentlyOpened.json` may be empty/stale → tree-view empty,
		// `vscode.workspace.findFiles` returns nothing, SCM panel can't
		// find a repo). Override with `Walk=0` to keep the stock
		// VS Code "File → Open Folder" UX.
		//
		// Safety: when cwd is the filesystem root `/` (always the case
		// when launched via `open` from Finder/Dock), the walk-up
		// returns `/` itself which would scan the entire disk. Skip
		// that and fall through to the HOME fallback below.
		let AutoloadCwd = std::env::var("Walk")
			.map(|Value| matches!(Value.as_str(), "1" | "true" | "yes" | "on"))
			.unwrap_or(true);

		if AutoloadCwd && let Ok(Cwd) = std::env::current_dir() {
			let IsFilesystemRoot = Cwd.parent().is_none();

			if !IsFilesystemRoot {
				Collected.push(WalkUpToProjectRoot(&Cwd));
			}
		}
	}

	// Final fallback: HOME directory. Reached when the binary was
	// launched via Finder / `open` (cwd=`/`), there's no
	// `RecentlyOpened.json` entry, and no `Open=` env. A workspace
	// rooted at `$HOME` lets the tree view list the user's actual
	// directories instead of showing an empty "no folder open" panel.
	// The user can still pick a more specific folder via "File → Open
	// Folder"; this just ensures something visible is there on first
	// launch.
	if Collected.is_empty()
		&& let Some(Home) = dirs::home_dir()
		&& Home.is_dir()
	{
		Collected.push(Home);
	}

	Collected
		.into_iter()
		.filter_map(|Path| {
			if !Path.is_dir() {
				eprintln!("[LandFix:WsInit] Skipping non-directory workspace folder: {}", Path.display());

				return None;
			}

			Path.canonicalize().ok().or(Some(Path))
		})
		.collect()
}

/// Read `~/.land/workspaces/RecentlyOpened.json`'s top workspace entry and
/// resolve it to a directory path. Mirrors the probe used by
/// `Binary/Build/WindowBuild.rs::BuildInitialUrl` so the boot-seeded
/// workspace folder agrees with the URL the webview actually loads. Returns
/// `None` when the file is missing/malformed, the entry has no resolvable
/// path, the path doesn't exist on disk, or it isn't a directory.
fn ResolveRecentlyOpenedTopFolder() -> Option<PathBuf> {
	use crate::IPC::WindServiceHandlers::Utilities::RecentlyOpened::Read::Fn as ReadRecentlyOpened;

	let Recent = ReadRecentlyOpened().ok()?;

	let Workspaces = Recent.get("workspaces").and_then(|V| V.as_array())?;

	// Same priority order as BuildInitialUrl: own writer's `uri`,
	// VS Code's `folderUri`/`folderUri.path`, then `workspace.configPath.path`.
	let Probe = |Entry:&serde_json::Value| -> Option<String> {
		if let Some(Uri) = Entry.get("uri").and_then(|V| V.as_str()) {
			return Some(Uri.to_string());
		}

		if let Some(Uri) = Entry.get("folderUri").and_then(|V| V.as_str()) {
			return Some(Uri.to_string());
		}

		if let Some(Path) = Entry.get("folderUri").and_then(|V| V.get("path")).and_then(|V| V.as_str()) {
			return Some(Path.to_string());
		}

		if let Some(Path) = Entry
			.get("workspace")
			.and_then(|V| V.get("configPath"))
			.and_then(|V| V.get("path"))
			.and_then(|V| V.as_str())
		{
			return Some(Path.to_string());
		}

		None
	};

	let Raw = Workspaces.iter().find_map(Probe)?;

	let Normalised = Raw.strip_prefix("file://").unwrap_or(Raw.as_str()).to_string();

	let Candidate = PathBuf::from(&Normalised);

	if Candidate.is_dir() { Some(Candidate) } else { None }
}

/// Walk up from `Start` looking for a project-root marker (`Cargo.toml`,
/// `package.json`, `.git`, `pyproject.toml`, `go.mod`, `pnpm-workspace.yaml`).
/// Returns the first ancestor that owns one. Falls back to `Start` itself
/// when nothing matches before the filesystem root.
///
/// Why: when a developer launches the binary from a `Target/debug/` build
/// directory, `current_dir()` is the build folder, which has no source
/// files. `vscode.workspace.findFiles('**/*')` walks it and returns
/// nothing; the SCM panel can't find a repo; tree-views render empty.
/// Walking up to the nearest project root mirrors what every
/// `git`/`cargo`/`npm` CLI does and gives extensions a workspace folder
/// they can actually scan.
fn WalkUpToProjectRoot(Start:&Path) -> PathBuf {
	const Markers:&[&str] = &[
		"Cargo.toml",
		"package.json",
		".git",
		"pyproject.toml",
		"go.mod",
		"pnpm-workspace.yaml",
		"deno.json",
		"deno.jsonc",
	];

	let mut Cursor:&Path = Start;

	loop {
		for Marker in Markers {
			if Cursor.join(Marker).exists() {
				return Cursor.to_path_buf();
			}
		}

		match Cursor.parent() {
			Some(Parent) if Parent != Cursor => Cursor = Parent,

			_ => break,
		}
	}

	Start.to_path_buf()
}
