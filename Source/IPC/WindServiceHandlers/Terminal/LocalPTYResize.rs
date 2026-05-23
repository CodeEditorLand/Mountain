#![allow(unused_variables, dead_code, unused_imports)]

//! Wire method: `localPty:resize`.
//! Forwards a resize event to the PTY master (SIGWINCH) via
//! `TerminalProvider::ResizeTerminal`. Accepts either positional
//! `[id, cols, rows]` or object `{ id, cols, rows }` from the workbench.
//!
//! Clamps cols/rows to ≥ 1 - portable-pty crashes the IO thread with
//! "size out of range" on 0×0, which the workbench can emit during
//! pane drag-storms before the requestAnimationFrame settle.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};
use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let (TerminalId, Columns, Rows) = {
		let First = Arguments.first().cloned().unwrap_or(Value::Null);

		if First.is_object() {
			let Id = First.get("id").and_then(|V| V.as_u64()).unwrap_or(0);

			let C = First.get("cols").and_then(|V| V.as_u64()).unwrap_or(80) as u16;

			let R = First.get("rows").and_then(|V| V.as_u64()).unwrap_or(24) as u16;

			(Id, C, R)
		} else {
			let Id = Arguments.first().and_then(|V| V.as_u64()).unwrap_or(0);

			let C = Arguments.get(1).and_then(|V| V.as_u64()).unwrap_or(80) as u16;

			let R = Arguments.get(2).and_then(|V| V.as_u64()).unwrap_or(24) as u16;

			(Id, C, R)
		}
	};

	if TerminalId == 0 {
		return Ok(Value::Null);
	}

	let Columns = if Columns == 0 { 1 } else { Columns };

	let Rows = if Rows == 0 { 1 } else { Rows };

	let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();

	match Provider.ResizeTerminal(TerminalId, Columns, Rows).await {
		Ok(_) => Ok(Value::Null),

		Err(Error) => {
			// Resize on a disposed terminal is a common race during shutdown -
			// the workbench layout pass fires after `exit`, the PTY closes, and
			// the resize call lands on a dropped master. Log at warn, return
			// Null so the workbench's resize loop continues instead of stalling.
			crate::dev_log!(
				"terminal",
				"warn: localPty:resize id={} cols={} rows={} failed: {}",
				TerminalId,
				Columns,
				Rows,
				Error
			);

			Ok(Value::Null)
		},
	}
}
