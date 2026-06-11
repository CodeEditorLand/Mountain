//! `workspaces:deleteUntitledWorkspace` IPC handler - removes an
//! untitled `.code-workspace` file. The target is canonicalised and must
//! live inside the app-data `.untitled-workspaces` directory; anything
//! else is rejected (silent Null) so the arm can never be used to delete
//! arbitrary files.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::dev_log;

pub async fn Fn(ApplicationHandle:AppHandle, Arguments:Vec<Value>) -> Result<Value, String> {
	let Arg = Arguments.first().cloned().unwrap_or(Value::Null);

	let ConfigPath = Arg
		.as_str()
		.or_else(|| Arg.get("configPath").and_then(Value::as_str))
		.unwrap_or("")
		.to_string();

	if ConfigPath.is_empty() {
		dev_log!("workspaces", "deleteUntitledWorkspace: no configPath");

		return Ok(Value::Null);
	}

	let PathResolver = ApplicationHandle.path();

	let AppDataDir = PathResolver
		.app_data_dir()
		.map_err(|E| format!("workspaces:deleteUntitledWorkspace app_data_dir: {}", E))?;

	let UntitledDir = AppDataDir.join(".untitled-workspaces");

	let Target = std::path::PathBuf::from(&ConfigPath);

	let IsInUntitledDir = Target
		.canonicalize()
		.ok()
		.zip(UntitledDir.canonicalize().ok())
		.map(|(T, U)| T.starts_with(U))
		.unwrap_or(false);

	if !IsInUntitledDir {
		dev_log!("workspaces", "deleteUntitledWorkspace: rejected path outside untitled dir: {}", ConfigPath);

		return Ok(Value::Null);
	}

	tokio::fs::remove_file(&Target)
		.await
		.map_err(|E| format!("workspaces:deleteUntitledWorkspace remove: {}", E))?;

	dev_log!("workspaces", "deleteUntitledWorkspace: removed {}", ConfigPath);

	Ok(Value::Null)
}
