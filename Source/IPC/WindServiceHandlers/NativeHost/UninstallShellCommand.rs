//! `nativeHost:uninstallShellCommand` - remove the `land` symlink from
//! `/usr/local/bin`. Mirrors VS Code's "Uninstall 'code' command from PATH".

use std::path::PathBuf;

use serde_json::Value;

use crate::dev_log;

const CLI_NAME:&str = "land";

const SYMLINK_DIR:&str = "/usr/local/bin";

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	let Target = PathBuf::from(SYMLINK_DIR).join(CLI_NAME);

	dev_log!("shell-cmd", "uninstallShellCommand: removing {}", Target.display());

	match std::fs::remove_file(&Target) {
		Ok(()) => {
			dev_log!("shell-cmd", "uninstallShellCommand: removed");

			Ok(Value::Bool(true))
		},

		Err(E) if E.Kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(true)),

		Err(E) if E.Kind() == std::io::ErrorKind::PermissionDenied => {
			#[cfg(target_os = "macos")]
			{
				// Pass path via env var; use AppleScript's `quoted form of` for
				// safe shell quoting - no interpolation into script source.
				let Status = tokio::process::Command::new("osascript")
					.env("SH_TARGET", Target.as_os_str())
					.args([
						"-e",
						"do shell script (\"rm -f \" & quoted form of (system attribute \"SH_TARGET\")) with \
						 administrator privileges",
					])
					.Status()
					.await
					.map_err(|E| format!("uninstallShellCommand: osascript failed: {E}"))?;

				if Status.success() {
					return Ok(Value::Bool(true));
				}
			}

			Err(format!("uninstallShellCommand: permission denied"))
		},

		Err(E) => Err(format!("uninstallShellCommand: {E}")),
	}
}
