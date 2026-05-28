//! Pick the system default shell. Unix: `$SHELL`, then probe
//! `/bin/{zsh,bash,sh}`. Windows: PowerShell 7 if installed,
//! else stock Windows PowerShell. Used by Wind's "Open Default
//! Terminal" command and by extensions that spawn unparented
//! shells.

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
	#[cfg(unix)]
	{
		let Shell = std::env::var("SHELL").unwrap_or_else(|_| {
			for Path in &["/bin/zsh", "/bin/bash", "/bin/sh"] {
				if std::path::Path::new(Path).exists() {
					return Path.to_string();
				}
			}

			"/bin/sh".to_string()
		});

		Ok(json!(Shell))
	}

	#[cfg(target_os = "windows")]
	{
		let SystemRoot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());

		let PwshPath = format!("{}\\PowerShell\\7\\pwsh.exe", std::env::var("ProgramFiles").unwrap_or_default());

		if std::path::Path::new(&PwshPath).exists() {
			return Ok(json!(PwshPath));
		}

		Ok(json!(format!(
			"{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
			SystemRoot
		)))
	}

	#[cfg(not(any(unix, target_os = "windows")))]
	{
		Ok(json!("/bin/sh"))
	}
}
