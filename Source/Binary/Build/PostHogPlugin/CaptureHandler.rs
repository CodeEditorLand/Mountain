#![allow(non_snake_case)]

//! Capture a `land:mountain:handler:complete` event for one IPC handler
//! invocation. `Feature` is the Mountain-side route key (e.g.
//! `file:read`, `extensions:getInstalled`); `DurationMs` measures the
//! handler body only (Tauri-frame overhead excluded); `Ok` reports
//! whether the handler returned `Ok(_)`.
//!
//! The Feature Parity dashboard pivots `Feature` to compare Mountain
//! (Rust) vs Cocoon (Node) handler latency for migrated routes.

use crate::Binary::Build::PostHogPlugin::{CaptureAllowed, CaptureEvent};

pub fn Fn(Feature:&str, DurationMs:u64, Successful:bool) {
	if !CaptureAllowed::Fn() {
		return;
	}

	let DurationString = format!("{}", DurationMs);

	let OkString = if Successful { "true" } else { "false" };

	CaptureEvent::Fn(
		"land:mountain:handler:complete",
		Some(vec![("feature", Feature), ("duration_ms", &DurationString), ("ok", OkString)]),
	);
}
