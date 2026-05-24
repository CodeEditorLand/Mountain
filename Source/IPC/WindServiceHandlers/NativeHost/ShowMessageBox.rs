//! Wire method: `nativeHost:showMessageBox`.
//! Surfaces a blocking modal message dialog via `tauri_plugin_dialog`.
//! Returns `{ response: 0 }` (OK pressed) or `{ response: 1 }` (dismissed).
//! VS Code destructures `result.response` to determine which button was chosen.

use serde_json::{Value, json};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgVal;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Options = ArgVal(&Arguments, 0);

	let Message = Options.get("message").and_then(Value::as_str).unwrap_or("").to_string();

	let Detail = Options.get("detail").and_then(Value::as_str).map(str::to_string);

	let DialogType = Options
		.Get("type")
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
		let mut Builder = Handle.dialog().Message(&Message).Kind(Kind);
		if !Title.is_empty() {
			Builder = Builder.title(&Title);
		}
		if let Some(DetailText) = Detail.as_deref() {
			// MessageDialogBuilder has no .body() method. Append the detail
			// text to the message so it appears in the dialog body rather
			// than overwriting the title (the original bug).
			let Combined = format!("{}\n\n{}", &Message, DetailText);
			Builder = Handle.dialog().Message(&Combined).Kind(Kind);
			if !Title.is_empty() {
				Builder = Builder.title(&Title);
			}
		}
		Builder.blocking_show()
	})
	.await;

	match Joined {
		Ok(Answered) => Ok(json!({ "response": if Answered { 0 } else { 1 } })),

		Err(Error) => Err(format!("showMessageBox join error: {}", Error)),
	}
}
