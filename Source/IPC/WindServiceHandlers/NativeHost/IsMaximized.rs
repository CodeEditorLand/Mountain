#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:isMaximized`.
//! Returns true if the `main` webview window is maximized. Missing window
//! returns false (matches VS Code's behaviour on orphaned calls).

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

pub async fn NativeIsMaximized(ApplicationHandle:AppHandle) -> Result<Value, String> {
	let Window = ApplicationHandle.get_webview_window("main");
	if let Some(W) = Window {
		Ok(json!(W.is_maximized().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}
