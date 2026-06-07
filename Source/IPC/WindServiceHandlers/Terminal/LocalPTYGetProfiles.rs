//! Discover available terminal profiles. Probes every well-
//! known shell location plus `/etc/shells` (Unix) or known
//! Windows install paths. The first existing match flags
//! `isDefault=true`; on Unix the user's `$SHELL` wins.
//!
//! The wire shape matches VS Code's
//! `ITerminalProfileProvider.profileName / path / args /
//! env / icon / isDefault` so Wind's terminal picker renders
//! without reshaping. VS Code's `ITerminalProfile` reads
//! `args` (lowercase); emitting `Arguments` silently mis-parses
//! and the profile dropdown falls back to `$SHELL`.

use serde_json::{Value, json};

use crate::dev_log;

pub async fn Fn() -> Result<Value, String> {
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

	dev_log!("terminal", "[GetProfiles] returning {} profiles", Profiles.len());

	Ok(json!(Profiles))
}
