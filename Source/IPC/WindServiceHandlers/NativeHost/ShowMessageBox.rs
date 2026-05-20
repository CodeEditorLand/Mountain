#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:showMessageBox`.
//! Surfaces a blocking modal message dialog via `tauri_plugin_dialog`.
//! Returns `{ response: 0 }` (OK pressed) or `{ response: 1 }` (dismissed).
//! VS Code destructures `result.response` to determine which button was chosen.

use serde_json::{Value, json};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

pub async fn NativeShowMessageBox(
	ApplicationHandle:AppHandle,
	Arguments:Vec<Value>,
) -> Result<Value, String> {
	let Options = Arguments.first().cloned().unwrap_or(Value::Null);
	let Message = Options.get("message").and_then(Value::as_str).unwrap_or("").to_string();
	let Detail = Options.get("detail").and_then(Value::as_str).map(str::to_string);
	let DialogType = Options
		.get("type")
		.and_then(Value::as_str)
		.map(|S| S.to_lowercase())
		.unwrap_or_default();
	let Title = Options.get("title").and_then(Value::as_str).unwrap_or("").to_string();
	let Kind = match DialogType.as_str() {
		"warning" | "warn" => MessageDialogKind::Warning,
		"error" => MessageDialogKind::Error,
		_ => MessageDialogKind::Info,
	};
	let Handle = ApplicationHandle.clone();
	let Joined = tokio::task::spawn_blocking(move || -> bool {
		let mut Builder = Handle.dialog().message(&Message).kind(Kind);
		if !Title.is_empty() {
			Builder = Builder.title(&Title);
		}
		if let Some(DetailText) = Detail.as_deref() {
			Builder = Builder.title(DetailText);
		}
		Builder.blocking_show()
	})
	.await;
	match Joined {
		Ok(Answered) => Ok(json!({ "response": if Answered { 0 } else { 1 } })),
		Err(Error) => Err(format!("showMessageBox join error: {}", Error)),
	}
}
