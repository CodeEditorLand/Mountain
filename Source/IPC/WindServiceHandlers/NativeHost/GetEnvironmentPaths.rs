#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:getEnvironmentPaths`.
//! Returns paths used by VS Code's `ResolveConfiguration` to locate user-data,
//! logs, home, and temp directories. The session-timestamped logs subdirectory
//! is created on first call so VS Code can write output files immediately.

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

pub async fn NativeGetEnvironmentPaths(ApplicationHandle:AppHandle) -> Result<Value, String> {
	let PathResolver = ApplicationHandle.path();

	let AppDataDir = PathResolver.app_data_dir().unwrap_or_default();

	let HomeDir = PathResolver.home_dir().unwrap_or_default();

	let TmpDir = std::env::temp_dir();

	// Logs go under {appDataDir}/logs/{sessionTimestamp}/ - same tree as all
	// other VS Code data, not Tauri's separate app_log_dir(). VS Code requires
	// a session-timestamped subdir for log rotation. `DevLog::SessionTimestamp`
	// is the single source of truth so that `Mountain.dev.log` (written by
	// DevLog) and VS Code's `window1/output/*.log` files (written into
	// `logsPath`) share one directory per session.
	let SessionLogRoot = AppDataDir.join("logs").join(crate::IPC::DevLog::SessionTimestamp::Fn());

	let SessionLogWindowDir = SessionLogRoot.join("window1");

	let _ = std::fs::create_dir_all(&SessionLogWindowDir);

	crate::dev_log!(
		"config",
		"getEnvironmentPaths: userDataDir={} logsPath={} homeDir={}",
		AppDataDir.display(),
		SessionLogRoot.display(),
		HomeDir.display()
	);

	let DevLogEnv = std::env::var("Trace").unwrap_or_default();

	Ok(json!({
		"userDataDir": AppDataDir.to_string_lossy(),
		"logsPath": SessionLogRoot.to_string_lossy(),
		"homeDir": HomeDir.to_string_lossy(),
		"tmpDir": TmpDir.to_string_lossy(),
		"devLog": if DevLogEnv.is_empty() { Value::Null } else { json!(DevLogEnv) },
	}))
}
