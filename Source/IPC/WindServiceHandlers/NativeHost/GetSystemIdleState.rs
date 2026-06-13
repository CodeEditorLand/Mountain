//! Wire method: `nativeHost:getSystemIdleState`.
//!
//! Queries CGEventSource for last keyboard/mouse event on macOS.
//! Returns "idle" when inactive > 60s, "active" otherwise. Falls back
//! to "unknown" on non-macOS platforms.

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

		if Idle > 60.0 { Ok(json!("idle")) } else { Ok(json!("active")) }
	}

	#[cfg(not(target_os = "macos"))]
	Ok(json!("unknown"))
}
