//! Wrap `app_handle.emit(name, payload)` so every outbound Sky-side
//! Tauri event carries a W3C `_traceparent` field on its JSON payload.
//! Sky's `Workbench/Electron/TraceparentBridge.ts::ConsumeFromPayload`
//! strips the field at the receiving end, registers the trace context
//! for the duration of the event handler, and Sky's `OTELBridge` reads
//! it so spans emitted inside the handler attach to the same trace.
//!
//! Migration plan: replace `app_handle.emit(...)` call sites
//! incrementally with `EmitWithTraceparent::Fn(...)`. Both paths
//! coexist - the bridge tolerates payloads without `_traceparent`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

/// Emit a Tauri event with a `_traceparent` field merged into its
/// JSON payload. `Payload` must be a JSON object (or null - we'll
/// build one). Non-object payloads pass through unchanged so existing
/// emit sites that send raw arrays / numbers / strings stay correct.
///
/// Release builds: `cfg!(debug_assertions)` short-circuits to a plain
/// `app_handle.emit(...)` so no traceparent bytes ship to production.
pub fn Fn<R:tauri::Runtime>(ApplicationHandle:&AppHandle<R>, EventName:&str, Payload:Value) -> tauri::Result<()> {
	if !cfg!(debug_assertions) {
		return ApplicationHandle.emit(EventName, Payload);
	}

	let Header = CommonLibrary::Telemetry::Traceparent::Build();

	let Stamped = match Payload {
		Value::Object(mut Map) => {
			Map.insert("_traceparent".to_string(), Value::String(Header));

			Value::Object(Map)
		},

		Value::Null => json!({ "_traceparent": Header }),

		Other => Other,
	};

	ApplicationHandle.emit(EventName, Stamped)
}

/// Variant for callers that already serialise into a `serde_json::Map`.
pub fn Fn<R:tauri::Runtime>(
	ApplicationHandle:&AppHandle<R>,

	EventName:&str,

	mut Map:serde_json::Map<String, Value>,
) -> tauri::Result<()> {
	if cfg!(debug_assertions) {
		let Header = CommonLibrary::Telemetry::Traceparent::Build();

		Map.insert("_traceparent".to_string(), Value::String(Header));
	}

	ApplicationHandle.emit(EventName, Value::Object(Map))
}
