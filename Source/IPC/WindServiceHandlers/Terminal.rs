#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Terminal and local PTY handlers.

use std::{collections::HashMap, sync::Arc};

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

// ============================================================================
// Terminal handlers
// ============================================================================

/// Create a new PTY terminal via TerminalProvider.
pub async fn TerminalCreate(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let Options = Arguments.first().cloned().unwrap_or(Value::Null);
	RunTime
		.Environment
		.CreateTerminal(Options)
		.await
		.map_err(|Error| format!("terminal:create failed: {}", Error))
}

/// Write text to PTY stdin via TerminalProvider.
pub async fn TerminalSendText(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = Arguments
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:sendText requires terminal_id as first argument".to_string())?;
	let Text = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	RunTime
		.Environment
		.SendTextToTerminal(TerminalId, Text)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:sendText failed: {}", Error))
}

/// Dispose a terminal via TerminalProvider.
pub async fn TerminalDispose(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = Arguments
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:dispose requires terminal_id as first argument".to_string())?;

	RunTime
		.Environment
		.DisposeTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:dispose failed: {}", Error))
}

/// Show a terminal in the UI.
pub async fn TerminalShow(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = Arguments.first().and_then(|V| V.as_u64()).unwrap_or(0);
	let PreserveFocus = Arguments.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	RunTime
		.Environment
		.ShowTerminal(TerminalId, PreserveFocus)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:show failed: {}", Error))
}

/// Hide a terminal.
pub async fn TerminalHide(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = Arguments.first().and_then(|V| V.as_u64()).unwrap_or(0);

	RunTime
		.Environment
		.HideTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:hide failed: {}", Error))
}

// ============================================================================
// Local PTY handlers
// ============================================================================

/// Detect available terminal profiles - cross-platform.
pub async fn LocalPTYGetProfiles() -> Result<Value, String> {
	let mut Profiles = Vec::new();

	#[cfg(unix)]
	{
		let DefaultShell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

		// Common Unix shells - macOS, Ubuntu, RHEL, Fedora, Arch, etc.
		let UnixShells = [
			"/bin/zsh",
			"/bin/bash",
			"/bin/sh",
			"/usr/bin/zsh",
			"/usr/bin/bash",
			"/usr/bin/fish",
			"/usr/local/bin/fish",
			"/usr/local/bin/zsh",
			"/usr/local/bin/bash",
			"/bin/dash",     // Ubuntu/Debian default /bin/sh symlink target
			"/usr/bin/ksh",  // KornShell (RHEL, Solaris)
			"/usr/bin/tcsh", // C Shell variant
			"/bin/csh",      // C Shell
			"/usr/bin/pwsh", // PowerShell on Linux/macOS
			"/usr/local/bin/pwsh",
		];

		for Shell in &UnixShells {
			if std::path::Path::new(Shell).exists() {
				let Name = std::path::Path::new(Shell)
					.file_name()
					.and_then(|N| N.to_str())
					.unwrap_or("shell");

				Profiles.push(json!({
					"profileName": Name,
					"path": Shell,
					"isDefault": *Shell == DefaultShell.as_str(),
					"Arguments": [],
					"env": {},
					"icon": "terminal"
				}));
			}
		}

		// Also check /etc/shells for additional entries
		if let Ok(ShellsFile) = std::fs::read_to_string("/etc/shells") {
			for Line in ShellsFile.lines() {
				let Trimmed = Line.trim();
				if Trimmed.starts_with('/') && !Trimmed.starts_with('#') {
					let AlreadyAdded = Profiles.iter().any(|P| P.get("path").and_then(|V| V.as_str()) == Some(Trimmed));
					if !AlreadyAdded && std::path::Path::new(Trimmed).exists() {
						let Name = std::path::Path::new(Trimmed)
							.file_name()
							.and_then(|N| N.to_str())
							.unwrap_or("shell");

						Profiles.push(json!({
							"profileName": Name,
							"path": Trimmed,
							"isDefault": Trimmed == DefaultShell.as_str(),
							"Arguments": [],
							"env": {},
							"icon": "terminal"
						}));
					}
				}
			}
		}
	}

	#[cfg(target_os = "windows")]
	{
		// Windows terminal profiles
		let SystemRoot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
		let ProgramFiles = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
		let LocalAppData =
			std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\Users\\User\\AppData\\Local".to_string());

		let WindowsShells:Vec<(&str, String, Vec<&str>)> = vec![
			(
				"PowerShell",
				format!("{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", SystemRoot),
				vec!["-NoLogo"],
			),
			(
				"PowerShell 7",
				format!("{}\\PowerShell\\7\\pwsh.exe", ProgramFiles),
				vec!["-NoLogo"],
			),
			("Command Prompt", format!("{}\\System32\\cmd.exe", SystemRoot), vec![]),
			(
				"Git Bash",
				format!("{}\\Git\\bin\\bash.exe", ProgramFiles),
				vec!["--login", "-i"],
			),
			(
				"Git Bash (User)",
				format!("{}\\Programs\\Git\\bin\\bash.exe", LocalAppData),
				vec!["--login", "-i"],
			),
			("WSL", format!("{}\\System32\\wsl.exe", SystemRoot), vec![]),
			("MSYS2", "C:\\msys64\\usr\\bin\\bash.exe".to_string(), vec!["--login", "-i"]),
			("Cygwin", "C:\\cygwin64\\bin\\bash.exe".to_string(), vec!["--login", "-i"]),
		];

		let mut IsFirstFound = true;
		for (Name, Path, Args) in &WindowsShells {
			if std::path::Path::new(Path).exists() {
				Profiles.push(json!({
					"profileName": Name,
					"path": Path,
					"isDefault": IsFirstFound,
					"Arguments": Args,
					"env": {},
					"icon": "terminal"
				}));
				IsFirstFound = false;
			}
		}
	}

	Ok(json!(Profiles))
}

/// Get default system shell - cross-platform.
pub async fn LocalPTYGetDefaultShell() -> Result<Value, String> {
	#[cfg(unix)]
	{
		let Shell = std::env::var("SHELL").unwrap_or_else(|_| {
			// Try common fallbacks
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
		// Check for PowerShell 7 first, then Windows PowerShell, then cmd
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

/// Get terminal environment.
pub async fn LocalPTYGetEnvironment() -> Result<Value, String> {
	let Env:HashMap<String, String> = std::env::vars().collect();
	Ok(json!(Env))
}
