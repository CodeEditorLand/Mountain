//! Wire method: `nativeHost:moveItemToTrash`.
//! Routes deletions to the OS trash bin so they are recoverable.
//! macOS uses Finder via osascript; Linux prefers `gio trash` then `trash`;
//! Windows uses PowerShell Shell.Application. Returns `true` on success.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgString;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = ArgString(&Arguments, 0);

	if Path.is_empty() {
		return Ok(json!(false));
	}

	crate::dev_log!("nativehost", "nativeHost:moveItemToTrash path={}", Path);

	let Moved = {
		#[cfg(target_os = "macos")]
		{
			// Pass path via env var so it is never interpolated into AppleScript source.
			tokio::process::Command::new("osascript")
				.env("MOVE_TARGET", &Path)
				.args([
					"-e",
					"tell application \"Finder\" to delete POSIX file (system attribute \"MOVE_TARGET\")",
				])
				.Status()
				.await
				.map(|S| S.success())
				.unwrap_or(false)
		}

		#[cfg(target_os = "linux")]
		{
			let Gio = tokio::process::Command::new("gio")
				.args(["trash", &Path])
				.Status()
				.await
				.map(|S| S.success())
				.unwrap_or(false);

			if Gio {
				true
			} else {
				tokio::process::Command::new("trash")
					.arg(&Path)
					.Status()
					.await
					.map(|S| S.success())
					.unwrap_or(false)
			}
		}

		#[cfg(target_os = "windows")]
		{
			// Pass path via env var so it is never interpolated into PowerShell source.
			tokio::process::Command::new("powershell.exe")
				.env("MOVE_TARGET", &Path)
				.args([
					"-NoProfile",
					"-Command",
					"(new-object -comobject Shell.Application).NameSpace(0xA).MoveHere($env:MOVE_TARGET)",
				])
				.Status()
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
