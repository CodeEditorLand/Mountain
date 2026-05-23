//! `nativeHost:reload` - reload the webview without restarting the process.
//! VS Code calls this from `ILifecycleMainService.reload()` for "Reload
//! Window" (Developer menu / Cmd+Shift+P → Reload Window).

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::dev_log;

pub async fn Fn(ApplicationHandle:AppHandle, _Arguments:Vec<Value>) -> Result<Value, String> {
	dev_log!("lifecycle", "nativeHost:reload - reloading webview");

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		Window
			.eval("location.reload()")
			.map_err(|E| format!("reload eval failed: {E}"))?;
	}

	Ok(Value::Null)
}
