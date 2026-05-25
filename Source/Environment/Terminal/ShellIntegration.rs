//! Shell integration injection for the integrated terminal.
//!
//! When `LAND_SHELL_INTEGRATION` is not explicitly set to `"0"`, this module
//! finds the appropriate shell integration script (bash, zsh, or fish) in the
//! app resource directory and injects it into the shell's startup sequence
//! so the workbench receives OSC 633 command-tracking sequences.
//!
//! ## OSC 633 sequence meanings
//!
//! | Code | Meaning              |
//! |------|----------------------|
//! | A    | Prompt start         |
//! | B    | Prompt end           |
//! | C    | Command start        |
//! | D;N  | Command end (exit N) |
//! | P;cwd=<path> | Current working directory |
//!
//! ## Injection strategy per shell
//!
//! - **bash**: `--init-file <script>` - replaces `.bashrc`; script sources the
//!   original before setting hooks.
//! - **zsh**: set `ZDOTDIR` to a temp dir whose `.zshrc` sources the script
//!   then `LAND_ORIG_ZDOTDIR/.zshrc`; avoids touching `--rcs`.
//! - **fish**: `--init-command 'source <script>'`
//! - All others: no injection; integration unavailable for that shell.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::dev_log;

/// Describes how a shell integration script should be injected.
pub struct Injection {
	/// Additional environment variables to set before spawning the shell.
	pub EnvVars:Vec<(String, String)>,
	/// Extra arguments to prepend to the shell's argument list.
	pub PrependArgs:Vec<String>,
	/// Extra arguments to append to the shell's argument list.
	pub AppendArgs:Vec<String>,
}

/// Returns the resource-dir path for a named integration script.
fn ScriptPath(AppHandle:&AppHandle, Name:&str) -> Option<PathBuf> {
	let Base = AppHandle.path().resource_dir().ok()?;

	let Candidate = Base.join("scripts/shell-integration").join(Name);

	if Candidate.exists() {
		Some(Candidate)
	} else {
		dev_log!(
			"terminal",
			"[ShellIntegration] script not found at {} (bundled .app only)",
			Candidate.display()
		);
		None
	}
}

/// Returns the shell binary name (lowercase) extracted from a full path.
fn ShellName(ShellPath:&str) -> &str { Path::new(ShellPath).file_name().and_then(|N| N.to_str()).unwrap_or("") }

/// Computes the `Injection` for `shell_path`, or `None` if the shell is
/// unsupported or integration is explicitly disabled via
/// `LAND_SHELL_INTEGRATION=0`.
pub fn Compute(AppHandle:&AppHandle, ShellPath:&str) -> Option<Injection> {
	if std::env::var("LAND_SHELL_INTEGRATION").as_deref() == Ok("0") {
		dev_log!("terminal", "[ShellIntegration] disabled via LAND_SHELL_INTEGRATION=0");
		return None;
	}

	let Shell = ShellName(ShellPath);
	dev_log!("terminal", "[ShellIntegration] shell={} path={}", Shell, ShellPath);

	match Shell {
		"bash" => {
			let Script = ScriptPath(AppHandle, "bash.sh")?;
			dev_log!("terminal", "[ShellIntegration] bash: --init-file {}", Script.display());
			Some(Injection {
				EnvVars:vec![("VSCODE_SHELL_INTEGRATION".into(), "1".into())],
				PrependArgs:Vec::new(),
				AppendArgs:vec!["--init-file".into(), Script.to_string_lossy().into_owned()],
			})
		},

		"zsh" => {
			let Script = ScriptPath(AppHandle, "zsh.zsh")?;
			dev_log!(
				"terminal",
				"[ShellIntegration] zsh: ZDOTDIR injection script={}",
				Script.display()
			);

			// Create a temporary ZDOTDIR containing a .zshrc that sources our
			// script. Preserve the user's original ZDOTDIR so the integration
			// script can re-source their config.
			let TmpDir = std::env::temp_dir().join(format!("land-zsh-integration-{}", std::process::id()));

			if std::fs::create_dir_all(&TmpDir).is_err() {
				return None;
			}

			let OrigZdotDir = std::env::var("ZDOTDIR").unwrap_or_else(|_| std::env::var("HOME").unwrap_or_default());

			// Write a minimal .zshrc that forwards to our integration script.
			let ZshRcContent = format!(
				"export LAND_ORIG_ZDOTDIR=\"{}\"\nexport LAND_SHELL_INTEGRATION_ACTIVE=1\nsource \"{}\"\n",
				OrigZdotDir.replace('"', "\\\""),
				Script.to_string_lossy().replace('"', "\\\""),
			);

			let ZshRcPath = TmpDir.join(".zshrc");

			if std::fs::write(&ZshRcPath, ZshRcContent).is_err() {
				return None;
			}

			Some(Injection {
				EnvVars:vec![
					("ZDOTDIR".into(), TmpDir.to_string_lossy().into_owned()),
					("VSCODE_SHELL_INTEGRATION".into(), "1".into()),
				],
				PrependArgs:Vec::new(),
				AppendArgs:Vec::new(),
			})
		},

		"fish" => {
			let Script = ScriptPath(AppHandle, "fish.fish")?;
			dev_log!(
				"terminal",
				"[ShellIntegration] fish: --init-command source {}",
				Script.display()
			);
			Some(Injection {
				EnvVars:vec![("VSCODE_SHELL_INTEGRATION".into(), "1".into())],
				PrependArgs:Vec::new(),
				AppendArgs:vec![
					"--init-command".into(),
					format!("source \"{}\"", Script.to_string_lossy().replace('"', "\\\"")),
				],
			})
		},

		Other => {
			dev_log!("terminal", "[ShellIntegration] unsupported shell '{}' - no injection", Other);
			None
		},
	}
}
