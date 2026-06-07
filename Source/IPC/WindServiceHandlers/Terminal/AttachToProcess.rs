//! `localPty:attachToProcess` - reconnect the workbench to an existing
//! Mountain-owned PTY after a window reload.
//!
//! VS Code calls this when the workbench learns that a terminal ID from the
//! previous session is already live (via `localPty:getTerminalLayoutInfo` or
//! `localPty:reviveTerminalProcesses`). Instead of spawning a new PTY it
//! attaches to the one that Mountain kept running.
//!
//! Wire shape: `Arguments[0]` = id (u64)
//!
//! Returns `{ id, pid }` on success or `null` when the id is unknown.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let TerminalId = match Arguments.first() {
		Some(Value::Number(N)) => N.as_u64().unwrap_or(0),

		Some(Value::Object(Obj)) => Obj.get("id").and_then(Value::as_u64).unwrap_or(0),

		_ => 0,
	};

	if TerminalId == 0 {
		dev_log!("terminal", "warn: [AttachToProcess] called with id=0, ignoring");

		return Ok(Value::Null);
	}

	let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();

	// Fetch the PID so the workbench can bind its tooltip + debug adapter.
	// If the terminal no longer exists the provider returns None.
	match Provider.GetTerminalProcessId(TerminalId).await {
		Ok(Some(Pid)) => {
			dev_log!("terminal", "[AttachToProcess] attached id={} pid={}", TerminalId, Pid);

			Ok(json!({ "id": TerminalId, "pid": Pid }))
		},

		Ok(None) => {
			dev_log!(
				"terminal",

				"warn: [AttachToProcess] id={} not found in active terminals",

				TerminalId
			);

			Ok(Value::Null)
		},

		Err(Error) => {
			dev_log!("terminal", "warn: [AttachToProcess] id={} error: {}", TerminalId, Error);

			Ok(Value::Null)
		},
	}
}
