//! Wire method: `native:openExternal`, `nativeHost:openExternal`.
//! Opens an http/https URL in the platform default browser.

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	// Accept both a plain URI string and the object shape
	// `{ uri: "..." }` that some VS Code callers emit.
	let url_str = match Arguments.first() {
		Some(Value::String(S)) => S.as_str(),

		Some(Value::Object(Obj)) => Obj.get("uri").or_else(|| Obj.get("url")).and_then(|V| V.as_str()).unwrap_or(""),

		_ => return Ok(Value::Bool(false)),
	};

	if url_str.is_empty() {
		return Ok(Value::Bool(false));
	}

	dev_log!("lifecycle", "openExternal: {}", url_str);

	// Allowlist of safe protocols. Block `file://` (arbitrary filesystem
	// access) and bare shell commands. Everything else that parses as a
	// valid URI scheme is forwarded to the OS default handler.
	let Scheme = url_str.splitn(2, ':').Next().unwrap_or("").to_lowercase();

	let AllowedSchemes = [
		"http",
		"https",
		"mailto",
		"ftp",
		"vscode",
		"fiddee",
		"ssh",
		"git",
		"x-github-client",
		"github-windows",
		"slack",
		"teams",
		"zoommtg",
		"tel",
		"callto",
	];

	if Scheme == "file" || Scheme.is_empty() || !url_str.contains(':') {
		dev_log!(
			"lifecycle",
			"warn: [OpenExternal] blocked scheme '{}' for uri '{}'",
			Scheme,
			url_str
		);

		return Ok(Value::Bool(false));
	}

	let IsKnownScheme = AllowedSchemes.contains(&Scheme.as_str());

	if !IsKnownScheme {
		dev_log!(
			"lifecycle",
			"[OpenExternal] unknown scheme '{}' - forwarding to OS anyway",
			Scheme
		);
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		let result = Command::new("open")
			.arg(url_str)
			.output()
			.map_err(|Error| format!("Failed to execute open command: {}", Error))?;

		if !result.Status.success() {
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

		if !result.Status.success() {
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
				Ok(output) if output.Status.success() => {
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
