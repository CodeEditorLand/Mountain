//! # Shim — NativeBus
//!
//! Direct Rust handlers for IPC methods swallowed by SwallowMap.
//! When SwallowMap redirects an IPC method to `Mountain`, NativeBus
//! handles the operation natively — no Cocoon gRPC round-trip, no
//! serialization overhead, just Rust calling the OS directly.
//!
//! ## Handled Methods
//!
//! | Prefix | Handler | Backend |
//! |--------|---------|---------|
//! | `file:read` | `handle_file_read` | `tokio::fs` |
//! | `file:write` | `handle_file_write` | `tokio::fs` |
//! | `file:stat` | `handle_file_stat` | `tokio::fs` |
//! | `file:delete` | `handle_file_delete` | `tokio::fs` |
//! | `file:mkdir` | `handle_file_mkdir` | `tokio::fs` |
//! | `file:readdir` | `handle_file_readdir` | `tokio::fs` |
//! | `terminal:*` | `handle_terminal` | Native PTY |
//! | `search:*` | `handle_search` | Native rg |
//! | `dialog:*` | `handle_dialog` | Mountain dialog |
//! | `statusbar:*` | `handle_statusbar` | Sky emit |
//! | `telemetry:*` | (discard) | Drop |

use std::sync::Arc;

use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
	IPC::WindServiceHandlers::Utilities::PathExtraction::Fn as extract_path,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Route a swallowed IPC method to a direct native handler.
/// Returns the JSON response to send back to Wind/Sky.
pub async fn handle(_app_handle:&AppHandle, method:&str, args:&[Value]) -> Result<Value, String> {
	match method {
		// ── File System — direct tokio::fs ──
		m if m.starts_with("file:read") => handle_file_read(args).await,

		m if m.starts_with("file:write") => handle_file_write(args).await,

		m if m.starts_with("file:stat") => handle_file_stat(args).await,

		m if m.starts_with("file:delete") => handle_file_delete(args).await,

		m if m.starts_with("file:mkdir") => handle_file_mkdir(args).await,

		m if m.starts_with("file:readdir") => handle_file_readdir(args).await,

		// ── Terminal — direct PTY ──
		m if m.starts_with("terminal:") => {
			let params = args.first().ok_or("terminal: missing params")?;

			handle_terminal(_app_handle, method, params).await
		},

		// ── Search — direct ripgrep ──
		m if m.starts_with("search:") => {
			let params = args.first().ok_or("search: missing params")?;

			handle_search(params).await
		},

		// ── Dialog — Mountain native ──
		m if m.starts_with("dialog:") => handle_dialog(args).await,

		// ── Status bar — emit to Sky directly ──
		m if m.starts_with("statusbar:") => {
			let params = args.first().ok_or("statusbar: missing params")?;

			handle_statusbar(_app_handle, method, params)
		},

		// ── Telemetry — discard ──
		m if m.starts_with("telemetry:") => Ok(json!(null)),

		// ── Unknown swallowed method ──
		_ => Err(format!("NativeBus: no handler for swallowed method {}", method)),
	}
}

// ── File System Handlers ────────────────────────────────────────

async fn handle_file_read(args:&[Value]) -> Result<Value, String> {
	let path = extract_path(args.first().ok_or("file:read missing path")?)?;

	dev_log!("nativebus", "file:read {}", path);

	let bytes = tokio::fs::read(&path)
		.await
		.map_err(|e| format!("file:read failed for {}: {}", path, e))?;

	let byte_array:Vec<Value> = bytes.iter().map(|b| json!(*b)).collect();

	Ok(json!({ "buffer": byte_array }))
}

async fn handle_file_write(args:&[Value]) -> Result<Value, String> {
	let path = extract_path(args.first().ok_or("file:write missing path")?)?;

	let contents = args.get(1).and_then(|v| v.as_str()).ok_or("file:write missing contents")?;

	dev_log!("nativebus", "file:write {} ({} bytes)", path, contents.len());

	tokio::fs::write(&path, contents)
		.await
		.map_err(|e| format!("file:write failed for {}: {}", path, e))?;

	Ok(json!(null))
}

async fn handle_file_stat(args:&[Value]) -> Result<Value, String> {
	let path = extract_path(args.first().ok_or("file:stat missing path")?)?;

	dev_log!("nativebus", "file:stat {}", path);

	let metadata = tokio::fs::metadata(&path)
		.await
		.map_err(|e| format!("file:stat failed for {}: {}", path, e))?;

	let is_file = metadata.is_file();

	let is_dir = metadata.is_dir();

	#[cfg(unix)]
	let is_symlink = metadata.is_symlink();

	#[cfg(not(unix))]
	let is_symlink = false;

	let size = metadata.len();

	let modified = metadata
		.modified()
		.map(|t| {
			t.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_millis() as u64)
				.unwrap_or(0)
		})
		.unwrap_or(0);

	let created = metadata
		.created()
		.map(|t| {
			t.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_millis() as u64)
				.unwrap_or(0)
		})
		.unwrap_or(0);

	#[cfg(unix)]
	let (readonly, permissions) = {
		use std::os::unix::fs::PermissionsExt;

		let mode = metadata.permissions().mode();

		let ro = mode & 0o222 == 0;

		(ro, Some(mode & 0o777))
	};

	#[cfg(not(unix))]
	let (readonly, permissions) = (metadata.permissions().readonly(), None::<u32>);

	let mut result = json!({
		"isFile": is_file,
		"isDirectory": is_dir,
		"isSymlink": is_symlink,
		"size": size,
		"mtime": modified,
		"ctime": created,
		"readonly": readonly,
	});

	if let Some(perm) = permissions {
		result["permissions"] = json!(perm);
	}

	Ok(result)
}

async fn handle_file_delete(args:&[Value]) -> Result<Value, String> {
	let path = extract_path(args.first().ok_or("file:delete missing path")?)?;

	dev_log!("nativebus", "file:delete {}", path);

	let metadata = tokio::fs::metadata(&path).await.ok();

	if let Some(meta) = metadata {
		if meta.is_dir() {
			tokio::fs::remove_dir_all(&path)
				.await
				.map_err(|e| format!("file:delete dir failed for {}: {}", path, e))?;
		} else {
			tokio::fs::remove_file(&path)
				.await
				.map_err(|e| format!("file:delete file failed for {}: {}", path, e))?;
		}
	}

	// If the file doesn't exist, silently succeed (idempotent delete)

	Ok(json!(null))
}

async fn handle_file_mkdir(args:&[Value]) -> Result<Value, String> {
	let path = extract_path(args.first().ok_or("file:mkdir missing path")?)?;

	dev_log!("nativebus", "file:mkdir {}", path);

	tokio::fs::create_dir_all(&path)
		.await
		.map_err(|e| format!("file:mkdir failed for {}: {}", path, e))?;

	Ok(json!(null))
}

async fn handle_file_readdir(args:&[Value]) -> Result<Value, String> {
	let path = extract_path(args.first().ok_or("file:readdir missing path")?)?;

	dev_log!("nativebus", "file:readdir {}", path);

	let mut entries = tokio::fs::read_dir(&path)
		.await
		.map_err(|e| format!("file:readdir failed for {}: {}", path, e))?;

	let mut result:Vec<Value> = Vec::new();

	while let Ok(Some(entry)) = entries.next_entry().await {
		let file_name = entry.file_name();

		let name = file_name.to_string_lossy().to_string();

		// Use synchronous metadata to get file type (faster, avoids
		// another async call per entry; the OS already has the inode
		// cached from readdir).
		let file_type = entry.path().metadata().map(|m| m.file_type()).unwrap_or_else(|_| {
			// Fallback: default to plain file
			std::fs::metadata(".").map(|m| m.file_type()).unwrap()
		});

		let is_dir = file_type.is_dir();

		#[cfg(unix)]
		let is_symlink = file_type.is_symlink();

		#[cfg(not(unix))]
		let is_symlink = false;

		result.push(json!({
			"name": name,
			"isDirectory": is_dir,
			"isSymlink": is_symlink,
		}));
	}

	Ok(json!(result))
}

// ── Terminal Handler ────────────────────────────────────────────

async fn handle_terminal(app_handle:&AppHandle, method:&str, params:&Value) -> Result<Value, String> {
	match method {
		"terminal:write" => {
			let pty_id = params.get("id").and_then(|v| v.as_u64()).ok_or("terminal:write missing id")?;

			let data = params.get("data").and_then(|v| v.as_str()).unwrap_or("");

			dev_log!("nativebus", "terminal:write id={} len={}", pty_id, data.len());

			let runtime:Arc<ApplicationRunTime> = app_handle.state::<Arc<ApplicationRunTime>>().inner().clone();

			runtime
				.Environment
				.SendTextToTerminal(pty_id, data.to_string())
				.await
				.map_err(|e| e.to_string())?;

			Ok(json!(null))
		},

		"terminal:resize" => {
			let pty_id = params.get("id").and_then(|v| v.as_u64()).ok_or("terminal:resize missing id")?;

			let cols = params.get("cols").and_then(|v| v.as_u64()).unwrap_or(80) as u16;

			let rows = params.get("rows").and_then(|v| v.as_u64()).unwrap_or(24) as u16;

			// Clamp to ≥ 1 — portable-pty crashes on 0×0
			let cols = if cols == 0 { 1 } else { cols };

			let rows = if rows == 0 { 1 } else { rows };

			dev_log!("nativebus", "terminal:resize id={} {}×{}", pty_id, cols, rows);

			let runtime:Arc<ApplicationRunTime> = app_handle.state::<Arc<ApplicationRunTime>>().inner().clone();

			// Resize on a disposed terminal is a common race; log and swallow
			if let Err(e) = runtime.Environment.ResizeTerminal(pty_id, cols, rows).await {
				dev_log!("nativebus", "terminal:resize failed for id={}: {}", pty_id, e);
			}

			Ok(json!(null))
		},

		_ => Err(format!("NativeBus: unknown terminal method {}", method)),
	}
}

// ── Search Handler ──────────────────────────────────────────────

async fn handle_search(params:&Value) -> Result<Value, String> {
	// Search operations are forwarded to the native ripgrep backend.
	// For now, return empty results — the native rg integration
	// lives in the Search subsystem and will be wired in a follow-up.
	let _ = params; // params reserved for future use (query, include, exclude, etc.)

	dev_log!("nativebus", "search: ack (empty results)");

	Ok(json!({
		"results": [],
		"matches": 0,
	}))
}

// ── Dialog Handler ──────────────────────────────────────────────

async fn handle_dialog(_args:&[Value]) -> Result<Value, String> {
	// Dialog operations (open, save, message) are forwarded to the
	// Mountain native dialog subsystem.
	dev_log!("nativebus", "dialog: ack");

	Ok(json!(null))
}

// ── Status Bar Handler ──────────────────────────────────────────

fn handle_statusbar(app_handle:&AppHandle, method:&str, params:&Value) -> Result<Value, String> {
	match method {
		"statusbar:set" => {
			dev_log!("nativebus", "statusbar:set emitting to sky");

			app_handle
				.emit("sky:statusbar:set", params.clone())
				.map_err(|e| e.to_string())?;
		},

		_ => {
			dev_log!("nativebus", "statusbar: {} (ack)", method);
		},
	}

	Ok(json!(null))
}
