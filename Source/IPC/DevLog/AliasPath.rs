//! Replace the long Tauri app-data prefix with `$APP` so
//! `Trace=short` lines stay readable.

use crate::IPC::DevLog::AppDataPrefix;

pub fn Fn(Input:&str) -> String {
	if let Some(Prefix) = AppDataPrefix::Fn() {
		Input.replace(Prefix, "$APP")
	} else {
		Input.to_string()
	}
}
