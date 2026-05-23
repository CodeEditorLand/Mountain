//! `localGit:cancel(operationId)` - SIGTERM (Unix) or
//! `taskkill /T /F` (Windows) the pid stashed for
//! `OperationId`. Silent no-op when the id is unknown so
//! late-arriving cancels for already-finished operations
//! don't spam errors.

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::{Git::Shared::TakePid::Fn as TakePid, Utilities::JsonValueHelpers::arg_string},
	dev_log,
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let OperationId = arg_string(&Arguments, 0);

	if let Some(Pid) = TakePid(&OperationId) {
		dev_log!("git", "[Git] cancel op={} pid={}", OperationId, Pid);

		#[cfg(unix)]
		{
			let _ = std::process::Command::new("kill").args(["-TERM", &Pid.to_string()]).output();
		}

		#[cfg(windows)]
		{
			let _ = std::process::Command::new("taskkill")
				.args(["/PID", &Pid.to_string(), "/T", "/F"])
				.output();
		}
	} else {
		dev_log!("git", "[Git] cancel op={} pid=<unknown>", OperationId);
	}

	Ok(Value::Null)
}
