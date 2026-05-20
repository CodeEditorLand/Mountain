#![allow(non_snake_case)]

//! Cocoon → Mountain `window.applyTextEdits` notification.
//!
//! Fired when an extension calls `editor.edit(editBuilder => { ... })`.
//! Cocoon's TextEditor shim collects the edits and sends them here.
//! Mountain emits `sky://editor/apply-text-edits` so Sky can apply them
//! via `ICodeEditorService.listCodeEditors()` → `editor.executeEdits(...)`.
//!
//! Payload shape:
//! ```json
//! {
//!   "uri": "file:///path/to/file.ts",
//!   "edits": [
//!     { "range": { "startLineNumber": 1, "startColumn": 1, "endLineNumber": 1, "endColumn": 10 }, "text": "replacement" },
//!     { "range": { ... }, "text": "" }
//!   ]
//! }
//! ```

use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ApplyTextEdits(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Uri = Parameter.get("uri").and_then(Value::as_str).unwrap_or("").to_string();

	let EditCount = Parameter.get("edits").and_then(Value::as_array).map(|A| A.len()).unwrap_or(0);

	dev_log!("model", "[ApplyTextEdits] uri={} edits={}", Uri, EditCount);

	if Uri.is_empty() || EditCount == 0 {
		return;
	}

	if let Err(E) = Service.ApplicationHandle().emit("sky://editor/apply-text-edits", Parameter) {
		dev_log!("sky-emit", "[ApplyTextEdits] emit failed: {}", E);
	}
}
