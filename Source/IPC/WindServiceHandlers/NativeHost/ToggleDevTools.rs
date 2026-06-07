//! `nativeHost:toggleDevTools` - open the inspector if closed, close it if
//! open. Used by the Help → Toggle Developer Tools menu item.

use serde_json::Value;

use tauri::{AppHandle, Manager};

use crate::dev_log;

pub async fn Fn(ApplicationHandle:AppHandle, _Arguments:Vec<Value>) -> Result<Value, String> {

	dev_log!("devtools", "nativeHost:toggleDevTools");

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		if Window.is_devtools_open() {
			Window.close_devtools();
		} else {
			Window.open_devtools();
		}
	}

	Ok(Value::Null)
}
