//! Wire method: `nativeHost:positionWindow`.
//!
//! Moves/resizes the main window to an explicit screen position, used
//! by multi-window restore logic.

use serde_json::Value;
use tauri::{AppHandle, Manager};

pub fn Fn(ApplicationHandle:&AppHandle, Arguments:&[Value]) -> Result<Value, String> {
	if let Some(Rect) = Arguments.first() {
		let X = Rect.get("x").and_then(|V| V.as_i64()).unwrap_or(0) as i32;

		let Y = Rect.get("y").and_then(|V| V.as_i64()).unwrap_or(0) as i32;

		let W = Rect.get("width").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

		let H = Rect.get("height").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

		if let Some(Window) = ApplicationHandle.get_webview_window("main") {
			let _ = Window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x:X, y:Y }));

			if W > 0 && H > 0 {
				let _ = Window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width:W, height:H }));
			}
		}
	}

	Ok(Value::Null)
}
