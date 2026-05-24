//! Wire method: `notification:showProgress`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::{ArgBool, ArgString};

fn NewId() -> String {
	use std::sync::atomic::{AtomicU64, Ordering};

	static SEQ:AtomicU64 = AtomicU64::new(1);

	format!("progress-{}", SEQ.fetch_add(1, Ordering::Relaxed))
}

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Title = ArgString(&Arguments, 0);

	let Cancellable = ArgBool(&Arguments, 1);

	let Id = NewId();

	let _ = ApplicationHandle.emit(
		SkyEvent::NotificationProgressBegin.AsStr(),
		json!({
			"id": Id,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}
