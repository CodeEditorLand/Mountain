//! `nativeHost:exit` - exit with an explicit code.
//! VS Code calls this from `NativeHostMainService.exit(code)` when an
//! extension or the workbench requests a non-zero exit (crash reporter,
//! restart-on-crash sentinel, etc.).

use serde_json::Value;
use tauri::AppHandle;

use crate::{IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgI64, dev_log};

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Code = ArgI64(&Arguments, 0) as i32;

	dev_log!("lifecycle", "nativeHost:exit code={}", Code);

	ApplicationHandle.exit(Code);

	Ok(Value::Null)
}
