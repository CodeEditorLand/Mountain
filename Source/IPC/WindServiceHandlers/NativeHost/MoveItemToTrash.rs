#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:moveItemToTrash`.
//! Routes deletions to the OS trash bin so they are recoverable.
//! macOS uses Finder via osascript; Linux prefers `gio trash` then `trash`;
//! Windows uses PowerShell Shell.Application. Returns `true` on success.

use serde_json::{Value, json};

pub async fn NativeMoveItemToTrash(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	if Path.is_empty() {
		return Ok(json!(false));
	}
	crate::dev_log!("nativehost", "nativeHost:moveItemToTrash path={}", Path);
	let Moved = {
		#[cfg(target_os = "macos")]
		{
			tokio::process::Command::new("osascript")
				.args([
					"-e",
					&format!(
						"tell application \"Finder\" to delete POSIX file \"{}\"",
						Path.replace('"', "\\\"")
					),
				])
				.status()
				.await
				.map(|S| S.success())
				.unwrap_or(false)
		}
		#[cfg(target_os = "linux")]
		{
			let Gio = tokio::process::Command::new("gio")
				.args(["trash", &Path])
				.status()
				.await
				.map(|S| S.success())
				.unwrap_or(false);
			if Gio {
				true
			} else {
				tokio::process::Command::new("trash")
					.arg(&Path)
					.status()
					.await
					.map(|S| S.success())
					.unwrap_or(false)
			}
		}
		#[cfg(target_os = "windows")]
		{
			let Script = format!(
				"(new-object -comobject Shell.Application).NameSpace(0xA).MoveHere('{}')",
				Path.replace('\'', "''")
			);
			tokio::process::Command::new("powershell.exe")
				.args(["-NoProfile", "-Command", &Script])
				.status()
				.await
				.map(|S| S.success())
				.unwrap_or(false)
		}
		#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
		{
			false
		}
	};
	Ok(json!(Moved))
}
