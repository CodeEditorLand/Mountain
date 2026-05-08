#![allow(non_snake_case)]
//! User-scope install destination for VSIX unpacks. Matches the user-scope
//! scan path in `Binary/Extension/ScanPathConfigure.rs` so VSIX-installed
//! extensions are discovered on the next Mountain boot without a sync step.
//!
//! Resolution order (honours Atom V1 `Lodge`):
//!   1. `$Lodge` - explicit per-operator override. Leading `~/` expands against
//!      `$HOME`.
//!   2. `$HOME/.land/extensions` - VS Code-style user-scope default.
//!   3. `./extensions` - fallback when `$HOME` is unavailable.

use std::path::PathBuf;

pub fn UserExtensionDirectory() -> PathBuf {
	if let Ok(Override) = std::env::var("Lodge") {
		if let Some(Stripped) = Override.strip_prefix("~/") {
			if let Some(HomeDirectory) = dirs::home_dir() {
				return HomeDirectory.join(Stripped);
			}
		}

		return PathBuf::from(Override);
	}

	if let Some(HomeDirectory) = dirs::home_dir() {
		return HomeDirectory.join(".land/extensions");
	}

	PathBuf::from("extensions")
}
