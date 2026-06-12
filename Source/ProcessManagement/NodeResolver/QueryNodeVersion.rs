//! Run `node --version` on the resolved binary and return its reported
//! version string (e.g. `v24.8.0`). Returns `None` when the binary cannot be
//! spawned (bare `node` fallback under a misconfigured PATH) or when it exits
//! non-zero. No timeout - `node --version` never blocks.

use std::path::Path;

/// Public entry point for this module.
pub fn Fn(NodePath:&Path) -> Option<String> {
	let Output = std::process::Command::new(NodePath).arg("--version").output().ok()?;

	if !Output.status.success() {
		return None;
	}

	let Reported = String::from_utf8(Output.stdout).ok()?.trim().to_string();

	if Reported.is_empty() { None } else { Some(Reported) }
}
