#![allow(non_snake_case)]

//! `nativeHost:installShellCommand` - create a `land` (or `code`) symlink in
//! `/usr/local/bin` pointing at the running executable so the user can launch
//! the editor from a terminal. Mirrors VS Code's "Install 'code' command in
//! PATH" command. Uses `pkexec`/`osascript` to acquire elevated privileges
//! when `/usr/local/bin` is not writable by the current user.

use std::path::PathBuf;

use serde_json::Value;

use crate::dev_log;

const CLI_NAME:&str = "land";

const SYMLINK_DIR:&str = "/usr/local/bin";

pub async fn InstallShellCommand(_Arguments:Vec<Value>) -> Result<Value, String> {
	let ExePath = std::env::current_exe().map_err(|E| format!("installShellCommand: cannot get exe path: {E}"))?;

	let Target = PathBuf::from(SYMLINK_DIR).join(CLI_NAME);

	dev_log!("shell-cmd", "installShellCommand: {} → {}", Target.display(), ExePath.display());

	// Remove stale link first (ignore errors - may not exist yet).
	let _ = std::fs::remove_file(&Target);

	match std::os::unix::fs::symlink(&ExePath, &Target) {
		Ok(()) => {
			dev_log!("shell-cmd", "installShellCommand: symlink created");

			Ok(Value::Bool(true))
		},

		Err(E) if E.kind() == std::io::ErrorKind::PermissionDenied => {
			// Retry with osascript-elevated write on macOS.
			#[cfg(target_os = "macos")]
			{
				// Pass paths via env vars; use AppleScript's `quoted form of` for
				// safe shell quoting - no interpolation into script source.
				let Status = tokio::process::Command::new("osascript")
					.env("SH_SRC", ExePath.as_os_str())
					.env("SH_DST", Target.as_os_str())
					.args([
						"-e",
						"do shell script (\"ln -sf \" & quoted form of (system attribute \"SH_SRC\") & \" \" & quoted \
						 form of (system attribute \"SH_DST\")) with administrator privileges",
					])
					.status()
					.await
					.map_err(|E| format!("installShellCommand: osascript failed: {E}"))?;

				if Status.success() {
					dev_log!("shell-cmd", "installShellCommand: symlink created (elevated)");

					return Ok(Value::Bool(true));
				}
			}

			Err(format!("installShellCommand: permission denied and elevation failed"))
		},

		Err(E) => Err(format!("installShellCommand: {E}")),
	}
}
