#![allow(non_snake_case)]

//! Convenience wrapper that emits an error event under the
//! `mountain:error` name with `error_tag` + `error_message`.

use crate::Binary::Build::PostHogPlugin::{CaptureAllowed, CaptureEvent};

pub fn Fn(Tag:&str, Message:&str) {
	if !CaptureAllowed::Fn() {
		return;
	}
	CaptureEvent::Fn("mountain:error", Some(vec![("error_tag", Tag), ("error_message", Message)]));
}
