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

	let WorkSpacePathArgument = CliArgs.iter().find(|Arg| Arg.ends_with(".code-workspace"));

	WorkSpacePathArgument.map(|PathString| PathBuf::from(PathString))
}

/// Check if a workspace argument was provided.
///
/// Returns true if a workspace file path was found in CLI arguments.
pub fn HasWorkspaceArgument() -> bool { Parse().is_some() }
