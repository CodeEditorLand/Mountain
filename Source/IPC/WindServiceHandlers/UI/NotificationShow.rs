//! Wire method: `notification:show`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

fn NewId() -> String {
	use std::sync::atomic::{AtomicU64, Ordering};

	static SEQ:AtomicU64 = AtomicU64::new(1);

	format!("notification-{}", SEQ.fetch_add(1, Ordering::Relaxed))
}

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Message = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Severity = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("info").to_string();

	let Actions = Arguments.get(2).cloned().unwrap_or(json!([]));

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
