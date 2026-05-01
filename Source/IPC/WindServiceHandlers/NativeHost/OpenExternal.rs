#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `native:openExternal`, `nativeHost:openExternal`.
//! Opens an http/https URL in the platform default browser.

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn OpenExternal(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let url_str = Arguments
		.get(0)
		.ok_or("Missing URL".to_string())?
		.as_str()
		.ok_or("URL must be a string".to_string())?;

	dev_log!("lifecycle", "openExternal: {}", url_str);

	if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
		return Err(format!("Invalid URL format. Must start with http:// or https://: {}", url_str));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		let result = Command::new("open")
			.arg(url_str)
			.output()
			.map_err(|Error| format!("Failed to execute open command: {}", Error))?;

		if !result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&result.stderr)));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		let result = Command::new("cmd")
			.arg("/c")
			.arg("start")
			.arg(url_str)
			.output()
			.map_err(|Error| format!("Failed to execute start command: {}", Error))?;

		if !result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&result.stderr)));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		let handlers = ["xdg-open", "gnome-open", "kde-open", "x-www-browser"];
		let mut last_error = String::new();

		for handler in handlers.iter() {
			let result = Command::new(handler).arg(url_str).output();

			match result {
				Ok(output) if output.status.success() => {
					dev_log!("lifecycle", "opened with {}", handler);
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
			return Err(format!("Failed to open URL with any handler: {}", last_error));
		}
	}

	dev_log!("lifecycle", "opened URL: {}", url_str);
	Ok(Value::Bool(true))
}
