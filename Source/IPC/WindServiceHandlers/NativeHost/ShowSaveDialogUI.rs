//! Wire method: `UserInterface.ShowSaveDialog`.
//! Returns bare path string or null so Wind's `typeof Result === "string"`
//! guard finds a string value directly.

use serde_json::{Value, json};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Options = Arguments.first().cloned().unwrap_or(Value::Null);

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
		Ok(Some(Path)) => Ok(json!(Path)),

		Ok(None) => Ok(Value::Null),

		Err(Error) => Err(format!("UserInterface.ShowSaveDialog join error: {}", Error)),
	}
}
