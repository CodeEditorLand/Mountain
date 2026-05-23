
//! Serialise all active terminals to the `ISerializedTerminalState[]` shape
//! that VS Code's `ILocalPtyService.serializeTerminalProcesses` contract
//! requires.
//!
//! VS Code calls this immediately before a window reload to snapshot running
//! PTY state. The result is written to storage and later passed back via
//! `localPty:reviveTerminalProcesses` so the workbench can restore the panel
//! without the user losing their shell sessions.
//!
//! ## Output shape per terminal
//! ```json
//! {
//!   "id": 1,
//!   "shellLaunchConfig": { "name": "zsh", "executable": "/bin/zsh", "args": [] },
//!   "processDetails":    { "cwd": "/...", "pid": 1234, "title": "zsh" },
//!   "orphanQuestionReply": false,
//!   "replayEvent":        { "events": [] },
//!   "timestamp":          1716134400000
//! }
//! ```
//!
//! Runtime handles (`PTYMaster`, `PTYInputTransmitter`, task `JoinHandle`s)
//! are `#[serde(skip)]` in `TerminalStateDTO` and are intentionally absent
//! from the wire payload - only the configuration fields needed to respawn
//! the shell are serialised.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Terminals = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Terminals
		.ActiveTerminals
		.lock()
		.map_err(|Error| format!("SerializeTerminalState: lock poisoned: {}", Error))?;

	let NowMs = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map(|D| D.as_millis() as u64)
		.unwrap_or(0);

	let Serialized:Vec<Value> = Terminals
		.iter()
		.filter_map(|(TerminalId, ArcState)| {
			let State = ArcState.lock().ok()?;

			let Cwd = State.GetWorkingDirectory();

			let Pid = State.OSProcessIdentifier.unwrap_or(0) as u64;

			Some(json!({
				"id": TerminalId,
				"shellLaunchConfig": {
					"name":       State.Name,
					"executable": State.ShellPath,
					"args":       State.ShellArguments,
					"cwd":        Cwd,
				},
				"processDetails": {
					"cwd":   Cwd,
					"pid":   Pid,
					"title": State.Name,
				},
				// False means the orphan-question dialog was NOT shown;
				// revived terminals start fresh without a stale prompt.
				"orphanQuestionReply": false,
				// Empty replay - the xterm buffer will be restored from the
				// output replay buffer separately via `sky:replay-events`.
				"replayEvent": { "events": [] },
				"timestamp": NowMs,
			}))
		})
		.collect();

	Ok(Value::Array(Serialized))
}
