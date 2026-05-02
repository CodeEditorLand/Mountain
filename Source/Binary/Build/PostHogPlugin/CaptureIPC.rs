#![allow(non_snake_case)]

//! Convenience wrapper for IPC dispatch instrumentation. Emits
//! `mountain:ipc:invoke` with the wire `method` name.

use crate::Binary::Build::PostHogPlugin::{CaptureAllowed, CaptureEvent};

pub fn Fn(Method:&str) {
	if !CaptureAllowed::Fn() {
		return;
	}
	CaptureEvent::Fn("mountain:ipc:invoke", Some(vec![("method", Method)]));
}
