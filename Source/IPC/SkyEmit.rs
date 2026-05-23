//! # SkyEmit
//!
//! Wrapper over `tauri::Emitter::emit(channel, payload)` that logs every
//! Mountain → Wind/Sky emit under the `sky-emit` DevLog tag. Drop-in
//! replacement for `handle.emit(channel, payload)` at any call site that
//! benefits from per-channel traffic instrumentation.
//!
//! ## When to use
//!
//! Prefer `LogSkyEmit` over bare `emit` for any `sky://...` channel the
//! user might want to audit. The existing ~50+ emit sites remain
//! compatible - convert them incrementally as you touch nearby code.
//!
//! ## Output shape
//!
//! Each successful emit produces:
//!
//! ```
//! [DEV:SKY-EMIT] [SkyEmit] ok channel=sky://tree-view/create bytes=64
//! ```
//!
//! Failures produce:
//!
//! ```
//! [DEV:SKY-EMIT] [SkyEmit] fail channel=sky://… bytes=64 error=<reason>
//! ```
//!
//! ## Tag filtering
//!
//! `Trace=sky-emit tail -f Mountain.dev.log` shows the stream on
//! its own so you can audit exactly which channels are being emitted,
//! in what order, and with what payload size - without re-running or
//! adding ad-hoc prints.


use serde::Serialize;
use tauri::Emitter;

use crate::dev_log;

/// Emit a tagged log line around any `ApplicationHandle::emit`. Returns
/// the same `Result` as the underlying emit so callers using
/// `let _ = …` / `?` / `if let Err(e) = …` keep their existing shape.
pub fn LogSkyEmit<R:tauri::Runtime, P:Serialize + Clone>(
	Handle:&impl Emitter<R>,

	Channel:&str,

	Payload:P,
) -> tauri::Result<()> {
	// Measure the serialized payload size for traffic-volume diagnostics.
	// Silently falls back to 0 on (very unusual) serialize failures -
	// never blocks the emit itself.
	let Bytes = serde_json::to_vec(&Payload).map(|V| V.len()).unwrap_or(0);

	match Handle.emit(Channel, Payload) {
		Ok(()) => {
			dev_log!("sky-emit", "[SkyEmit] ok channel={} bytes={}", Channel, Bytes);

			Ok(())
		},

		Err(Error) => {
			dev_log!("sky-emit", "[SkyEmit] fail channel={} bytes={} error={}", Channel, Bytes, Error);

			Err(Error)
		},
	}
}
