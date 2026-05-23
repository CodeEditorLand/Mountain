//! Converts VS Code `Uri`-shaped arguments to platform-native paths.
//! Co-locates percent-decoding, userdata remapping, and `/Static/Application`
//! rewriting because each is a private helper of `extract_path_from_arg`.
//! Percent-decoding is also re-exported for callers outside the VFS path
//! (configuration loaders, etc.).

use serde_json::Value;

use super::{ApplicationRoot::Get::Fn as get_static_application_root, UserdataDir::Get::Fn as get_userdata_base_dir};
use crate::dev_log;

/// Extract a filesystem path from a VS Code argument.
/// VS Code sends URI objects `{ scheme: "file", path: "/C:/foo", fsPath:
/// "C:\\foo" }` but Mountain handlers expect platform-native path strings.
///
/// Windows URI paths have a leading slash: `/C:/Users/...` → strip it.
/// Unix paths start with `/` normally.
pub fn Fn(Arg:&Value) -> Result<String, String> {
	if let Some(Path) = Arg.as_str() {
		return Ok(normalize_uri_path(Path));
	}

	if let Some(Object) = Arg.as_object() {
		if let Some(FsPath) = Object.get("fsPath").and_then(|V| V.as_str()) {
			if !FsPath.is_empty() {
				return Ok(FsPath.to_string());
			}
		}

		if let Some(Path) = Object.get("path").and_then(|V| V.as_str()) {
			if !Path.is_empty() {
				return Ok(normalize_uri_path(Path));
			}
		}

		if let Some(External) = Object.get("external").and_then(|V| V.as_str()) {
			if External.starts_with("file://") {
				let Stripped = External.trim_start_matches("file://");

				return Ok(normalize_uri_path(Stripped));
			}
		}
	}

	Err("File path must be a string or URI object with path/fsPath field".to_string())
}

fn normalize_uri_path(Path:&str) -> String {
	let Decoded = percent_decode(Path);

	let Resolved = resolve_userdata_path(&Decoded);

	let Resolved = resolve_static_application_path(&Resolved);

	#[cfg(target_os = "windows")]
	{
		let Trimmed = if Resolved.len() >= 3 && Resolved.starts_with('/') && Resolved.as_bytes().get(2) == Some(&b':') {
			Resolved[1..].to_string()
		} else {
			Resolved
		};

		Trimmed.replace('/', "\\")
	}

	#[cfg(not(target_os = "windows"))]
	{
		Resolved
	}
}

fn resolve_userdata_path(Path:&str) -> String {
	if !Path.starts_with("/User/") && Path != "/User" {
		return Path.to_string();
	}

	let UserDataBase = get_userdata_base_dir();

	let Resolved = format!("{}{}", UserDataBase, Path);

	dev_log!("vfs", "resolve_userdata: {} -> {}", Path, Resolved);

	Resolved
}

/// Map paths starting with /Static/Application/ to the real Sky Target
/// directory. Also accepts the leading-slash-less form - the WASM loader
/// (`vscode-oniguruma` → `onig.wasm`) resolves asset URLs relative to the
/// current document, which strips the leading slash before the path
/// reaches `file:read`. Without this branch, `tokio::fs::read` would be
/// called with a relative path and fail with ENOENT, breaking TextMate
/// syntax highlighting.
fn resolve_static_application_path(Path:&str) -> String {
	let Normalized = if Path.starts_with("/Static/Application/") || Path == "/Static/Application" {
		Path.to_string()
	} else if Path.starts_with("Static/Application/") || Path == "Static/Application" {
		format!("/{}", Path)
	} else {
		return Path.to_string();
	};

	if let Some(Root) = get_static_application_root() {
		let Relative = Normalized.strip_prefix("/Static/Application").unwrap_or("");

		let Resolved = format!("{}/Static/Application{}", Root, Relative);

		dev_log!("vfs", "resolve_static: {} -> {}", Path, Resolved);

		Resolved
	} else {
		Path.to_string()
	}
}

/// Decode percent-encoded characters in URI paths.
/// Handles: %20 (space), %23 (#), %25 (%), %5B ([), %5D (]), etc.
pub fn percent_decode(Input:&str) -> String {
	let mut Result = String::with_capacity(Input.len());

	let Bytes = Input.as_bytes();

	let mut I = 0;

	while I < Bytes.len() {
		if Bytes[I] == b'%' && I + 2 < Bytes.len() {
			let High = hex_digit(Bytes[I + 1]);

			let Low = hex_digit(Bytes[I + 2]);

			if let (Some(H), Some(L)) = (High, Low) {
				Result.push((H * 16 + L) as char);

				I += 3;

				continue;
			}
		}

		Result.push(Bytes[I] as char);

		I += 1;
	}

	Result
}

fn hex_digit(Byte:u8) -> Option<u8> {
	match Byte {
		b'0'..=b'9' => Some(Byte - b'0'),

		b'a'..=b'f' => Some(Byte - b'a' + 10),

		b'A'..=b'F' => Some(Byte - b'A' + 10),

		_ => None,
	}
}
