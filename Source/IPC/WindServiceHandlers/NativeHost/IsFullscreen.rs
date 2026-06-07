//! Wire method: `nativeHost:isFullscreen`.
//! Returns true if the `main` webview window is fullscreen. Missing window
//! returns false - this is a read-only probe and should not error.

use serde_json::{Value, json};

use tauri::{AppHandle, Manager};

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {

	let Window = ApplicationHandle.get_webview_window("main");

	if let Some(W) = Window {
		Ok(json!(W.is_fullscreen().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}
