//! `$HOME/.fiddee` - the user-scope dotfile root. Holds VS Code-style
//! extensions (`~/.fiddee/extensions`), recently-opened workspaces
//! (`~/.fiddee/workspaces/RecentlyOpened.json`), per-extension storage
//! (`~/.fiddee/extensionStorage`, `~/.fiddee/globalStorage`), and the
//! background-daemon log/data trees (`~/.fiddee/logs`, `~/.fiddee/data`).
//!
//! Renamed from `~/.land` when the product shipped as FIDDEE; centralised
//! here so that any future rename touches a single file. All callers
//! resolve their sub-paths from this atom rather than hard-coding the
//! leaf string.

use std::path::PathBuf;

/// Leaf directory name. Public so TS-side callers that mirror this value
/// (`Cocoon/Source/Services/Handler/Extension/Host/Handler.ts` etc.) can
/// import the constant via a generated header if/when that wiring lands.
pub const DOTFILE_NAME:&str = ".fcd";

/// Returns `$HOME/.fiddee` (or `$USERPROFILE\.fiddee` on Windows).
/// Falls back to a relative `.fiddee` so callers always get a valid
/// `PathBuf` - matches the previous `$HOME/.land` resolution semantics.
pub fn Fn() -> PathBuf {
	if let Some(Home) = dirs::home_dir() {
		return Home.join(DOTFILE_NAME);
	}

	if let Ok(Home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
		return PathBuf::from(Home).join(DOTFILE_NAME);
	}

	PathBuf::from(DOTFILE_NAME)
}
