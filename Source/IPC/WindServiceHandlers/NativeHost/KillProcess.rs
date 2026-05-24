//! `nativeHost:killProcess` - send SIGKILL (Unix) or TerminateProcess
//! (Windows) to a child process. VS Code uses this to forcibly stop
//! language servers and debug adapters that don't respond to graceful
//! shutdown within their timeout.

use serde_json::Value;

use crate::{IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgU64, dev_log};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Pid = ArgU64(&Arguments, 0) as u32;

	if Pid == 0 {
		return Ok(Value::Null);
	}

	dev_log!("process", "nativeHost:killProcess pid={}", Pid);

	#[cfg(unix)]
	{
		use std::process::Command;

		let _ = Command::new("kill").args(["-9", &Pid.to_string()]).Status();
	}

	#[cfg(windows)]
	{
		use std::process::Command;

		let _ = Command::new("taskkill").args(["/F", "/PID", &Pid.to_string()]).Status();
	}

	Ok(Value::Null)
}
