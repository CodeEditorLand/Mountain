#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Shared utilities: path resolution, userdata mapping, percent-decoding,
//! metadata conversion, and recently-opened workspace helpers.

use serde_json::{Value, json};

use crate::dev_log;

// =========================================================================
// Static roots
// =========================================================================

/// The real filesystem root for /Static/Application/ paths.
/// Set once at startup with the Sky Target directory.
static STATIC_APPLICATION_ROOT:std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Set the real filesystem root for /Static/Application/ paths.
/// Call once at startup with the Sky Target directory.
pub fn set_static_application_root(Path:String) { let _ = STATIC_APPLICATION_ROOT.set(Path); }

/// Get the filesystem root for /Static/Application/ paths.
/// Returns None if not yet initialized.
pub fn get_static_application_root() -> Option<String> { STATIC_APPLICATION_ROOT.get().cloned() }

/// Canonical userdata base directory, set once from Tauri's PathResolver.
/// All /User/... paths resolve against this so the VFS mapping matches
/// the real Tauri app_data_dir (which includes the full bundle identifier).
static USERDATA_BASE_DIR:std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Set the userdata base from Tauri's app_data_dir. Call once at startup.
pub fn set_userdata_base_dir(Path:String) { let _ = USERDATA_BASE_DIR.set(Path); }

/// Get the base directory for userdata storage.
/// Returns the Tauri-resolved path if available, otherwise a fallback.
pub fn get_userdata_base_dir() -> String {
	if let Some(Dir) = USERDATA_BASE_DIR.get() {
		return Dir.clone();
	}
	// Fallback before Tauri sets it (should not happen in normal flow)
	if let Ok(Home) = std::env::var("HOME") {
		#[cfg(target_os = "macos")]
		return format!("{}/Library/Application Support/Land", Home);
		#[cfg(target_os = "linux")]
		return format!("{}/.local/share/Land", Home);
	}
	"/tmp/Land".to_string()
}

// =========================================================================
// Path extraction and normalization
// =========================================================================

/// Extract a filesystem path from a VS Code argument.
/// VS Code sends URI objects `{ scheme: "file", path: "/C:/foo", fsPath:
/// "C:\\foo" }` but Mountain handlers expect platform-native path strings.
///
/// Windows URI paths have a leading slash: `/C:/Users/...` → strip it.
/// Unix paths start with `/` normally.
pub fn extract_path_from_arg(Arg:&Value) -> Result<String, String> {
	// Direct string path
	if let Some(Path) = Arg.as_str() {
		return Ok(normalize_uri_path(Path));
	}
	// URI object { scheme, path, fsPath, ... }
	if let Some(Object) = Arg.as_object() {
		// Prefer fsPath (already OS-normalized by VS Code)
		if let Some(FsPath) = Object.get("fsPath").and_then(|V| V.as_str()) {
			if !FsPath.is_empty() {
				return Ok(FsPath.to_string());
			}
		}
		// Fall back to path field (URI-encoded, may have leading / on Windows)
		if let Some(Path) = Object.get("path").and_then(|V| V.as_str()) {
			if !Path.is_empty() {
				return Ok(normalize_uri_path(Path));
			}
		}
		// Handle external (full URI string like file:///C:/foo or file:///home/user)
		if let Some(External) = Object.get("external").and_then(|V| V.as_str()) {
			if External.starts_with("file://") {
				let Stripped = External.trim_start_matches("file://");
				return Ok(normalize_uri_path(Stripped));
			}
		}
	}
	Err("File path must be a string or URI object with path/fsPath field".to_string())
}

/// Normalize a URI-style path to a platform-native path.
/// On Windows, URI paths look like `/C:/Users/...` - strip the leading slash.
/// On Unix, paths already start with `/`.
/// Also handles percent-encoded characters (%20 for space, etc.)
/// Also maps vscode-userdata paths `/User/...` to the real userdata directory.
fn normalize_uri_path(Path:&str) -> String {
	let Decoded = percent_decode(Path);

	// Map vscode-userdata paths: /User/... → ~/.land/User/...
	let Resolved = resolve_userdata_path(&Decoded);

	// Map /Static/Application/... → real Sky Target directory.
	let Resolved = resolve_static_application_path(&Resolved);

	#[cfg(target_os = "windows")]
	{
		// Windows: URI path "/C:/Users/foo" → "C:\\Users\\foo"
		let Trimmed = if Resolved.len() >= 3 && Resolved.starts_with('/') && Resolved.as_bytes().get(2) == Some(&b':') {
			// /C:/path → C:/path
			Resolved[1..].to_string()
		} else {
			Resolved
		};
		// Normalize forward slashes to backslashes for Windows
		Trimmed.replace('/', "\\")
	}

	#[cfg(not(target_os = "windows"))]
	{
		Resolved
	}
}

/// Map vscode-userdata paths to real filesystem paths.
/// /User/settings.json → ~/.land/User/settings.json
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
/// directory.
///
/// Also accepts the leading-slash-less form `Static/Application/...`: the
/// webview's WASM loader (`vscode-oniguruma` → `onig.wasm`) resolves the
/// asset URL relative to the current document, which strips the leading
/// slash before the path reaches the `file:read` IPC handler. Without this
/// branch, `tokio::fs::read` would be called with a relative path and fail
/// with ENOENT, breaking TextMate syntax highlighting.
fn resolve_static_application_path(Path:&str) -> String {
	let Normalized = if Path.starts_with("/Static/Application/") || Path == "/Static/Application" {
		Path.to_string()
	} else if Path.starts_with("Static/Application/") || Path == "Static/Application" {
		format!("/{}", Path)
	} else {
		return Path.to_string();
	};

	if let Some(Root) = STATIC_APPLICATION_ROOT.get() {
		let Relative = Normalized.strip_prefix("/Static/Application").unwrap_or("");
		let Resolved = format!("{}/Static/Application{}", Root, Relative);
		dev_log!("vfs", "resolve_static: {} -> {}", Path, Resolved);
		Resolved
	} else {
		// Fallback: return path unchanged (will fail with ENOENT)
		Path.to_string()
	}
}

// =========================================================================
// Percent decoding
// =========================================================================

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

pub fn hex_digit(Byte:u8) -> Option<u8> {
	match Byte {
		b'0'..=b'9' => Some(Byte - b'0'),
		b'a'..=b'f' => Some(Byte - b'a' + 10),
		b'A'..=b'F' => Some(Byte - b'A' + 10),
		_ => None,
	}
}

// =========================================================================
// Metadata conversion
// =========================================================================

/// Convert a `std::fs::Metadata` to VS Code's `IStat` shape.
pub fn metadata_to_istat(Metadata:&std::fs::Metadata) -> Value {
	let FileType = if Metadata.is_symlink() {
		64 // SymbolicLink
	} else if Metadata.is_dir() {
		2 // Directory
	} else {
		1 // File
	};

	let Size = Metadata.len();

	let Mtime = Metadata
		.modified()
		.ok()
		.and_then(|T| T.duration_since(std::time::UNIX_EPOCH).ok())
		.map(|D| D.as_millis() as u64)
		.unwrap_or(0);

	let Ctime = Metadata
		.created()
		.ok()
		.and_then(|T| T.duration_since(std::time::UNIX_EPOCH).ok())
		.map(|D| D.as_millis() as u64)
		.unwrap_or(Mtime);

	json!({
		"type": FileType,
		"size": Size,
		"mtime": Mtime,
		"ctime": Ctime
	})
}

// =========================================================================
// Echo priority helper
// =========================================================================

use Echo::Task::Priority::Priority as EchoPriority;

/// Map a wire command string → Echo scheduler lane.
/// Unknown commands fall back to `Priority::Normal`.
pub fn ResolveChannelPriority(Command:&str) -> EchoPriority {
	use std::str::FromStr;

	match CommonLibrary::IPC::Channel::Channel::from_str(Command) {
		Ok(Channel) => {
			match Channel.Priority() {
				CommonLibrary::IPC::Channel::ChannelPriority::High => EchoPriority::High,
				CommonLibrary::IPC::Channel::ChannelPriority::Normal => EchoPriority::Normal,
				CommonLibrary::IPC::Channel::ChannelPriority::Low => EchoPriority::Low,
			}
		},
		Err(_) => EchoPriority::Normal,
	}
}

// =========================================================================
// Userdata directory initialization
// =========================================================================

/// Ensure userdata directories exist on first access.
static USERDATA_INITIALIZED:std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn ensure_userdata_dirs() {
	if USERDATA_INITIALIZED.swap(true, std::sync::atomic::Ordering::Relaxed) {
		return; // Already initialized
	}

	let Base = get_userdata_base_dir();
	let Dirs = [
		format!("{}/User", Base),
		format!("{}/User/globalStorage", Base),
		format!("{}/User/profiles/__default__profile__", Base),
		format!("{}/User/snippets", Base),
		format!("{}/User/prompts", Base),
		format!("{}/User/cacheHome", Base),
		format!("{}/logs", Base),
		format!("{}/User/workspaceStorage", Base),
		format!(
			"{}/CachedConfigurations/defaults/__default__profile__-configurationDefaultsOverrides",
			Base
		),
	];

	for Dir in &Dirs {
		if let Err(E) = std::fs::create_dir_all(Dir) {
			dev_log!("lifecycle", "Failed to create userdata dir {}: {}", Dir, E);
		}
	}

	// Create default empty files if they don't exist
	let DefaultFiles = [
		(format!("{}/User/settings.json", Base), "{}"),
		(format!("{}/User/keybindings.json", Base), "[]"),
		(format!("{}/User/tasks.json", Base), "{}"),
		(format!("{}/User/extensions.json", Base), "[]"),
		(format!("{}/User/mcp.json", Base), "{}"),
	];

	for (FilePath, DefaultContent) in &DefaultFiles {
		if !std::path::Path::new(FilePath).exists() {
			if let Err(E) = std::fs::write(FilePath, DefaultContent) {
				dev_log!("lifecycle", "Failed to create default file {}: {}", FilePath, E);
			}
		}
	}

	dev_log!("lifecycle", "userdata dirs initialized at: {}/User/", Base);
}

// =========================================================================
// Recently-opened helpers
// =========================================================================

pub fn RecentlyOpenedPath() -> std::path::PathBuf {
	let Home = std::env::var("HOME")
		.or_else(|_| std::env::var("USERPROFILE"))
		.unwrap_or_default();
	std::path::PathBuf::from(Home)
		.join(".land")
		.join("workspaces")
		.join("RecentlyOpened.json")
}

pub fn ReadRecentlyOpened() -> Result<Value, String> {
	let Path = RecentlyOpenedPath();
	match std::fs::read_to_string(&Path) {
		Ok(Contents) => {
			match serde_json::from_str::<Value>(&Contents) {
				Ok(Parsed) => Ok(Parsed),
				Err(_) => Ok(json!({ "workspaces": [], "files": [] })),
			}
		},
		Err(_) => Ok(json!({ "workspaces": [], "files": [] })),
	}
}

pub fn MutateRecentlyOpened<F:FnOnce(&mut serde_json::Map<String, Value>)>(Apply:F) {
	let Path = RecentlyOpenedPath();
	let mut Parsed:serde_json::Map<String, Value> = std::fs::read_to_string(&Path)
		.ok()
		.and_then(|Contents| serde_json::from_str::<Value>(&Contents).ok())
		.and_then(|V| V.as_object().cloned())
		.unwrap_or_default();
	if !Parsed.contains_key("workspaces") {
		Parsed.insert("workspaces".into(), json!([]));
	}
	if !Parsed.contains_key("files") {
		Parsed.insert("files".into(), json!([]));
	}
	Apply(&mut Parsed);
	if let Some(Parent) = Path.parent() {
		let _ = std::fs::create_dir_all(Parent);
	}
	if let Ok(Serialised) = serde_json::to_vec_pretty(&Value::Object(Parsed)) {
		let _ = std::fs::write(&Path, Serialised);
	}
}

/// Extract a string out of a Value that may be either a raw string or a
/// UriComponents-ish object with `external`/`path`/`toString`.
pub fn v_str(Value:&Value) -> Option<String> {
	if let Some(s) = Value.as_str() {
		return Some(s.to_string());
	}
	if let Some(Object) = Value.as_object() {
		if let Some(s) = Object.get("external").and_then(|V| V.as_str()) {
			return Some(s.to_string());
		}
		if let Some(s) = Object.get("path").and_then(|V| V.as_str()) {
			return Some(s.to_string());
		}
	}
	None
}
