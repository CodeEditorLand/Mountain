#![allow(non_snake_case)]

//! `nativeHost:quit` - save state and gracefully exit the process.
//! VS Code calls this from `ILifecycleMainService.quit()` on Cmd+Q,
//! File → Quit, and the "Quit" tray item.

use serde_json::Value;
use tauri::AppHandle;

use crate::dev_log;

pub async fn Fn(ApplicationHandle:AppHandle, _Arguments:Vec<Value>) -> Result<Value, String> {
	dev_log!("lifecycle", "nativeHost:quit - exiting cleanly");

	ApplicationHandle.exit(0);

	Ok(Value::Null)
}
