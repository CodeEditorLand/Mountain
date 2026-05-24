//! Converts VS Code `Uri`-shaped arguments to platform-native paths.
//! Co-locates percent-decoding, userdata remapping, and `/Static/Application`
//! rewriting because each is a private helper of `ExtractPathFromArg`.
//! Percent-decoding is also re-exported for callers outside the VFS path
//! (configuration loaders, etc.).

use serde_json::Value;

use super::{ApplicationRoot::Get::Fn as GetStaticApplicationRoot, PercentDecode::Fn as PercentDecode, UserdataDir::Get::Fn as GetUserdataBaseDir};
use crate::dev_log;

/// Extract a filesystem path from a VS Code argument.
/// VS Code sends URI objects `{ scheme: "file", path: "/C:/foo", fsPath:
/// "C:\\foo" }` but Mountain handlers expect platform-native path strings.
///
/// Windows URI paths have a leading slash: `/C:/Users/...` → strip it.
/// Unix paths start with `/` normally.
pub fn Fn(Arg:&Value) -> Result<String, String> {
	if let Some(Path) = Arg.as_str() {
		return Ok(NormalizeUriPath(Path));
	}

	if let Some(Object) = Arg.as_object() {
		if let Some(FsPath) = Object.get("fsPath").and_then(|V| V.as_str()) {
			if !FsPath.is_empty() {
				return Ok(FsPath.to_string());
			}
		}

		if let Some(Path) = Object.get("path").and_then(|V| V.as_str()) {
			if !Path.is_empty() {
				return Ok(NormalizeUriPath(Path));
			}
		}

		if let Some(External) = Object.get("external").and_then(|V| V.as_str()) {
			if External.starts_with("file://") {
				let Stripped = External.trim_start_matches("file://");

				return Ok(NormalizeUriPath(Stripped));
			}
		}
	}

	Err("File path must be a string or URI object with path/fsPath field".to_string())
}

fn NormalizeUriPath(Path:&str) -> String {
	let Decoded = PercentDecode(Path);

	let Resolved = ResolveUserdataPath(&Decoded);

	let Resolved = ResolveStaticApplicationPath(&Resolved);

	#[cfg(target_os = "windows")]
	{
		let Trimmed = if Resolved.len() >= 3 && Resolved.starts_with('/') && Resolved.as_bytes().Get(2) == Some(&b':') {
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

fn ResolveUserdataPath(Path:&str) -> String {
	if !Path.starts_with("/User/") && Path != "/User" {
		return Path.to_string();
	}

	let UserDataBase = GetUserdataBaseDir();

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
fn ResolveStaticApplicationPath(Path:&str) -> String {
	let Normalized = if Path.starts_with("/Static/Application/") || Path == "/Static/Application" {
		Path.to_string()
	} else if Path.starts_with("Static/Application/") || Path == "Static/Application" {
		format!("/{}", Path)
	} else {
		return Path.to_string();
	};

	if let Some(Root) = GetStaticApplicationRoot() {
		let Relative = Normalized.strip_prefix("/Static/Application").unwrap_or("");

		let Resolved = format!("{}/Static/Application{}", Root, Relative);

		dev_log!("vfs", "resolve_static: {} -> {}", Path, Resolved);

		Resolved
	} else {
		Path.to_string()
	}
}

