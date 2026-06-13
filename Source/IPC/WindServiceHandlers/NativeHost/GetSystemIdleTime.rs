//! Wire method: `nativeHost:getSystemIdleTime`.
//!
//! Queries CGEventSource for last keyboard/mouse event on macOS
//! and returns the idle time in milliseconds.

use serde_json::{Value, json};

pub fn Fn() -> Result<Value, String> {
	#[cfg(target_os = "macos")]
	{
		use std::ffi::c_double;

		unsafe extern {
			fn CGEventSourceSecondsSinceLastEventType(eventSourceState:i32, eventType:u32) -> c_double;
		}

		const kCGEventSourceStateHIDSystemState:i32 = 1;

		const kCGEventKeyDown:u32 = 10;

		const kCGEventLeftMouseDown:u32 = 1;

		let Idle = unsafe {
			let Key = CGEventSourceSecondsSinceLastEventType(kCGEventSourceStateHIDSystemState, kCGEventKeyDown);

			let Mouse =
				CGEventSourceSecondsSinceLastEventType(kCGEventSourceStateHIDSystemState, kCGEventLeftMouseDown);

			Key.min(Mouse)
		};

		Ok(json!((Idle * 1000.0) as u64))
	}

	#[cfg(not(target_os = "macos"))]
	Ok(json!(0))
}
