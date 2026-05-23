#![allow(non_snake_case, unused_variables)]

//! Wire method: `notification:showProgress`.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

fn NewId() -> String {
	use std::sync::atomic::{AtomicU64, Ordering};

	static SEQ:AtomicU64 = AtomicU64::new(1);

	format!("progress-{}", SEQ.fetch_add(1, Ordering::Relaxed))
}

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Title = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Cancellable = Arguments.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

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
