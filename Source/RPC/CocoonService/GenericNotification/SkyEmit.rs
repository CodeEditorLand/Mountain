
//! Notification handlers that forward events to Sky via `app.emit()`.
//! Covers: webview messages, status bar, output channels, progress,
//! `openExternal`, and language configuration.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

// ── Webview ───────────────────────────────────────────────────────────────

pub fn HandleOnDidReceiveMessage(Params:Value, Env:&MountainEnvironment) {
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

pub fn HandleWebviewPostMessage(Params:Value, Env:&MountainEnvironment) {
	let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Method = Params.get("method").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let MsgParams = Params.get("params").cloned().unwrap_or(Value::Null);

	let _ = Env.ApplicationHandle.emit(
		"sky://webview/message",
		json!({ "panelId": PanelId, "method": Method, "params": MsgParams }),
	);
}

pub fn HandleWebviewDispose(Params:Value, Env:&MountainEnvironment) {
	let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://webview/dispose", json!({ "panelId": PanelId }));
}

// ── Status bar ────────────────────────────────────────────────────────────

pub fn HandleSetStatusBarText(Params:Value, Env:&MountainEnvironment) {
	let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("text").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://statusbar/update", json!({ "id": ItemId, "text": Text }));
}

pub fn HandleDisposeStatusBarItem(Params:Value, Env:&MountainEnvironment) {
	let ItemId = Params.get("itemId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit("sky://statusbar/dispose", json!({ "id": ItemId }));
}

// ── Output channels ───────────────────────────────────────────────────────

pub fn HandleOutputCreate(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Name = Params.get("name").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://output/create", json!({ "id": Id, "name": Name }));
}

pub fn HandleOutputAppend(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Text = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://output/append", json!({ "channel": Channel, "text": Text }));
}

pub fn HandleOutputAppendLine(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Line = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit(
		"sky://output/append",
		json!({ "channel": Channel, "text": format!("{}\n", Line) }),
	);
}

pub fn HandleOutputClear(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit("sky://output/clear", json!({ "channel": Channel }));
}

pub fn HandleOutputShow(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit("sky://output/show", json!({ "channel": Channel }));
}

pub fn HandleOutputDispose(Params:Value, Env:&MountainEnvironment) {
	let Channel = Params.get("channel").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://output/dispose", json!({ "channel": Channel }));
}

// ── Progress ──────────────────────────────────────────────────────────────

pub fn HandleProgressStart(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Title = Params.get("title").and_then(|V| V.as_str()).map(|S| S.to_string());

	let Location = Params.get("location").cloned();

	let Cancellable = Params.get("cancellable").and_then(|V| V.as_bool()).unwrap_or(false);

	let _ = Env.ApplicationHandle.emit(
		"sky://progress/start",
		json!({ "id": Id, "title": Title, "location": Location, "cancellable": Cancellable }),
	);
}

pub fn HandleProgressUpdate(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Message = Params.get("message").and_then(|V| V.as_str()).map(|S| S.to_string());

	let Increment = Params.get("increment").and_then(|V| V.as_f64());

	let _ = Env.ApplicationHandle.emit(
		"sky://progress/update",
		json!({ "id": Id, "message": Message, "increment": Increment }),
	);
}

pub fn HandleProgressComplete(Params:Value, Env:&MountainEnvironment) {
	let Id = Params.get("id").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit("sky://progress/complete", json!({ "id": Id }));
}

// ── Misc ──────────────────────────────────────────────────────────────────

pub fn HandleOpenExternal(Params:Value, Env:&MountainEnvironment) {
	let Url = Params.get("url").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.ApplicationHandle.emit("sky://native/openExternal", json!({ "url": Url }));
}

pub fn HandleSetLanguageConfiguration(Params:Value, Env:&MountainEnvironment) {
	let Language = Params.get("language").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env
		.ApplicationHandle
		.emit("sky://language/configure", json!({ "language": Language }));
}
