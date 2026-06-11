//! `workspaces:createUntitledWorkspace` IPC handler - writes an empty
//! `.code-workspace` file into the app-data `.untitled-workspaces`
//! directory and returns VS Code's `{ configPath, id }` workspace
//! identifier shape.

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

use crate::dev_log;

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {
	let PathResolver = ApplicationHandle.path();

	let AppDataDir = PathResolver
		.app_data_dir()
		.map_err(|E| format!("workspaces:createUntitledWorkspace app_data_dir: {}", E))?;

	let UntitledDir = AppDataDir.join(".untitled-workspaces");

	tokio::fs::create_dir_all(&UntitledDir)
		.await
		.map_err(|E| format!("workspaces:createUntitledWorkspace mkdir: {}", E))?;

	let Id = uuid::Uuid::new_v4();

	let FileName = format!("Untitled-{}.code-workspace", Id);

	let FilePath = UntitledDir.join(&FileName);

	let Content = r#"{"folders":[],"settings":{}}"#;

	tokio::fs::write(&FilePath, Content)
		.await
		.map_err(|E| format!("workspaces:createUntitledWorkspace write: {}", E))?;

	let FilePathStr = FilePath.to_string_lossy().to_string();

	dev_log!("workspaces", "createUntitledWorkspace: id={} path={}", Id, FilePathStr);

	Ok(json!({ "configPath": FilePathStr, "id": Id.to_string() }))
}
