#![allow(non_snake_case)]

//! `nativeHost:relaunch` - restart the process with the same argv.
//! VS Code calls this from `ILifecycleMainService.relaunch()` when an
//! extension update is applied, the user picks "Restart to Apply", or
//! the workbench triggers a self-restart after a crash.

use serde_json::Value;
use tauri::AppHandle;

use crate::dev_log;

pub async fn Fn(ApplicationHandle:AppHandle, _Arguments:Vec<Value>) -> Result<Value, String> {
	dev_log!("lifecycle", "nativeHost:relaunch - restarting process");

	// restart() calls std::process::exit internally - return type is `!`
	// which coerces to Result<Value, String>, so no Ok() needed.
	ApplicationHandle.restart()
}
