#![allow(non_snake_case)]

//! `localPty:detachFromProcess` - signal that the workbench is detaching
//! from a live PTY (e.g. on window close while keeping the process alive).
//!
//! Mountain keeps the PTY running. The PTY output buffer continues to
//! accumulate so that `sky:replay-events` can replay missed data when the
//! window reattaches. Returns `null` unconditionally - the workbench treats
//! any truthy resolve as "detach acknowledged".
//!
//! Wire shape: `Arguments[0]` = id (u64)

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(_RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let TerminalId = match Arguments.first() {
		Some(Value::Number(N)) => N.as_u64().unwrap_or(0),

		Some(Value::Object(Obj)) => Obj.get("id").and_then(Value::as_u64).unwrap_or(0),

		_ => 0,
	};

	dev_log!(
		"terminal",
		"[DetachFromProcess] id={} (PTY kept alive; output buffer accumulating for next attach)",
		TerminalId
	);

	// Mountain intentionally keeps the PTY alive - no action needed.
	// The next `localPty:attachToProcess` or `sky:replay-events` will
	// drain the output buffer back to the renderer.
	Ok(Value::Null)
}
