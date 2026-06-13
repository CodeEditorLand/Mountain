//! Wire method: `nativeHost:setMinimumSize`.
//!
//! Enforces a minimum window size so the workbench never collapses
//! to a 1×1 pixel frame. Defaults to 400×300.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_u64_or;

pub fn Fn(ApplicationHandle:&AppHandle, Arguments:&[Value]) -> Result<Value, String> {
	let Width = arg_u64_or(Arguments, 0, 400) as u32;

	let Height = arg_u64_or(Arguments, 1, 300) as u32;

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let _ = Window.set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize { width:Width, height:Height })));
	}

	Ok(Value::Null)
}
