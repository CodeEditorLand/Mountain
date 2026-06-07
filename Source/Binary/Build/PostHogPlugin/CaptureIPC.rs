//! Convenience wrapper for IPC dispatch instrumentation. Emits
//! `land:mountain:ipc:invoke` with the wire `method` name. Pair with
//! `CaptureHandler::Fn` to also stamp `land:mountain:handler:complete`
//! with `duration_ms` once the handler returns.

use crate::Binary::Build::PostHogPlugin::{CaptureAllowed, CaptureEvent};

pub fn Fn(Method:&str) {
	if !CaptureAllowed::Fn() {
		return;
	}

	CaptureEvent::Fn("land:mountain:ipc:invoke", Some(vec![("method", Method)]));
}
