
use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0);

	let Message = Params
		.get("stringMessage")
		.and_then(|V| V.as_str())
		.map(|S| S.to_string())
		.or_else(|| Params.get("bytesMessage").map(|_| "[binary]".to_string()))
		.unwrap_or_default();

	let _ = Env
		.ApplicationHandle
		.emit("sky://webview/post-message", json!({ "handle": Handle, "message": Message }));
}
