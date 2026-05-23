
//! `nativeHost:killProcess` - send SIGKILL (Unix) or TerminateProcess
//! (Windows) to a child process. VS Code uses this to forcibly stop
//! language servers and debug adapters that don't respond to graceful
//! shutdown within their timeout.

use serde_json::Value;

use crate::dev_log;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Pid = Arguments.first().and_then(Value::as_u64).unwrap_or(0) as u32;

	if Pid == 0 {
		return Ok(Value::Null);
	}

	dev_log!("process", "nativeHost:killProcess pid={}", Pid);

	#[cfg(unix)]
	{
		use std::process::Command;

		let _ = Command::new("kill").args(["-9", &Pid.to_string()]).status();
	}

	#[cfg(windows)]
	{
		use std::process::Command;

		let _ = Command::new("taskkill").args(["/F", "/PID", &Pid.to_string()]).status();
	}

	Ok(Value::Null)
}
