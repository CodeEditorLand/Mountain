#![allow(non_snake_case)]

//! Environment domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Handler for environment get requests
pub async fn handle_environment_get(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Key = Args
		.get(0)
		.ok_or("Missing environment key".to_string())?
		.as_str()
		.ok_or("Environment key must be a string".to_string())?;

	let EnvValue = std::env::var(Key).map_err(|E| format!("Failed to get environment variable: {}", E))?;

	dev_log!("config", "env_get: {}", Key);
	Ok(json!(EnvValue))
}

/// Handler for showing items in folder
pub async fn handle_show_item_in_folder(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let PathStr = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	dev_log!("vfs", "showInFolder: {}", PathStr);

	let Path = std::path::PathBuf::from(PathStr);

	if !Path.exists() {
		return Err(format!("Path does not exist: {}", PathStr));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		let Result = Command::new("open")
			.arg("-R")
			.arg(&Path)
			.output()
			.map_err(|E| format!("Failed to execute open command: {}", E))?;

		if !Result.status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&Result.stderr)
			));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		let Result = Command::new("explorer")
			.arg("/select,")
			.arg(&Path)
			.output()
			.map_err(|E| format!("Failed to execute explorer command: {}", E))?;

		if !Result.status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&Result.stderr)
			));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		let FileManagers = ["nautilus", "dolphin", "thunar", "pcmanfm", "nemo"];
		let mut LastError = String::new();

		for Manager in FileManagers.iter() {
			let Result = Command::new(Manager).arg(&Path).output();

			match Result {
				Ok(Output) if Output.status.success() => {
					dev_log!("lifecycle", "opened with {}", Manager);
					break;
				},
				Err(E) => {
					LastError = E.to_string();
					continue;
				},
				_ => continue,
			}
		}

		if !LastError.is_empty() {
			return Err(format!("Failed to show item in folder with any file manager: {}", LastError));
		}
	}

	dev_log!("vfs", "showed in folder: {}", PathStr);
	Ok(Value::Bool(true))
}

/// Handler for opening external URLs
pub async fn handle_open_external(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let UrlStr = Args
		.get(0)
		.ok_or("Missing URL".to_string())?
		.as_str()
		.ok_or("URL must be a string".to_string())?;

	dev_log!("lifecycle", "openExternal: {}", UrlStr);

	if !UrlStr.starts_with("http://") && !UrlStr.starts_with("https://") {
		return Err(format!("Invalid URL format. Must start with http:// or https://: {}", UrlStr));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		let Result = Command::new("open")
			.arg(UrlStr)
			.output()
			.map_err(|E| format!("Failed to execute open command: {}", E))?;

		if !Result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&Result.stderr)));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		let Result = Command::new("cmd")
			.arg("/c")
			.arg("start")
			.arg(UrlStr)
			.output()
			.map_err(|E| format!("Failed to execute start command: {}", E))?;

		if !Result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&Result.stderr)));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		let Handlers = ["xdg-open", "gnome-open", "kde-open", "x-www-browser"];
		let mut LastError = String::new();

		for Handler in Handlers.iter() {
			let Result = Command::new(Handler).arg(UrlStr).output();

			match Result {
				Ok(Output) if Output.status.success() => {
					dev_log!("lifecycle", "opened with {}", Handler);
					break;
				},
				Err(E) => {
					LastError = E.to_string();
					continue;
				},
				_ => continue,
			}
		}

		if !LastError.is_empty() {
			return Err(format!("Failed to open URL with any handler: {}", LastError));
		}
	}

	dev_log!("lifecycle", "opened URL: {}", UrlStr);
	Ok(Value::Bool(true))
}
