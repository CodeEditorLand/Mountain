//! Wire method: `notification:show`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::{ArgString, ArgStringOr, ArgVal};

fn NewId() -> String {
	use std::sync::atomic::{AtomicU64, Ordering};

	static SEQ:AtomicU64 = AtomicU64::new(1);

	format!("notification-{}", SEQ.fetch_add(1, Ordering::Relaxed))
}

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Message = ArgString(&Arguments, 0);

	let Severity = ArgStringOr(&Arguments, 1, "info");

	let Actions = {
		let V = ArgVal(&Arguments, 2);
		if V.is_null() { json!([]) } else { V }
	};

	let Id = NewId();

	let _ = ApplicationHandle.emit(
		SkyEvent::NotificationShow.AsStr(),
		json!({
			"id": Id,
			"message": Message,
			"severity": Severity,
			"actions": Actions,
		}),
	);

	Ok(json!(Id))
}
