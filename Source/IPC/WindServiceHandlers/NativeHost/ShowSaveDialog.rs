//! Wire method: `nativeHost:showSaveDialog`.
//! Returns `{ canceled: bool, filePath?: string }`.

use serde_json::{Value, json};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_val;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Options = arg_val(&Arguments, 0);

	let Title = Options.get("title").and_then(Value::as_str).unwrap_or("Save").to_string();

	let DefaultPath = Options.get("defaultPath").and_then(Value::as_str).map(str::to_string);

	let Handle = ApplicationHandle.clone();

	let Joined = tokio::task::spawn_blocking(move || -> Option<String> {
		let mut Builder = Handle.dialog().file().set_title(&Title);
		if let Some(Path) = DefaultPath.as_deref() {
			Builder = Builder.set_directory(Path);
		}
		Builder.blocking_save_file().map(|P| P.to_string())
	})
	.await;

	match Joined {
		Ok(Some(Path)) => Ok(json!({ "canceled": false, "filePath": Path })),

		Ok(None) => Ok(json!({ "canceled": true })),

		Err(Error) => Err(format!("showSaveDialog join error: {}", Error)),
	}
}
