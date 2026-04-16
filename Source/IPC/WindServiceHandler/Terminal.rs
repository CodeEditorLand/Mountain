#![allow(non_snake_case)]

//! Terminal domain handlers for Wind IPC.

use std::{collections::HashMap, sync::Arc};

use serde_json::{Value, json};

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Create a new PTY terminal via TerminalProvider.
pub async fn handle_terminal_create(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Options = Args.first().cloned().unwrap_or(Value::Null);
	Runtime
		.Environment
		.CreateTerminal(Options)
		.await
		.map_err(|Error| format!("terminal:create failed: {}", Error))
}

/// Write text to PTY stdin via TerminalProvider.
pub async fn handle_terminal_send_text(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let TerminalId = Args
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:sendText requires terminal_id as first argument".to_string())?;
	let Text = Args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	Runtime
		.Environment
		.SendTextToTerminal(TerminalId, Text)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:sendText failed: {}", Error))
}

/// Dispose a terminal via TerminalProvider.
pub async fn handle_terminal_dispose(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let TerminalId = Args
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:dispose requires terminal_id as first argument".to_string())?;

	Runtime
		.Environment
		.DisposeTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:dispose failed: {}", Error))
}

/// Show a terminal in the UI.
pub async fn handle_terminal_show(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let TerminalId = Args.first().and_then(|V| V.as_u64()).unwrap_or(0);
	let PreserveFocus = Args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	Runtime
		.Environment
		.ShowTerminal(TerminalId, PreserveFocus)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:show failed: {}", Error))
}

/// Hide a terminal.
pub async fn handle_terminal_hide(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let TerminalId = Args.first().and_then(|V| V.as_u64()).unwrap_or(0);

	Runtime
		.Environment
		.HideTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:hide failed: {}", Error))
}

// ============================================================================
// Local PTY handlers
// ============================================================================

/// Detect available terminal profiles - cross-platform
pub async fn handle_local_pty_get_profiles() -> Result<Value, String> {
	let mut Profiles = Vec::new();

	#[cfg(unix)]
	{
		let DefaultShell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

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
			"/bin/dash",
			"/usr/bin/ksh",
			"/usr/bin/tcsh",
			"/bin/csh",
			"/usr/bin/pwsh",
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
					"args": [],
					"env": {},
					"icon": "terminal"
				}));
			}
		}

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
							"args": [],
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
					"args": Args,
					"env": {},
					"icon": "terminal"
				}));
				IsFirstFound = false;
			}
		}
	}

	Ok(json!(Profiles))
}

/// Get default system shell - cross-platform
pub async fn handle_local_pty_get_default_shell() -> Result<Value, String> {
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

/// Get terminal environment
pub async fn handle_local_pty_get_environment() -> Result<Value, String> {
	let Env:HashMap<String, String> = std::env::vars().collect();
	Ok(json!(Env))
}
