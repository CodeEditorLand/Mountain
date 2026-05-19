#![allow(non_snake_case)]

//! `nativeHost:uninstallShellCommand` - remove the `land` symlink from
//! `/usr/local/bin`. Mirrors VS Code's "Uninstall 'code' command from PATH".

use std::path::PathBuf;

use serde_json::Value;

use crate::dev_log;

const CLI_NAME:&str = "land";

const SYMLINK_DIR:&str = "/usr/local/bin";

pub async fn UninstallShellCommand(_Arguments:Vec<Value>) -> Result<Value, String> {
	let Target = PathBuf::from(SYMLINK_DIR).join(CLI_NAME);

	dev_log!("shell-cmd", "uninstallShellCommand: removing {}", Target.display());

	match std::fs::remove_file(&Target) {
		Ok(()) => {
			dev_log!("shell-cmd", "uninstallShellCommand: removed");

			Ok(Value::Bool(true))
		},

		Err(E) if E.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(true)),

		Err(E) if E.kind() == std::io::ErrorKind::PermissionDenied => {
			#[cfg(target_os = "macos")]
			{
				let Script = format!("do shell script \"rm -f '{}'\" with administrator privileges", Target.display());

				let Status = tokio::process::Command::new("osascript")
					.args(["-e", &Script])
					.status()
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
