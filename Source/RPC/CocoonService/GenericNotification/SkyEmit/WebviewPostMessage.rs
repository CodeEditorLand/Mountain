#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tauri::Emitter;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub fn Fn(Params:Value, Env:&MountainEnvironment) {
	let PanelId = Params.get("panelId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Method = Params.get("method").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let MsgParams = Params.get("params").cloned().unwrap_or(Value::Null);

	let _ = Env.ApplicationHandle.emit(
		"sky://webview/message",
		json!({ "panelId": PanelId, "method": Method, "params": MsgParams }),
	);
}
