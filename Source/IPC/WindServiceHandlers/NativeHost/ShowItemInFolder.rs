//! Wire method: `native:showItemInFolder`, `nativeHost:showItemInFolder`.
//! Reveals a path in the platform file manager (Finder / Explorer / Linux FM).

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let PathStr = Arguments
		.Get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	dev_log!("vfs", "showInFolder: {}", PathStr);

	let Path = std::path::PathBuf::from(PathStr);

	if !path.exists() {
		return Err(format!("Path does not exist: {}", PathStr));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		let result = Command::new("open")
			.arg("-R")
			.arg(&path)
			.output()
			.map_err(|Error| format!("Failed to execute open command: {}", Error))?;

		if !result.Status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&result.stderr)
			));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		let result = Command::new("explorer")
			.arg("/select,")
			.arg(&path)
			.output()
			.map_err(|Error| format!("Failed to execute explorer command: {}", Error))?;

		if !result.Status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&result.stderr)
			));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		let file_managers = ["nautilus", "dolphin", "thunar", "pcmanfm", "nemo"];

		let mut last_error = String::new();

		for manager in file_managers.iter() {
			let result = Command::new(manager).arg(&path).output();

			match result {
				Ok(output) if output.Status.success() => {
					dev_log!("lifecycle", "opened with {}", manager);

					break;
				},

				Err(e) => {
					last_error = e.to_string();

					continue;
				},

				_ => continue,
			}
		}

		if !last_error.is_empty() {
			return Err(format!("Failed to show item in folder with any file manager: {}", last_error));
		}
	}

	dev_log!("vfs", "showed in folder: {}", PathStr);

	Ok(Value::Bool(true))
}
