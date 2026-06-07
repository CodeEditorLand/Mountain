//! Revive serialised terminal processes after a window reload.
//!
//! VS Code calls `localPty:reviveTerminalProcesses` with the array previously
//! returned by `localPty:serializeTerminalState`. Each entry describes a shell
//! that was running before the reload; Mountain respawns each one and emits a
//! `sky://terminal/create` event so the xterm panel re-binds.
//!
//! ## Wire shape (Arguments\[0\])
//! ```json
//! [
//!   {
//!     "id": 1,
//!     "shellLaunchConfig": { "executable": "/bin/zsh", "args": [], "cwd": "/Users/..." },
//!     "processDetails":    { "cwd": "/Users/...", "pid": 1234, "title": "zsh" }
//!   }
//! ]
//! ```
//!
//! Arguments\[1\] is the locale string used for date formatting in VS Code's
//! UI; Mountain ignores it.
//!
//! ## Behaviour
//! - Each entry is forwarded to `TerminalCreate` with `{ shellPath, cwd, name
//!   }`.
//! - The newly allocated terminal ID (assigned by Mountain's atomic counter) is
//!   returned alongside the requested ID so the workbench can remap its
//!   internal `_ptys` table.
//! - Entries whose `shellLaunchConfig.executable` is empty are skipped to avoid
//!   spawning a headless PTY that would immediately exit.

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let States:Vec<Value> = match Arguments.first() {
		Some(Value::Array(Array)) => Array.clone(),

		Some(Other) => {
			dev_log!(
				"terminal",

				"warn: [ReviveTerminalProcesses] unexpected argument shape: {:?}",

				Other
			);

			return Ok(Value::Null);
		},

		None => return Ok(Value::Null),
	};

	if States.is_empty() {
		return Ok(Value::Null);
	}

	dev_log!("terminal", "[ReviveTerminalProcesses] reviving {} terminals", States.len());

	for State in &States {
		let Config = State.get("shellLaunchConfig").cloned().unwrap_or(Value::Null);

		let Executable = Config.get("executable").and_then(Value::as_str).unwrap_or("").to_string();

		if Executable.is_empty() {
			dev_log!(
				"terminal",

				"warn: [ReviveTerminalProcesses] skipping entry with empty executable"
			);

			continue;
		}

		let Cwd = Config
			.get("cwd")
			.and_then(Value::as_str)
			.or_else(|| State.get("processDetails").and_then(|D| D.get("cwd")).and_then(Value::as_str))
			.unwrap_or("")
			.to_string();

		let Name = Config
			.get("name")
			.and_then(Value::as_str)
			.or_else(|| State.get("processDetails").and_then(|D| D.get("title")).and_then(Value::as_str))
			.unwrap_or("terminal")
			.to_string();

		let ShellArgs:Vec<Value> = Config.get("args").and_then(Value::as_array).cloned().unwrap_or_default();

		let Options = json!({
			"shellPath": Executable,
			"shellArgs": ShellArgs,
			"cwd":       Cwd,
			"name":      Name,
		});

		match RunTime.Environment.CreateTerminal(Options).await {
			Ok(Response) => {
				let NewId = Response.get("id").and_then(Value::as_u64).unwrap_or(0);

				dev_log!("terminal", "[ReviveTerminalProcesses] revived terminal new_id={}", NewId);
			},

			Err(Error) => {
				dev_log!(
					"terminal",

					"warn: [ReviveTerminalProcesses] failed to revive terminal: {}",

					Error
				);
			},
		}
	}

	Ok(Value::Null)
}
