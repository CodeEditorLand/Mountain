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

use std::path::PathBuf;

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
/// 3. `LAND_WORKSPACE_FOLDER` env var (colon-separated on POSIX, `;`-separated
///    on Windows to match the platform's PATH delimiter).
/// 4. The current working directory, if no other source is available AND
///    `LAND_AUTOLOAD_CWD` isn't set to `false`.
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
		if let Ok(EnvValue) = std::env::var("LAND_WORKSPACE_FOLDER") {
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

	if Collected.is_empty() {
		// CWD-autoload is OPT-IN (set `LAND_AUTOLOAD_CWD=1`). VS Code's UX
		// convention is to open an empty window when no folder is given and
		// let the user pick via File → Open Folder; silently seeding CWD
		// caused the TypeScript extension's workspace scan to walk the
		// entire monorepo (node_modules included) during boot, stalling
		// the UI for minutes.
		let AutoloadCwd = std::env::var("LAND_AUTOLOAD_CWD")
			.map(|Value| Value == "1" || Value == "true")
			.unwrap_or(false);
		if AutoloadCwd {
			if let Ok(Cwd) = std::env::current_dir() {
				Collected.push(Cwd);
			}
		}
	}

	Collected
		.into_iter()
		.filter_map(|Path| {
			if !Path.is_dir() {
				eprintln!(
					"[LandFix:WsInit] Skipping non-directory workspace folder: {}",
					Path.display()
				);
				return None;
			}
			Path.canonicalize().ok().or(Some(Path))
		})
		.collect()
}
