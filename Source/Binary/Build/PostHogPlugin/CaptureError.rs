#![allow(non_snake_case)]

//! Convenience wrapper that emits an error event under the
//! `land:mountain:error` name with `error_tag` + `error_message`. The
//! Errors & Reliability dashboard rolls these up alongside Cocoon /
//! Sky errors via a single `event LIKE 'land:%:error'` clause.

use crate::Binary::Build::PostHogPlugin::{CaptureAllowed, CaptureEvent};

pub fn Fn(Tag:&str, Message:&str) {
	if !CaptureAllowed::Fn() {
		return;
	}
	CaptureEvent::Fn(
		"land:mountain:error",
		Some(vec![("error_tag", Tag), ("error_message", Message)]),
	);
}
