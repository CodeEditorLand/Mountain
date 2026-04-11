#![allow(unused_variables, dead_code)]

//! # Wind Service Handlers - Cross-Language Service Bridge
//!
//! **File Responsibilities:**
//! This module provides the direct mapping layer between Wind's TypeScript
//! service invocations and Mountain's Rust service implementations. It acts as
//! the critical translation layer that enables Wind to request operations from
//! Mountain through Tauri's IPC mechanism.
//!
//! **Architectural Role in Wind-Mountain Connection:**
//!
//! The WindServiceHandlers module implements the concrete command handlers that
//! process IPC invocations from Wind. It serves as the single entry point for
//! all Wind->Mountain service requests:
//!
//! 1. **Command Mapping:** Maps Wind's TypeScript service methods to Rust
//!    implementations
//! 2. **Type Conversion:** Converts between JSON/TypeScript types and Rust
//!    types
//! 3. **Validation:** Validates all inputs before forwarding to Mountain
//!    services
//! 4. **Error Handling:** Provides comprehensive error messages back to Wind
//! 5. **Service Integration:** Connects to Mountain's internal service
//!    architecture
//!
//! **Handled Command Categories:**
//!
//! **1. Configuration Commands:**
//! - `configuration:get` - Retrieve configuration values
//! - `configuration:update` - Update configuration values
//!
//! **2. File System Commands:**
//! - `file:read` - Read file contents
//! - `file:write` - Write to files
//! - `file:stat` - Get file metadata
//! - `file:exists` - Check file existence
//! - `file:delete` - Delete files or directories
//! - `file:copy` - Copy files
//! - `file:move` - Move/rename files
//! - `file:mkdir` - Create directories
//! - `file:readdir` - Read directory contents
//! - `file:readBinary` - Read binary files
//! - `file:writeBinary` - Write binary files
//!
//! **3. Storage Commands:**
//! - `storage:get` - Retrieve persistent storage values
//! - `storage:set` - Store persistent values
//!
//! **4. Environment Commands:**
//! - `environment:get` - Get environment variables
//!
//! **5. Native Host Commands:**
//! - `native:showItemInFolder` - Reveal file in system file manager
//! - `native:openExternal` - Open URLs in external browser
//!
//! **6. Workbench Commands:**
//! - `workbench:getConfiguration` - Get complete workbench configuration
//!
//! **7. IPC Status Commands:**
//! - `mountain_get_status` - Get overall IPC system status
//! - `mountain_get_configuration` - Get Mountain configuration snapshot
//! - `mountain_get_services_status` - Get status of all Mountain services
//! - `mountain_get_state` - Get current application state
//!
//! **Communication Pattern:**
//!
//! ```text
//! Wind (TypeScript)
//!   |
//!   | app.handle.invoke('command', args)
//!   v
//! Tauri Bridge (IPC)
//!   |
//!   | mountain_ipc_invoke(command, args)
//!   v
//! WindServiceHandlers
//!   |
//!   | Type conversion + validation
//!   v
//! Mountain Services (Rust)
//!   |
//!   | Execute operation
//!   v
//! Return Result<serde_json::Value>
//! ```
//!
//! **Type Conversion Strategy (TypeScript <-> Rust):**
//!
//! **Primitive Types:**
//! - TypeScript `string` ↔ Rust `String` / `&str`
//! - TypeScript `number` ↔ Rust `f64` / `i32` / `u32`
//! - TypeScript `boolean` ↔ Rust `bool`
//! - TypeScript `null` ↔ Rust `Option::<T>::None`
//!
//! **Complex Types:**
//! - TypeScript `object` ↔ Rust `serde_json::Value` / `HashMap`
//! - TypeScript `Array<T>` ↔ Rust `Vec<T>`
//! - TypeScript custom interfaces ↔ Rust structs with Serialize/Deserialize
//!
//! **Example Type Conversion:**
//! ```typescript
//! // Wind (TypeScript)
//! interface FileReadOptions {
//!   encoding: 'utf8' | 'binary';
//!   withBOM: boolean;
//! }
//! const result = await invoke('file:read', {
//!   path: '/path/to/file.txt',
//!   options: { encoding: 'utf8', withBOM: false }
//! });
//! ```
//!
//! ```text
//! // Mountain (Rust)
//! args.get(0).and_then(|v| v.as_str()) // Extract path
//! args.get(1).and_then(|v| v.as_object()) // Extract options
//! // ... validation, processing, return Result
//! ```
//!
//! **Defensive Error Handling:**
//!
//! Each handler implements comprehensive error handling:
//!
//! 1. **Input Validation:**
//!    - Check parameter presence
//!    - Validate parameter types
//!    - Validate value ranges and formats
//!
//! 2. **Service Error Handling:**
//!    - Catch and translate service errors
//!    - Provide detailed error messages
//!    - Include context for debugging
//!
//! 3. **Error Response Format:**
//! ```rust
//! Error("Failed to read file: Permission denied (path: /etc/passwd)") 
//! ```
//!
//! **Comprehensive Error Messages:**
//! - Include operation that failed
//! - Include relevant parameters (paths, keys, etc.)
//! - Include the underlying cause
//! - Format: `"Failed to <operation>: <cause> (context: <value>)"`
//!
//! **Service Integration Pattern:**
//!
//! Handlers use Mountain's dependency injection system via `Requires` trait:
//!
//! ```text
//! let provider: Arc<dyn ConfigurationProvider> = runtime.Environment.Require();
//! provider.GetConfigurationValue(...).await?;
//! ```
//!
//! This provides:
//! - Loose coupling between handlers and services
//! - Testable architecture (can mock services)
//! - Centralized service lifecycle management
//!
//! **Command Registration:**
//!
//! All handlers are automatically registered when included in Tauri's
//! invoke_handler:
//!
//! ```rust
//! .invoke_handler(tauri::generate_handler![
//!     mountain_ipc_invoke,
//!     // ... other commands
//! ])
//! ```

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
// Type aliases for Configuration DTOs to simplify usage
use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};

use crate::dev_log;
type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;
type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Configuration::ConfigurationProvider::ConfigurationProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	Storage::StorageProvider::StorageProvider,
};

use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Extract a filesystem path from a VS Code argument.
/// VS Code sends URI objects `{ scheme: "file", path: "/C:/foo", fsPath:
/// "C:\\foo" }` but Mountain handlers expect platform-native path strings.
///
/// Windows URI paths have a leading slash: `/C:/Users/...` → strip it.
/// Unix paths start with `/` normally.
fn extract_path_from_arg(Arg:&Value) -> Result<String, String> {
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
	// Wind's configuration sets profiles.home = { scheme: "vscode-userdata", path:
	// "/User" } VS Code's FileUserDataProvider converts vscode-userdata: → file:
	// but keeps the path. We need to prepend the real userdata directory.
	let Resolved = resolve_userdata_path(&Decoded);

	// Map /Static/Application/... → real Sky Target directory.
	// VS Code computes builtinExtensionsPath as appRoot + "/../extensions"
	// which resolves to /Static/Application/extensions. Since appRoot is a
	// URL path served by Vite/Tauri, we map it to the real filesystem.
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
/// /User/profiles/__default__profile__/... → ~/.land/User/profiles/...
/// Paths not starting with /User/ are returned unchanged.
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
/// directory. VS Code's environmentService computes builtinExtensionsPath as
/// `join(FileAccess.asFileUri("").fsPath, "..", "extensions")` which resolves
/// to `/Static/Application/extensions`. Since /Static/Application/ is a URL
/// path (not a real filesystem path), we map it to the actual Sky Target
/// directory where the VS Code assets are served from.
///
/// The Sky Target directory is determined relative to the executable's location
/// in dev mode, or from Tauri's resource directory in production.
static STATIC_APPLICATION_ROOT:std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Set the real filesystem root for /Static/Application/ paths.
/// Call once at startup with the Sky Target directory.
pub fn set_static_application_root(Path:String) { let _ = STATIC_APPLICATION_ROOT.set(Path); }

fn resolve_static_application_path(Path:&str) -> String {
	if !Path.starts_with("/Static/Application/") && Path != "/Static/Application" {
		return Path.to_string();
	}

	if let Some(Root) = STATIC_APPLICATION_ROOT.get() {
		let Relative = Path.strip_prefix("/Static/Application").unwrap_or("");
		let Resolved = format!("{}/Static/Application{}", Root, Relative);
		dev_log!("vfs", "resolve_static: {} -> {}", Path, Resolved);
		Resolved
	} else {
		// Fallback: return path unchanged (will fail with ENOENT)
		Path.to_string()
	}
}

/// Canonical userdata base directory, set once from Tauri's PathResolver.
/// All /User/... paths resolve against this so the VFS mapping matches
/// the real Tauri app_data_dir (which includes the full bundle identifier).
static USERDATA_BASE_DIR:std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Session timestamp, generated once per process lifetime so every call to
/// `getEnvironmentPaths` returns the same `logsPath`.
static SESSION_TIMESTAMP:std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Set the userdata base from Tauri's app_data_dir. Call once at startup.
pub fn set_userdata_base_dir(Path:String) { let _ = USERDATA_BASE_DIR.set(Path); }

/// Get the base directory for userdata storage.
/// Returns the Tauri-resolved path if available, otherwise a fallback.
fn get_userdata_base_dir() -> String {
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

/// Decode percent-encoded characters in URI paths.
/// Handles: %20 (space), %23 (#), %25 (%), %5B ([), %5D (]), etc.
fn percent_decode(Input:&str) -> String {
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

/// Convert a `std::fs::Metadata` to VS Code's `IStat` shape.
fn metadata_to_istat(Metadata:&std::fs::Metadata) -> Value {
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

/// Ensure userdata directories exist on first access.
/// Creates ~/.land/User/ and subdirectories that VS Code expects.
static USERDATA_INITIALIZED:std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn ensure_userdata_dirs() {
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

/// Handler for Wind's MainProcessService.invoke() calls
/// Maps Tauri IPC commands to Mountain's internal command system
#[tauri::command]
pub async fn mountain_ipc_invoke(app_handle:AppHandle, command:String, args:Vec<Value>) -> Result<Value, String> {
	dev_log!("ipc", "invoke: {} args_count={}", command, args.len());

	// Ensure userdata directories exist on first IPC call
	ensure_userdata_dirs();

	// Get the application runtime
	let runtime = app_handle.state::<Arc<ApplicationRunTime>>();

	// =========================================================================
	// Route dispatch - every arm has a dev_log! with a granular tag.
	// Tags match the route prefix: vfs, config, storage, extensions,
	// terminal, output, textfile, notification, progress, quickinput,
	// workspaces, themes, search, decorations, workingcopy, keybinding,
	// lifecycle, label, model, history, commands, nativehost, window,
	// exthost, encryption, menubar, update, url, grpc.
	// Activate: LAND_DEV_LOG=all   or   LAND_DEV_LOG=vfs,ipc,config
	// =========================================================================

	match command.as_str() {
		// Configuration commands
		"configuration:get" => {
			dev_log!("config", "configuration:get");
			handle_configuration_get(runtime.inner().clone(), args).await
		},
		"configuration:update" => {
			dev_log!("config", "configuration:update");
			handle_configuration_update(runtime.inner().clone(), args).await
		},

		// Logger commands - fire-and-forget from Wind, just acknowledge
		"logger:log"
		| "logger:warn"
		| "logger:error"
		| "logger:info"
		| "logger:debug"
		| "logger:trace"
		| "logger:critical"
		| "logger:flush"
		| "logger:setLevel"
		| "logger:getLevel"
		| "logger:createLogger"
		| "logger:registerLogger"
		| "logger:deregisterLogger"
		| "logger:getRegisteredLoggers"
		| "logger:setVisibility" => Ok(Value::Null),

		// File system commands - use native handlers with URI support
		"file:read" => handle_file_read_native(args).await,
		"file:write" => handle_file_write_native(args).await,
		"file:stat" => handle_file_stat_native(args).await,
		"file:exists" => handle_file_exists_native(args).await,
		"file:delete" => handle_file_delete_native(args).await,
		"file:copy" => handle_file_clone_native(args).await,
		"file:move" => handle_file_rename_native(args).await,
		"file:mkdir" => handle_file_mkdir_native(args).await,
		"file:readdir" => handle_file_readdir_native(args).await,
		"file:readBinary" => handle_file_read_binary(runtime.inner().clone(), args).await,
		"file:writeBinary" => handle_file_write_binary(runtime.inner().clone(), args).await,

		// Storage commands
		"storage:get" => handle_storage_get(runtime.inner().clone(), args).await,
		"storage:set" => handle_storage_set(runtime.inner().clone(), args).await,
		"storage:getItems" => {
			dev_log!("storage", "storage:getItems");
			handle_storage_get_items(runtime.inner().clone(), args).await
		},
		"storage:updateItems" => {
			dev_log!("storage", "storage:updateItems");
			handle_storage_update_items(runtime.inner().clone(), args).await
		},
		"storage:optimize" => {
			dev_log!("storage", "storage:optimize");
			Ok(Value::Null)
		},
		"storage:isUsed" => {
			dev_log!("storage", "storage:isUsed");
			Ok(Value::Null)
		},
		"storage:close" => {
			dev_log!("storage", "storage:close");
			Ok(Value::Null)
		},

		// Environment commands
		"environment:get" => {
			dev_log!("config", "environment:get");
			handle_environment_get(runtime.inner().clone(), args).await
		},

		// Native host commands
		"native:showItemInFolder" => handle_show_item_in_folder(runtime.inner().clone(), args).await,
		"native:openExternal" => handle_open_external(runtime.inner().clone(), args).await,

		// Workbench commands
		"workbench:getConfiguration" => handle_workbench_configuration(runtime.inner().clone(), args).await,

		// Command registry commands
		"commands:execute" => handle_commands_execute(runtime.inner().clone(), args).await,
		"commands:getAll" => {
			dev_log!("commands", "commands:getAll");
			handle_commands_get_all(runtime.inner().clone()).await
		},

		// Extension host commands
		"extensions:getAll" => {
			dev_log!("extensions", "extensions:getAll");
			handle_extensions_get_all(runtime.inner().clone()).await
		},
		"extensions:get" => {
			dev_log!("extensions", "extensions:get");
			handle_extensions_get(runtime.inner().clone(), args).await
		},
		"extensions:isActive" => {
			dev_log!("extensions", "extensions:isActive");
			handle_extensions_is_active(runtime.inner().clone(), args).await
		},

		// Terminal commands
		"terminal:create" => {
			dev_log!("terminal", "terminal:create");
			handle_terminal_create(runtime.inner().clone(), args).await
		},
		"terminal:sendText" => {
			dev_log!("terminal", "terminal:sendText");
			handle_terminal_send_text(runtime.inner().clone(), args).await
		},
		"terminal:dispose" => {
			dev_log!("terminal", "terminal:dispose");
			handle_terminal_dispose(runtime.inner().clone(), args).await
		},
		"terminal:show" => {
			dev_log!("terminal", "terminal:show");
			handle_terminal_show(runtime.inner().clone(), args).await
		},
		"terminal:hide" => {
			dev_log!("terminal", "terminal:hide");
			handle_terminal_hide(runtime.inner().clone(), args).await
		},

		// Output channel commands
		"output:create" => handle_output_create(app_handle.clone(), args).await,
		"output:append" => {
			dev_log!("output", "output:append");
			handle_output_append(app_handle.clone(), args).await
		},
		"output:appendLine" => {
			dev_log!("output", "output:appendLine");
			handle_output_append_line(app_handle.clone(), args).await
		},
		"output:clear" => {
			dev_log!("output", "output:clear");
			handle_output_clear(app_handle.clone(), args).await
		},
		"output:show" => {
			dev_log!("output", "output:show");
			handle_output_show(app_handle.clone(), args).await
		},

		// TextFile commands
		"textFile:read" => {
			dev_log!("textfile", "textFile:read");
			handle_textfile_read(runtime.inner().clone(), args).await
		},
		"textFile:write" => {
			dev_log!("textfile", "textFile:write");
			handle_textfile_write(runtime.inner().clone(), args).await
		},
		"textFile:save" => handle_textfile_save(runtime.inner().clone(), args).await,

		// Storage commands (additional)
		"storage:delete" => {
			dev_log!("storage", "storage:delete");
			handle_storage_delete(runtime.inner().clone(), args).await
		},
		"storage:keys" => {
			dev_log!("storage", "storage:keys");
			handle_storage_keys(runtime.inner().clone()).await
		},

		// Notification commands (emit sky:// events for Sky to render)
		"notification:show" => {
			dev_log!("notification", "notification:show");
			handle_notification_show(app_handle.clone(), args).await
		},
		"notification:showProgress" => {
			dev_log!("notification", "notification:showProgress");
			handle_notification_show_progress(app_handle.clone(), args).await
		},
		"notification:updateProgress" => {
			dev_log!("notification", "notification:updateProgress");
			handle_notification_update_progress(app_handle.clone(), args).await
		},
		"notification:endProgress" => {
			dev_log!("notification", "notification:endProgress");
			handle_notification_end_progress(app_handle.clone(), args).await
		},

		// Progress commands
		"progress:begin" => {
			dev_log!("progress", "progress:begin");
			handle_progress_begin(app_handle.clone(), args).await
		},
		"progress:report" => {
			dev_log!("progress", "progress:report");
			handle_progress_report(app_handle.clone(), args).await
		},
		"progress:end" => {
			dev_log!("progress", "progress:end");
			handle_progress_end(app_handle.clone(), args).await
		},

		// QuickInput commands
		"quickInput:showQuickPick" => {
			dev_log!("quickinput", "quickInput:showQuickPick");
			handle_quick_input_show_quick_pick(runtime.inner().clone(), args).await
		},
		"quickInput:showInputBox" => {
			dev_log!("quickinput", "quickInput:showInputBox");
			handle_quick_input_show_input_box(runtime.inner().clone(), args).await
		},

		// Workspaces commands
		"workspaces:getFolders" => {
			dev_log!("workspaces", "workspaces:getFolders");
			handle_workspaces_get_folders(runtime.inner().clone()).await
		},
		"workspaces:addFolder" => {
			dev_log!("workspaces", "workspaces:addFolder");
			handle_workspaces_add_folder(runtime.inner().clone(), args).await
		},
		"workspaces:removeFolder" => {
			dev_log!("workspaces", "workspaces:removeFolder");
			handle_workspaces_remove_folder(runtime.inner().clone(), args).await
		},
		"workspaces:getName" => {
			dev_log!("workspaces", "workspaces:getName");
			handle_workspaces_get_name(runtime.inner().clone()).await
		},

		// Themes commands
		"themes:getActive" => {
			dev_log!("themes", "themes:getActive");
			handle_themes_get_active(runtime.inner().clone()).await
		},
		"themes:list" => {
			dev_log!("themes", "themes:list");
			handle_themes_list(runtime.inner().clone()).await
		},
		"themes:set" => {
			dev_log!("themes", "themes:set");
			handle_themes_set(runtime.inner().clone(), args).await
		},

		// Search commands
		"search:findInFiles" => {
			dev_log!("search", "search:findInFiles");
			handle_search_find_in_files(runtime.inner().clone(), args).await
		},
		"search:findFiles" => {
			dev_log!("search", "search:findFiles");
			handle_search_find_files(runtime.inner().clone(), args).await
		},

		// Decorations commands
		"decorations:get" => {
			dev_log!("decorations", "decorations:get");
			handle_decorations_get(runtime.inner().clone(), args).await
		},
		"decorations:getMany" => {
			dev_log!("decorations", "decorations:getMany");
			handle_decorations_get_many(runtime.inner().clone(), args).await
		},
		"decorations:set" => {
			dev_log!("decorations", "decorations:set");
			handle_decorations_set(runtime.inner().clone(), args).await
		},
		"decorations:clear" => {
			dev_log!("decorations", "decorations:clear");
			handle_decorations_clear(runtime.inner().clone(), args).await
		},

		// WorkingCopy commands
		"workingCopy:isDirty" => {
			dev_log!("workingcopy", "workingCopy:isDirty");
			handle_working_copy_is_dirty(runtime.inner().clone(), args).await
		},
		"workingCopy:setDirty" => {
			dev_log!("workingcopy", "workingCopy:setDirty");
			handle_working_copy_set_dirty(runtime.inner().clone(), args).await
		},
		"workingCopy:getAllDirty" => {
			dev_log!("workingcopy", "workingCopy:getAllDirty");
			handle_working_copy_get_all_dirty(runtime.inner().clone()).await
		},
		"workingCopy:getDirtyCount" => {
			dev_log!("workingcopy", "workingCopy:getDirtyCount");
			handle_working_copy_get_dirty_count(runtime.inner().clone()).await
		},

		// Keybinding commands
		"keybinding:add" => {
			dev_log!("keybinding", "keybinding:add");
			handle_keybinding_add(runtime.inner().clone(), args).await
		},
		"keybinding:remove" => {
			dev_log!("keybinding", "keybinding:remove");
			handle_keybinding_remove(runtime.inner().clone(), args).await
		},
		"keybinding:lookup" => {
			dev_log!("keybinding", "keybinding:lookup");
			handle_keybinding_lookup(runtime.inner().clone(), args).await
		},
		"keybinding:getAll" => {
			dev_log!("keybinding", "keybinding:getAll");
			handle_keybinding_get_all(runtime.inner().clone()).await
		},

		// Lifecycle commands
		"lifecycle:getPhase" => {
			dev_log!("lifecycle", "lifecycle:getPhase");
			handle_lifecycle_get_phase(runtime.inner().clone()).await
		},
		"lifecycle:whenPhase" => {
			dev_log!("lifecycle", "lifecycle:whenPhase");
			handle_lifecycle_when_phase(runtime.inner().clone(), args).await
		},
		"lifecycle:requestShutdown" => {
			dev_log!("lifecycle", "lifecycle:requestShutdown");
			handle_lifecycle_request_shutdown(app_handle.clone()).await
		},

		// Label commands
		"label:getUri" => {
			dev_log!("label", "label:getUri");
			handle_label_get_uri(runtime.inner().clone(), args).await
		},
		"label:getWorkspace" => {
			dev_log!("label", "label:getWorkspace");
			handle_label_get_workspace(runtime.inner().clone()).await
		},
		"label:getBase" => {
			dev_log!("label", "label:getBase");
			handle_label_get_base(args).await
		},

		// Model (text model registry) commands
		"model:open" => {
			dev_log!("model", "model:open");
			handle_model_open(runtime.inner().clone(), args).await
		},
		"model:close" => {
			dev_log!("model", "model:close");
			handle_model_close(runtime.inner().clone(), args).await
		},
		"model:get" => {
			dev_log!("model", "model:get");
			handle_model_get(runtime.inner().clone(), args).await
		},
		"model:getAll" => {
			dev_log!("model", "model:getAll");
			handle_model_get_all(runtime.inner().clone()).await
		},
		"model:updateContent" => {
			dev_log!("model", "model:updateContent");
			handle_model_update_content(runtime.inner().clone(), args).await
		},

		// Navigation history commands
		"history:goBack" => {
			dev_log!("history", "history:goBack");
			handle_history_go_back(runtime.inner().clone()).await
		},
		"history:goForward" => {
			dev_log!("history", "history:goForward");
			handle_history_go_forward(runtime.inner().clone()).await
		},
		"history:canGoBack" => {
			dev_log!("history", "history:canGoBack");
			handle_history_can_go_back(runtime.inner().clone()).await
		},
		"history:canGoForward" => {
			dev_log!("history", "history:canGoForward");
			handle_history_can_go_forward(runtime.inner().clone()).await
		},
		"history:push" => {
			dev_log!("history", "history:push");
			handle_history_push(runtime.inner().clone(), args).await
		},
		"history:clear" => {
			dev_log!("history", "history:clear");
			handle_history_clear(runtime.inner().clone()).await
		},
		"history:getStack" => {
			dev_log!("history", "history:getStack");
			handle_history_get_stack(runtime.inner().clone()).await
		},

		// IPC status commands
		"mountain_get_status" => {
			let status = json!({
				"connected": true,
				"version": "1.0.0"
			});
			Ok(status)
		},
		"mountain_get_configuration" => {
			let config = json!({
				"editor": { "theme": "dark" },
				"extensions": { "installed": [] }
			});
			Ok(config)
		},
		"mountain_get_services_status" => {
			let services = json!({
				"editor": { "status": "running" },
				"extensionHost": { "status": "running" }
			});
			Ok(services)
		},
		"mountain_get_state" => {
			let state = json!({
				"ui": {},
				"editor": {},
				"workspace": {}
			});
			Ok(state)
		},

		// =====================================================================
		// File system command ALIASES
		// VS Code's DiskFileSystemProviderClient calls readFile/writeFile/rename
		// but Mountain's original handlers use read/write/move.
		// =====================================================================
		"file:readFile" => handle_file_read_native(args).await,
		"file:writeFile" => handle_file_write_native(args).await,
		"file:rename" => handle_file_rename_native(args).await,
		"file:realpath" => handle_file_realpath(args).await,
		"file:watch" => {
			dev_log!("vfs", "file:watch stub - no-op");
			Ok(Value::Null)
		},
		"file:unwatch" => {
			dev_log!("vfs", "file:unwatch stub - no-op");
			Ok(Value::Null)
		},
		"file:open" => {
			dev_log!("vfs", "file:open stub - no fd support yet");
			Ok(json!(0))
		},
		"file:close" => {
			dev_log!("vfs", "file:close stub");
			Ok(Value::Null)
		},
		"file:cloneFile" => handle_file_clone_native(args).await,

		// =====================================================================
		// Native Host commands (INativeHostService)
		// =====================================================================

		// Dialogs
		"nativeHost:pickFolderAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
		"nativeHost:pickFileAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
		"nativeHost:pickFileFolderAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
		"nativeHost:pickWorkspaceAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
		"nativeHost:showOpenDialog" => handle_native_show_open_dialog(app_handle.clone(), args).await,
		"nativeHost:showSaveDialog" => Ok(json!({ "canceled": true })),
		"nativeHost:showMessageBox" => Ok(json!({ "response": 0 })),

		// Environment paths - called by ResolveConfiguration to get real Tauri paths.
		// Returns the session log directory (with timestamp + window1 subdir)
		// so VS Code can immediately write output files without stat errors.
		"nativeHost:getEnvironmentPaths" => {
			let PathResolver = app_handle.path();
			let AppDataDir = PathResolver.app_data_dir().unwrap_or_default();
			let HomeDir = PathResolver.home_dir().unwrap_or_default();
			let TmpDir = std::env::temp_dir();

			// Logs go under {appDataDir}/logs/{sessionTimestamp}/ - same tree as
			// all other VS Code data, not Tauri's separate app_log_dir().
			// VS Code requires a session-timestamped subdir for log rotation.
			// The timestamp is generated ONCE per process so every call returns
			// the same logsPath (prevents VS Code from seeing stale dirs).
			let SessionTimestamp =
				SESSION_TIMESTAMP.get_or_init(|| chrono::Local::now().format("%Y%m%dT%H%M%S").to_string());
			let SessionLogRoot = AppDataDir.join("logs").join(SessionTimestamp);
			let SessionLogWindowDir = SessionLogRoot.join("window1");
			let _ = std::fs::create_dir_all(&SessionLogWindowDir);

			dev_log!(
				"config",
				"getEnvironmentPaths: userDataDir={} logsPath={} homeDir={}",
				AppDataDir.display(),
				SessionLogRoot.display(),
				HomeDir.display()
			);
			let DevLogEnv = std::env::var("LAND_DEV_LOG").unwrap_or_default();
			Ok(json!({
				"userDataDir": AppDataDir.to_string_lossy(),
				"logsPath": SessionLogRoot.to_string_lossy(),
				"homeDir": HomeDir.to_string_lossy(),
				"tmpDir": TmpDir.to_string_lossy(),
				"devLog": if DevLogEnv.is_empty() { Value::Null } else { json!(DevLogEnv) },
			}))
		},

		// OS info
		"nativeHost:getOSColorScheme" => {
			dev_log!("nativehost", "nativeHost:getOSColorScheme");
			handle_native_get_color_scheme().await
		},
		"nativeHost:getOSProperties" => {
			dev_log!("nativehost", "nativeHost:getOSProperties");
			handle_native_os_properties().await
		},
		"nativeHost:getOSStatistics" => {
			dev_log!("nativehost", "nativeHost:getOSStatistics");
			handle_native_os_statistics().await
		},
		"nativeHost:getOSVirtualMachineHint" => {
			dev_log!("nativehost", "nativeHost:getOSVirtualMachineHint");
			Ok(json!(0))
		},

		// Window state
		"nativeHost:isWindowAlwaysOnTop" => {
			dev_log!("window", "nativeHost:isWindowAlwaysOnTop");
			Ok(json!(false))
		},
		"nativeHost:isFullScreen" => {
			dev_log!("window", "nativeHost:isFullScreen");
			handle_native_is_fullscreen(app_handle.clone()).await
		},
		"nativeHost:isMaximized" => {
			dev_log!("window", "nativeHost:isMaximized");
			handle_native_is_maximized(app_handle.clone()).await
		},
		"nativeHost:getActiveWindowId" => {
			dev_log!("window", "nativeHost:getActiveWindowId");
			Ok(json!(1))
		},
		"nativeHost:getWindows" => Ok(json!([{ "id": 1, "title": "Land", "filename": "" }])),
		"nativeHost:getWindowCount" => Ok(json!(1)),

		// Window control (fire-and-forget)
		"nativeHost:updateWindowControls"
		| "nativeHost:setMinimumSize"
		| "nativeHost:notifyReady"
		| "nativeHost:saveWindowSplash"
		| "nativeHost:updateTouchBar"
		| "nativeHost:focusWindow"
		| "nativeHost:maximizeWindow"
		| "nativeHost:minimizeWindow"
		| "nativeHost:unmaximizeWindow"
		| "nativeHost:moveWindowTop"
		| "nativeHost:positionWindow"
		| "nativeHost:toggleFullScreen"
		| "nativeHost:setWindowAlwaysOnTop"
		| "nativeHost:toggleWindowAlwaysOnTop"
		| "nativeHost:closeWindow"
		| "nativeHost:setDocumentEdited"
		| "nativeHost:setRepresentedFilename"
		| "nativeHost:setBackgroundThrottling"
		| "nativeHost:updateWindowAccentColor" => {
			dev_log!("window", "{}", command);
			Ok(Value::Null)
		},

		// OS operations
		"nativeHost:isAdmin" => Ok(json!(false)),
		"nativeHost:isRunningUnderARM64Translation" => {
			#[cfg(target_os = "macos")]
			{
				// macOS: check if running under Rosetta 2
				let Output = std::process::Command::new("sysctl")
					.args(["-n", "sysctl.proc_translated"])
					.output();
				let IsTranslated = Output
					.ok()
					.map(|O| String::from_utf8_lossy(&O.stdout).trim() == "1")
					.unwrap_or(false);
				Ok(json!(IsTranslated))
			}
			#[cfg(not(target_os = "macos"))]
			{
				Ok(json!(false))
			}
		},
		"nativeHost:hasWSLFeatureInstalled" => {
			#[cfg(target_os = "windows")]
			{
				Ok(json!(std::path::Path::new("C:\\Windows\\System32\\wsl.exe").exists()))
			}
			#[cfg(not(target_os = "windows"))]
			{
				Ok(json!(false))
			}
		},
		"nativeHost:showItemInFolder" => handle_show_item_in_folder(runtime.inner().clone(), args).await,
		"nativeHost:openExternal" => handle_open_external(runtime.inner().clone(), args).await,
		"nativeHost:moveItemToTrash" => Ok(Value::Null),

		// Clipboard
		"nativeHost:readClipboardText" => {
			dev_log!("clipboard", "readClipboardText");
			Ok(json!(""))
		},
		"nativeHost:writeClipboardText" => {
			dev_log!("clipboard", "writeClipboardText");
			Ok(Value::Null)
		},
		"nativeHost:readClipboardFindText" => {
			dev_log!("clipboard", "readClipboardFindText");
			Ok(json!(""))
		},
		"nativeHost:writeClipboardFindText" => {
			dev_log!("clipboard", "writeClipboardFindText");
			Ok(Value::Null)
		},
		"nativeHost:readClipboardBuffer" => {
			dev_log!("clipboard", "readClipboardBuffer");
			Ok(json!([]))
		},
		"nativeHost:writeClipboardBuffer" => {
			dev_log!("clipboard", "writeClipboardBuffer");
			Ok(Value::Null)
		},
		"nativeHost:hasClipboard" => {
			dev_log!("clipboard", "hasClipboard");
			Ok(json!(false))
		},
		"nativeHost:readImage" => {
			dev_log!("clipboard", "readImage");
			Ok(json!([]))
		},
		"nativeHost:triggerPaste" => {
			dev_log!("clipboard", "triggerPaste");
			Ok(Value::Null)
		},

		// Process
		"nativeHost:getProcessId" => Ok(json!(std::process::id())),
		"nativeHost:killProcess" => Ok(Value::Null),

		// Network
		"nativeHost:findFreePort" => handle_native_find_free_port(args).await,
		"nativeHost:isPortFree" => Ok(json!(true)),
		"nativeHost:resolveProxy" => Ok(Value::Null),
		"nativeHost:lookupAuthorization" => Ok(Value::Null),
		"nativeHost:lookupKerberosAuthorization" => Ok(Value::Null),
		"nativeHost:loadCertificates" => Ok(json!([])),

		// Lifecycle
		"nativeHost:relaunch" => Ok(Value::Null),
		"nativeHost:reload" => Ok(Value::Null),
		"nativeHost:quit" => Ok(Value::Null),
		"nativeHost:exit" => Ok(Value::Null),

		// Dev tools
		"nativeHost:openDevTools" => Ok(Value::Null),
		"nativeHost:toggleDevTools" => Ok(Value::Null),

		// Power
		"nativeHost:getSystemIdleState" => Ok(json!("active")),
		"nativeHost:getSystemIdleTime" => Ok(json!(0)),
		"nativeHost:getCurrentThermalState" => Ok(json!("nominal")),
		"nativeHost:isOnBatteryPower" => Ok(json!(false)),
		"nativeHost:startPowerSaveBlocker" => Ok(json!(0)),
		"nativeHost:stopPowerSaveBlocker" => Ok(json!(false)),
		"nativeHost:isPowerSaveBlockerStarted" => Ok(json!(false)),

		// macOS specific
		"nativeHost:newWindowTab" => Ok(Value::Null),
		"nativeHost:showPreviousWindowTab" => Ok(Value::Null),
		"nativeHost:showNextWindowTab" => Ok(Value::Null),
		"nativeHost:moveWindowTabToNewWindow" => Ok(Value::Null),
		"nativeHost:mergeAllWindowTabs" => Ok(Value::Null),
		"nativeHost:toggleWindowTabsBar" => Ok(Value::Null),
		"nativeHost:installShellCommand" => Ok(Value::Null),
		"nativeHost:uninstallShellCommand" => Ok(Value::Null),

		// =====================================================================
		// Local PTY (terminal) commands
		// =====================================================================
		"localPty:getProfiles" => {
			dev_log!("terminal", "localPty:getProfiles");
			handle_local_pty_get_profiles().await
		},
		"localPty:getDefaultSystemShell" => {
			dev_log!("terminal", "localPty:getDefaultSystemShell");
			handle_local_pty_get_default_shell().await
		},
		"localPty:getTerminalLayoutInfo" => {
			dev_log!("terminal", "localPty:getTerminalLayoutInfo");
			Ok(Value::Null)
		},
		"localPty:setTerminalLayoutInfo" => {
			dev_log!("terminal", "localPty:setTerminalLayoutInfo");
			Ok(Value::Null)
		},
		"localPty:getPerformanceMarks" => {
			dev_log!("terminal", "localPty:getPerformanceMarks");
			Ok(json!([]))
		},
		"localPty:reduceConnectionGraceTime" => {
			dev_log!("terminal", "localPty:reduceConnectionGraceTime");
			Ok(Value::Null)
		},
		"localPty:listProcesses" => {
			dev_log!("terminal", "localPty:listProcesses");
			Ok(json!([]))
		},
		"localPty:getEnvironment" => {
			dev_log!("terminal", "localPty:getEnvironment");
			handle_local_pty_get_environment().await
		},

		// =====================================================================
		// Update service
		// =====================================================================
		"update:_getInitialState" => {
			dev_log!("update", "update:_getInitialState");
			Ok(json!({ "type": "idle", "updateType": 0 }))
		},
		"update:isLatestVersion" => {
			dev_log!("update", "update:isLatestVersion");
			Ok(json!(true))
		},
		"update:checkForUpdates" => {
			dev_log!("update", "update:checkForUpdates");
			Ok(Value::Null)
		},
		"update:downloadUpdate" => {
			dev_log!("update", "update:downloadUpdate");
			Ok(Value::Null)
		},
		"update:applyUpdate" => {
			dev_log!("update", "update:applyUpdate");
			Ok(Value::Null)
		},
		"update:quitAndInstall" => {
			dev_log!("update", "update:quitAndInstall");
			Ok(Value::Null)
		},

		// =====================================================================
		// Menubar
		// =====================================================================
		"menubar:updateMenubar" => {
			dev_log!("menubar", "menubar:updateMenubar");
			Ok(Value::Null)
		},

		// =====================================================================
		// URL handler
		// =====================================================================
		"url:registerExternalUriOpener" => {
			dev_log!("url", "url:registerExternalUriOpener");
			Ok(Value::Null)
		},

		// =====================================================================
		// Encryption
		// =====================================================================
		"encryption:encrypt" => {
			dev_log!("encryption", "encryption:encrypt");
			Ok(json!(""))
		},
		"encryption:decrypt" => {
			dev_log!("encryption", "encryption:decrypt");
			Ok(json!(""))
		},

		// =====================================================================
		// Extension host starter
		// =====================================================================
		"extensionHostStarter:createExtensionHost" => {
			dev_log!("exthost", "extensionHostStarter:createExtensionHost");
			Ok(json!({ "id": "1" }))
		},
		"extensionHostStarter:start" => {
			dev_log!("exthost", "extensionHostStarter:start pid={}", std::process::id());
			Ok(json!({ "pid": std::process::id() }))
		},
		"extensionHostStarter:kill" => {
			dev_log!("exthost", "extensionHostStarter:kill");
			Ok(Value::Null)
		},
		"extensionHostStarter:getExitInfo" => {
			dev_log!("exthost", "extensionHostStarter:getExitInfo");
			Ok(json!({ "code": null, "signal": null }))
		},

		// =====================================================================
		// Extension host debug service
		// =====================================================================
		"extensionhostdebugservice:reload" => {
			dev_log!("exthost", "extensionhostdebugservice:reload");
			Ok(Value::Null)
		},
		"extensionhostdebugservice:close" => {
			dev_log!("exthost", "extensionhostdebugservice:close");
			Ok(Value::Null)
		},

		// =====================================================================
		// Workspaces - additional commands
		// =====================================================================
		"workspaces:getRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:getRecentlyOpened");
			Ok(json!({
				"workspaces": [],
				"files": []
			}))
		},
		"workspaces:removeRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:removeRecentlyOpened");
			Ok(Value::Null)
		},
		"workspaces:addRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:addRecentlyOpened");
			Ok(Value::Null)
		},
		"workspaces:clearRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:clearRecentlyOpened");
			Ok(Value::Null)
		},
		"workspaces:enterWorkspace" => {
			dev_log!("workspaces", "workspaces:enterWorkspace");
			Ok(Value::Null)
		},
		"workspaces:createUntitledWorkspace" => {
			dev_log!("workspaces", "workspaces:createUntitledWorkspace");
			Ok(Value::Null)
		},
		"workspaces:deleteUntitledWorkspace" => {
			dev_log!("workspaces", "workspaces:deleteUntitledWorkspace");
			Ok(Value::Null)
		},
		"workspaces:getWorkspaceIdentifier" => Ok(Value::Null),
		"workspaces:getDirtyWorkspaces" => Ok(json!([])),

		// Default handler for unknown commands
		_ => {
			error!("[WindServiceHandlers] Unknown IPC command: {}", command);
			Err(format!("Unknown IPC command: {}", command))
		},
	}
}

/// Handler for configuration get requests
async fn handle_configuration_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	// Use Mountain's configuration system
	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	let value = provider
		.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|e| format!("Failed to get configuration: {}", e))?;

	dev_log!("config", "get: {} = {:?}", key, value);
	Ok(value)
}

/// Handler for configuration update requests
async fn handle_configuration_update(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let value = args.get(1).ok_or("Missing configuration value".to_string())?.clone();

	// Use Mountain's configuration system
	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	provider
		.UpdateConfigurationValue(
			key.to_string(),
			value,
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|e| format!("Failed to update configuration: {}", e))?;

	dev_log!("config", "updated: {}", key);
	Ok(Value::Null)
}

/// Handler for file read requests
async fn handle_file_read(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read file: {}", e))?;

	dev_log!("vfs", "read: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}

/// Handler for file write requests
async fn handle_file_write(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let content = args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content.as_bytes().to_vec(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write file: {}", e))?;

	dev_log!("vfs", "written: {} ({} bytes)", path, content.len());
	Ok(Value::Null)
}

/// Handler for file stat requests
async fn handle_file_stat(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let stats = provider
		.StatFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to stat file: {}", e))?;

	dev_log!("vfs", "legacy_stat: {}", path);
	Ok(json!(stats))
}

/// Handler for file exists requests
async fn handle_file_exists(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let exists = provider.StatFile(&PathBuf::from(path)).await.is_ok();

	dev_log!("vfs", "exists: {} = {}", path, exists);
	Ok(json!(exists))
}

/// Handler for file delete requests
async fn handle_file_delete(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Delete(&PathBuf::from(path), false, false)
		.await
		.map_err(|e:CommonError| format!("Failed to delete file: {}", e))?;

	dev_log!("vfs", "deleted: {}", path);
	Ok(Value::Null)
}

/// Handler for file copy requests
async fn handle_file_copy(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let source = args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let destination = args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Copy(&PathBuf::from(source), &PathBuf::from(destination), false)
		.await
		.map_err(|e:CommonError| format!("Failed to copy file: {} -> {}", source, destination))?;

	dev_log!("vfs", "copied: {} -> {}", source, destination);
	Ok(Value::Null)
}

/// Handler for file move requests
async fn handle_file_move(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let source = args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let destination = args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Rename(&PathBuf::from(source), &PathBuf::from(destination), false)
		.await
		.map_err(|e:CommonError| format!("Failed to move file: {} -> {}", source, destination))?;

	dev_log!("vfs", "moved: {} -> {}", source, destination);
	Ok(Value::Null)
}

/// Handler for directory creation requests
async fn handle_file_mkdir(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let recursive = args.get(1).and_then(|v| v.as_bool()).unwrap_or(true);

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.CreateDirectory(&PathBuf::from(path), recursive)
		.await
		.map_err(|e:CommonError| format!("Failed to create directory: {}", e))?;

	dev_log!("vfs", "mkdir: {} (recursive: {})", path, recursive);
	Ok(Value::Null)
}

/// Handler for directory reading requests
async fn handle_file_readdir(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let entries = provider
		.ReadDirectory(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read directory: {}", e))?;

	dev_log!("vfs", "readdir_legacy: {} ({} entries)", path, entries.len());
	Ok(json!(entries))
}

/// Handler for binary file read requests
async fn handle_file_read_binary(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read binary file: {}", e))?;

	dev_log!("vfs", "readBinary: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}

/// Handler for binary file write requests
async fn handle_file_write_binary(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let content = args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	// Convert string content to bytes
	let content_bytes = content.as_bytes().to_vec();
	let content_len = content_bytes.len();

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content_bytes.clone(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write binary file: {}", e))?;

	dev_log!("vfs", "writeBinary: {} ({} bytes)", path, content_len);
	Ok(Value::Null)
}

/// Handler for storage get requests
async fn handle_storage_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing storage key".to_string())?
		.as_str()
		.ok_or("Storage key must be a string".to_string())?;

	// Use Mountain's storage provider
	let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

	let value = provider
		.GetStorageValue(false, key)
		.await
		.map_err(|e| format!("Failed to get storage item: {}", e))?;

	dev_log!("storage", "get: {}", key);
	Ok(value.unwrap_or(Value::Null))
}

/// Handler for storage set requests
async fn handle_storage_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing storage key".to_string())?
		.as_str()
		.ok_or("Storage key must be a string".to_string())?;

	let value = args.get(1).ok_or("Missing storage value".to_string())?.clone();

	// Use Mountain's storage provider
	let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

	provider
		.UpdateStorageValue(false, key.to_string(), Some(value))
		.await
		.map_err(|e| format!("Failed to set storage item: {}", e))?;

	dev_log!("storage", "set: {}", key);
	Ok(Value::Null)
}

/// Handler for environment get requests
async fn handle_environment_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing environment key".to_string())?
		.as_str()
		.ok_or("Environment key must be a string".to_string())?;

	// Use std::env for environment variables
	let value = std::env::var(key).map_err(|e| format!("Failed to get environment variable: {}", e))?;

	dev_log!("config", "env_get: {}", key);
	Ok(json!(value))
}

/// Handler for showing items in folder
async fn handle_show_item_in_folder(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path_str = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// IMPLEMENTATION: Microsoft-inspired native file system integration
	dev_log!("vfs", "showInFolder: {}", path_str);

	let path = std::path::PathBuf::from(path_str);

	// Validate path exists
	if !path.exists() {
		return Err(format!("Path does not exist: {}", path_str));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		// Use macOS's open command with -R flag to reveal in Finder
		let result = Command::new("open")
			.arg("-R")
			.arg(&path)
			.output()
			.map_err(|e| format!("Failed to execute open command: {}", e))?;

		if !result.status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&result.stderr)
			));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		// Use Windows Explorer with /select flag
		let result = Command::new("explorer")
			.arg("/select,")
			.arg(&path)
			.output()
			.map_err(|e| format!("Failed to execute explorer command: {}", e))?;

		if !result.status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&result.stderr)
			));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		// Try common Linux file managers
		let file_managers = ["nautilus", "dolphin", "thunar", "pcmanfm", "nemo"];
		let mut last_error = String::new();

		for manager in file_managers.iter() {
			let result = Command::new(manager).arg(&path).output();

			match result {
				Ok(output) if output.status.success() => {
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

	dev_log!("vfs", "showed in folder: {}", path_str);
	Ok(Value::Bool(true))
}

/// Handler for opening external URLs
async fn handle_open_external(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let url_str = args
		.get(0)
		.ok_or("Missing URL".to_string())?
		.as_str()
		.ok_or("URL must be a string".to_string())?;

	// IMPLEMENTATION: Microsoft-inspired URL validation and opening
	dev_log!("lifecycle", "openExternal: {}", url_str);

	// Validate URL format
	if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
		return Err(format!("Invalid URL format. Must start with http:// or https://: {}", url_str));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		// Use macOS's open command
		let result = Command::new("open")
			.arg(url_str)
			.output()
			.map_err(|e| format!("Failed to execute open command: {}", e))?;

		if !result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&result.stderr)));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		// Use Windows start command
		let result = Command::new("cmd")
			.arg("/c")
			.arg("start")
			.arg(url_str)
			.output()
			.map_err(|e| format!("Failed to execute start command: {}", e))?;

		if !result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&result.stderr)));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		// Try common Linux URL handlers
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

/// Handler for workbench configuration requests
async fn handle_workbench_configuration(runtime:Arc<ApplicationRunTime>, _args:Vec<Value>) -> Result<Value, String> {
	// Get the complete workbench configuration
	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	let config = provider
		.GetConfigurationValue(None, ConfigurationOverridesDTO::default())
		.await
		.map_err(|e| format!("Failed to get workbench configuration: {}", e))?;

	dev_log!("config", "workbench config retrieved");
	Ok(config)
}

// ============================================================================
// Terminal Handlers
// ============================================================================

/// Create a new PTY terminal via TerminalProvider.
async fn handle_terminal_create(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let Options = args.first().cloned().unwrap_or(Value::Null);
	runtime
		.Environment
		.CreateTerminal(Options)
		.await
		.map_err(|Error| format!("terminal:create failed: {}", Error))
}

/// Write text to PTY stdin via TerminalProvider.
async fn handle_terminal_send_text(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:sendText requires terminal_id as first argument".to_string())?;
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	runtime
		.Environment
		.SendTextToTerminal(TerminalId, Text)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:sendText failed: {}", Error))
}

/// Dispose a terminal via TerminalProvider.
async fn handle_terminal_dispose(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:dispose requires terminal_id as first argument".to_string())?;

	runtime
		.Environment
		.DisposeTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:dispose failed: {}", Error))
}

/// Show a terminal in the UI.
async fn handle_terminal_show(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args.first().and_then(|V| V.as_u64()).unwrap_or(0);
	let PreserveFocus = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	runtime
		.Environment
		.ShowTerminal(TerminalId, PreserveFocus)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:show failed: {}", Error))
}

/// Hide a terminal.
async fn handle_terminal_hide(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args.first().and_then(|V| V.as_u64()).unwrap_or(0);

	runtime
		.Environment
		.HideTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:hide failed: {}", Error))
}

// ============================================================================
// Output Channel Handlers
// ============================================================================

/// Create a named output channel. Returns the channel name as its handle.
async fn handle_output_create(_app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("Output").to_string();
	dev_log!("ipc", "output:create channel='{}'", ChannelName);
	// Sky/frontend creates the channel panel on the `sky://output/create` event
	Ok(json!({ "channelName": ChannelName }))
}

/// Append text to an output channel.
async fn handle_output_append(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit("sky://output/append", json!({ "channel": ChannelName, "text": Text }));
	Ok(Value::Null)
}

/// Append a line to an output channel (text + newline).
async fn handle_output_append_line(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Line = format!("{}\n", Text);

	let _ = app_handle.emit("sky://output/append", json!({ "channel": ChannelName, "text": Line }));
	Ok(Value::Null)
}

/// Clear an output channel.
async fn handle_output_clear(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit("sky://output/clear", json!({ "channel": ChannelName }));
	Ok(Value::Null)
}

/// Show an output channel panel.
async fn handle_output_show(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit("sky://output/show", json!({ "channel": ChannelName }));
	Ok(Value::Null)
}

// ============================================================================
// TextFile Handlers
// ============================================================================

/// Read a text file from disk.
async fn handle_textfile_read(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Path = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:read requires path as first argument".to_string())?;

	tokio::fs::read_to_string(Path)
		.await
		.map(Value::String)
		.map_err(|Error| format!("textFile:read failed: {}", Error))
}

/// Write text to a file on disk.
async fn handle_textfile_write(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Path = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:write requires path as first argument".to_string())?;
	let Content = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	tokio::fs::write(Path, Content.as_bytes())
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("textFile:write failed: {}", Error))
}

/// Save a document - forward save intent to Sky frontend.
async fn handle_textfile_save(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	// Actual disk write happens via textFile:write; this is a UI-dirty-state hint.
	let _Uri = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	dev_log!("vfs", "textFile:save uri={:?}", _Uri);
	Ok(Value::Null)
}

/// Register all Wind IPC command handlers
pub fn register_wind_ipc_handlers(app_handle:&tauri::AppHandle) -> Result<(), String> {
	dev_log!("lifecycle", "registering IPC handlers");

	// Note: These handlers are automatically registered when included in the
	// Tauri invoke_handler macro in the main binary

	Ok(())
}

// ============================================================================
// Command Registry Handlers
// ============================================================================

/// Execute a command by ID, dispatching to Mountain's CommandExecutor.
async fn handle_commands_execute(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "commands:execute requires string command_id as first argument".to_string())?
		.to_string();

	let Argument = args.get(1).cloned().unwrap_or(Value::Null);

	dev_log!("ipc", "commands:execute id={}", CommandId);

	runtime
		.Environment
		.ExecuteCommand(CommandId, Argument)
		.await
		.map_err(|Error| format!("commands:execute failed: {}", Error))
}

/// Return all registered command IDs from Mountain's CommandRegistry.
async fn handle_commands_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Commands = runtime
		.Environment
		.GetAllCommands()
		.await
		.map_err(|Error| format!("commands:getAll failed: {}", Error))?;

	Ok(json!(Commands))
}

// ============================================================================
// Extension Host Handlers
// ============================================================================

/// Return metadata for all scanned extensions.
async fn handle_extensions_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = runtime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getAll failed: {}", Error))?;

	Ok(json!(Extensions))
}

/// Return metadata for a single extension by ID.
async fn handle_extensions_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Id = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:get requires string id as first argument".to_string())?
		.to_string();

	let Extension = runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:get failed: {}", Error))?;

	Ok(Extension.unwrap_or(Value::Null))
}

/// Check whether an extension is currently active (scanned and present).
async fn handle_extensions_is_active(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Id = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:isActive requires string id as first argument".to_string())?
		.to_string();

	let Extension = runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:isActive failed: {}", Error))?;

	Ok(json!(Extension.is_some()))
}

// ============================================================================
// Storage handlers
// ============================================================================

/// Delete a persistent storage key.
async fn handle_storage_delete(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Storage::StorageProvider::StorageProvider;

	let Key = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("storage:delete requires key as first argument".to_string())?
		.to_string();

	runtime
		.Environment
		.UpdateStorageValue(true, Key, None)
		.await
		.map_err(|Error| format!("storage:delete failed: {}", Error))?;

	Ok(Value::Null)
}

/// Return all storage keys.
async fn handle_storage_keys(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Storage::StorageProvider::StorageProvider;

	let Storage = runtime
		.Environment
		.GetAllStorage(true)
		.await
		.map_err(|Error| format!("storage:keys failed: {}", Error))?;

	let Keys:Vec<String> = Storage.as_object().map(|O| O.keys().cloned().collect()).unwrap_or_default();
	Ok(json!(Keys))
}

// ============================================================================
// Notification handlers
// ============================================================================

/// Show a notification message - emits sky://notification/show for Sky to
/// render.
async fn handle_notification_show(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Message = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Severity = args.get(1).and_then(|V| V.as_str()).unwrap_or("info").to_string();
	let Actions = args.get(2).cloned().unwrap_or(json!([]));

	let Id = format!(
		"notification-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		"sky://notification/show",
		json!({
			"id": Id,
			"message": Message,
			"severity": Severity,
			"actions": Actions,
		}),
	);

	Ok(json!(Id))
}

/// Begin a progress notification - emits sky://notification/progress-begin.
async fn handle_notification_show_progress(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Title = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		"sky://notification/progress-begin",
		json!({
			"id": Id,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Update an in-progress notification progress bar.
async fn handle_notification_update_progress(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		"sky://notification/progress-update",
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress notification.
async fn handle_notification_end_progress(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit("sky://notification/progress-end", json!({ "id": Id }));

	Ok(Value::Null)
}

// ============================================================================
// Progress handlers
// ============================================================================

/// Begin a window-level or status-bar progress indicator.
async fn handle_progress_begin(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Location = args.first().and_then(|V| V.as_str()).unwrap_or("notification").to_string();
	let Title = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		"sky://progress/begin",
		json!({
			"id": Id,
			"location": Location,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Report incremental progress on an active indicator.
async fn handle_progress_report(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		"sky://progress/report",
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress indicator.
async fn handle_progress_end(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit("sky://progress/end", json!({ "id": Id }));

	Ok(Value::Null)
}

// ============================================================================
// QuickInput handlers
// ============================================================================

/// Show a quick-pick dialog. Routes through UserInterfaceProvider (blocking
/// oneshot).
async fn handle_quick_input_show_quick_pick(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::{QuickPickItemDTO::QuickPickItemDTO, QuickPickOptionsDTO::QuickPickOptionsDTO},
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Items:Vec<QuickPickItemDTO> = args
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| {
			Arr.iter()
				.filter_map(|Item| {
					let Label = Item.get("label").and_then(|L| L.as_str()).unwrap_or("").to_string();
					let Description = Item.get("description").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Detail = Item.get("detail").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Picked = Item.get("picked").and_then(|P| P.as_bool()).unwrap_or(false);
					Some(QuickPickItemDTO { Label, Description, Detail, Picked:Some(Picked), AlwaysShow:Some(false) })
				})
				.collect()
		})
		.unwrap_or_default();

	let Options = QuickPickOptionsDTO {
		PlaceHolder:args
			.get(1)
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		CanPickMany:Some(
			args.get(1)
				.and_then(|V| V.get("canPickMany"))
				.and_then(|B| B.as_bool())
				.unwrap_or(false),
		),
		Title:args
			.get(1)
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),
		..Default::default()
	};

	let Result = runtime
		.Environment
		.ShowQuickPick(Items, Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showQuickPick failed: {}", Error))?;

	match Result {
		Some(Labels) => Ok(Labels.into_iter().next().map(|S| json!(S)).unwrap_or(Value::Null)),
		None => Ok(Value::Null),
	}
}

/// Show an input box dialog. Routes through UserInterfaceProvider (blocking
/// oneshot).
async fn handle_quick_input_show_input_box(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Opts = args.first();
	let Options = InputBoxOptionsDTO {
		Prompt:Opts
			.and_then(|V| V.get("prompt"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		PlaceHolder:Opts
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		IsPassword:Some(Opts.and_then(|V| V.get("password")).and_then(|B| B.as_bool()).unwrap_or(false)),
		Value:Opts
			.and_then(|V| V.get("value"))
			.and_then(|V| V.as_str())
			.map(|S| S.to_string()),
		Title:Opts
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),
		IgnoreFocusOut:None,
	};

	let Result = runtime
		.Environment
		.ShowInputBox(Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showInputBox failed: {}", Error))?;

	Ok(Result.map(|S| json!(S)).unwrap_or(Value::Null))
}

// ============================================================================
// Workspaces handlers
// ============================================================================

/// Return the current workspace folders.
async fn handle_workspaces_get_folders(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let Folders = Workspace.GetWorkspaceFolders();

	let FolderList:Vec<Value> = Folders
		.iter()
		.enumerate()
		.map(|(Index, Folder)| {
			json!({
				"uri": Folder.URI.to_string(),
				"name": Folder.Name,
				"index": Index,
			})
		})
		.collect();

	Ok(json!(FolderList))
}

/// Add a workspace folder.
async fn handle_workspaces_add_folder(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use url::Url;

	let UriStr = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:addFolder requires uri as first argument".to_string())?
		.to_string();

	let Name = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	let Index = Folders.len();
	let URI = Url::parse(&UriStr).map_err(|E| format!("workspaces:addFolder invalid URI: {}", E))?;
	if let Ok(Folder) = WorkspaceFolderStateDTO::New(URI, Name, Index) {
		Folders.push(Folder);
		Workspace.SetWorkspaceFolders(Folders);
	}

	Ok(Value::Null)
}

/// Remove a workspace folder by URI.
async fn handle_workspaces_remove_folder(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let UriStr = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:removeFolder requires uri as first argument".to_string())?
		.to_string();

	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	Folders.retain(|F| F.URI.to_string() != UriStr);
	for (I, F) in Folders.iter_mut().enumerate() {
		F.Index = I;
	}
	Workspace.SetWorkspaceFolders(Folders);

	Ok(Value::Null)
}

/// Return the workspace name (basename of root folder, or None if untitled).
async fn handle_workspaces_get_name(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Name = runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.GetDisplayName());

	Ok(Name.map(|N| json!(N)).unwrap_or(Value::Null))
}

// ============================================================================
// Themes handlers
// ============================================================================

/// Return the active color theme metadata from ConfigurationProvider.
async fn handle_themes_get_active(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	};

	let ThemeId = runtime
		.Environment
		.GetConfigurationValue(Some("workbench.colorTheme".to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("themes:getActive failed: {}", Error))?;

	let Id = ThemeId.as_str().unwrap_or("Default Dark Modern").to_string();

	// Infer kind from id string
	let Kind = if Id.to_lowercase().contains("light") {
		"light"
	} else if Id.to_lowercase().contains("high contrast light") {
		"highContrastLight"
	} else if Id.to_lowercase().contains("high contrast") {
		"highContrast"
	} else {
		"dark"
	};

	Ok(json!({ "id": Id, "label": Id, "kind": Kind }))
}

/// Return installed theme extensions.
async fn handle_themes_list(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	// For now return a hardcoded set of built-in themes; extensions contribute
	// more.
	let Themes = vec![
		json!({ "id": "Default Dark Modern", "label": "Default Dark Modern", "kind": "dark" }),
		json!({ "id": "Default Light Modern", "label": "Default Light Modern", "kind": "light" }),
		json!({ "id": "Default Dark+", "label": "Default Dark+", "kind": "dark" }),
		json!({ "id": "Default Light+", "label": "Default Light+", "kind": "light" }),
		json!({ "id": "High Contrast", "label": "High Contrast", "kind": "highContrast" }),
		json!({ "id": "High Contrast Light", "label": "High Contrast Light", "kind": "highContrastLight" }),
	];

	Ok(json!(Themes))
}

/// Set the active color theme by updating ConfigurationProvider.
async fn handle_themes_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
	};
	use tauri::Emitter;

	let ThemeId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("themes:set requires themeId as first argument".to_string())?
		.to_string();

	runtime
		.Environment
		.UpdateConfigurationValue(
			"workbench.colorTheme".to_string(),
			json!(ThemeId),
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|Error| format!("themes:set failed: {}", Error))?;

	let _ = runtime
		.Environment
		.ApplicationHandle
		.emit("sky://theme/change", json!({ "themeId": ThemeId }));

	Ok(Value::Null)
}

// ============================================================================
// Search handlers
// ============================================================================

/// Search text across all workspace files (line-by-line grep, max 1000
/// results).
async fn handle_search_find_in_files(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use std::path::PathBuf;

	use globset::GlobBuilder;
	use tokio::fs;

	let Pattern = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("search:findInFiles requires pattern".to_string())?
		.to_string();
	let IsRegex = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);
	let IsCaseSensitive = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);
	let _IsWordMatch = args.get(3).and_then(|V| V.as_bool()).unwrap_or(false);
	let IncludeGlob = args.get(4).and_then(|V| V.as_str()).unwrap_or("**").to_string();
	let ExcludeGlob = args.get(5).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let MaxResults = args.get(6).and_then(|V| V.as_u64()).unwrap_or(1000) as usize;

	let WorkspaceFolders = runtime.Environment.ApplicationState.Workspace.GetWorkspaceFolders();

	if WorkspaceFolders.is_empty() {
		return Ok(json!([]));
	}

	let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));

	// Build include matcher
	let IncludeMatcher = GlobBuilder::new(&IncludeGlob)
		.literal_separator(false)
		.build()
		.map(|G| G.compile_matcher())
		.ok();

	// Build exclude matcher
	let ExcludeMatcher = if !ExcludeGlob.is_empty() {
		GlobBuilder::new(&ExcludeGlob)
			.literal_separator(false)
			.build()
			.map(|G| G.compile_matcher())
			.ok()
	} else {
		None
	};

	let SearchText = Pattern.clone();
	let mut Matches = Vec::new();

	// Walk directory recursively
	let mut Stack = vec![RootPath.clone()];
	while let Some(Dir) = Stack.pop() {
		let mut Entries = match fs::read_dir(&Dir).await {
			Ok(E) => E,
			Err(_) => continue,
		};

		while let Ok(Some(Entry)) = Entries.next_entry().await {
			let Path = Entry.path();
			let RelPath = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();

			// Skip hidden dirs
			if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
				continue;
			}

			if Path.is_dir() {
				Stack.push(Path);
				continue;
			}

			// Check include/exclude globs
			if let Some(Ref) = &IncludeMatcher {
				if !Ref.is_match(&RelPath) {
					continue;
				}
			}
			if let Some(Ref) = &ExcludeMatcher {
				if Ref.is_match(&RelPath) {
					continue;
				}
			}

			// Read file and search line by line
			let Content = match fs::read_to_string(&Path).await {
				Ok(C) => C,
				Err(_) => continue,
			};

			for (LineIndex, Line) in Content.lines().enumerate() {
				let Hit = if IsRegex {
					// Simple contains fallback (no regex crate available here)
					Line.contains(&SearchText)
				} else if IsCaseSensitive {
					Line.contains(&SearchText)
				} else {
					Line.to_lowercase().contains(&SearchText.to_lowercase())
				};

				if Hit {
					let Uri = format!("file://{}", Path.to_string_lossy());
					Matches.push(json!({
						"uri": Uri,
						"lineNumber": LineIndex + 1,
						"preview": Line.trim(),
					}));

					if Matches.len() >= MaxResults {
						return Ok(json!(Matches));
					}
				}
			}
		}
	}

	Ok(json!(Matches))
}

/// Search file paths by glob pattern in workspace.
async fn handle_search_find_files(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use std::path::PathBuf;

	use globset::GlobBuilder;
	use tokio::fs;

	let Pattern = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("search:findFiles requires pattern".to_string())?
		.to_string();
	let MaxResults = args.get(1).and_then(|V| V.as_u64()).unwrap_or(500) as usize;

	let WorkspaceFolders = runtime.Environment.ApplicationState.Workspace.GetWorkspaceFolders();

	if WorkspaceFolders.is_empty() {
		return Ok(json!([]));
	}

	let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));

	let Matcher = GlobBuilder::new(&Pattern)
		.literal_separator(false)
		.build()
		.map(|G| G.compile_matcher())
		.map_err(|Error| format!("Invalid glob pattern: {}", Error))?;

	let mut Files = Vec::new();
	let mut Stack = vec![RootPath.clone()];

	while let Some(Dir) = Stack.pop() {
		let mut Entries = match fs::read_dir(&Dir).await {
			Ok(E) => E,
			Err(_) => continue,
		};

		while let Ok(Some(Entry)) = Entries.next_entry().await {
			let Path = Entry.path();

			if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
				continue;
			}

			if Path.is_dir() {
				Stack.push(Path);
				continue;
			}

			let RelPath = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();

			if Matcher.is_match(&RelPath) {
				Files.push(format!("file://{}", Path.to_string_lossy()));

				if Files.len() >= MaxResults {
					return Ok(json!(Files));
				}
			}
		}
	}

	Ok(json!(Files))
}

// ============================================================================
// Decorations handlers
// ============================================================================

/// Return the decoration (badge, tooltip, color) for a single URI.
/// Mountain holds decorations in ApplicationState; extensions push them via
/// the `decorations:set` IPC channel.
async fn handle_decorations_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:get requires uri".to_string())?;
	let Decoration = runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri);
	Ok(Decoration.unwrap_or(Value::Null))
}

/// Return decorations for multiple URIs in a single round-trip.
async fn handle_decorations_get_many(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uris:Vec<String> = args
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| Arr.iter().filter_map(|U| U.as_str().map(str::to_owned)).collect())
		.unwrap_or_default();

	let mut Result = serde_json::Map::new();
	for Uri in &Uris {
		if let Some(Decoration) = runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
			Result.insert(Uri.clone(), Decoration);
		}
	}
	Ok(Value::Object(Result))
}

/// Register or override the decoration for a URI.
async fn handle_decorations_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:set requires uri".to_string())?;
	let Decoration = args.get(1).cloned().unwrap_or(Value::Null);
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Decorations
		.SetDecoration(Uri, Decoration);
	Ok(Value::Null)
}

/// Remove the decoration for a URI.
async fn handle_decorations_clear(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:clear requires uri".to_string())?;
	runtime.Environment.ApplicationState.Feature.Decorations.ClearDecoration(Uri);
	Ok(Value::Null)
}

// ============================================================================
// WorkingCopy handlers
// ============================================================================

/// Check whether a URI has unsaved changes.
async fn handle_working_copy_is_dirty(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:isDirty requires uri".to_string())?;
	let IsDirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.IsDirty(Uri);
	Ok(json!(IsDirty))
}

/// Mark a URI as dirty (unsaved) or clean.
async fn handle_working_copy_set_dirty(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;
	let Dirty = args.get(1).and_then(|V| V.as_bool()).unwrap_or(true);
	runtime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);
	Ok(Value::Null)
}

/// Return all URIs that currently have unsaved changes.
async fn handle_working_copy_get_all_dirty(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Dirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetAllDirty();
	Ok(json!(Dirty))
}

/// Return the count of resources with unsaved changes.
async fn handle_working_copy_get_dirty_count(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Count = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetDirtyCount();
	Ok(json!(Count))
}

// ============================================================================
// Keybinding handlers
// ============================================================================

/// Register a dynamic keybinding in Mountain's keybinding registry.
async fn handle_keybinding_add(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires commandId".to_string())?
		.to_owned();
	let KeyExpression = args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires keybinding".to_string())?
		.to_owned();
	let When = args.get(2).and_then(|V| V.as_str()).map(str::to_owned);
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.AddKeybinding(CommandId, KeyExpression, When);
	Ok(Value::Null)
}

/// Remove all dynamic keybindings for a command.
async fn handle_keybinding_remove(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:remove requires commandId".to_string())?;
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.RemoveKeybinding(CommandId);
	Ok(Value::Null)
}

/// Look up the keybinding string for a command.
async fn handle_keybinding_lookup(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:lookup requires commandId".to_string())?;
	let Binding = runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.LookupKeybinding(CommandId);
	Ok(Binding.map(|B| json!(B)).unwrap_or(Value::Null))
}

/// Return all registered dynamic keybindings.
async fn handle_keybinding_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = runtime.Environment.ApplicationState.Feature.Keybindings.GetAllKeybindings();
	Ok(json!(All))
}

// ============================================================================
// Lifecycle handlers
// ============================================================================

/// Return the current application lifecycle phase (1–4).
async fn handle_lifecycle_get_phase(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Phase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	Ok(json!(Phase))
}

/// Wait (poll) until the application reaches at least the requested phase.
/// Returns immediately if the phase has already been reached.
async fn handle_lifecycle_when_phase(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let RequestedPhase = args.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;
	let CurrentPhase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	if CurrentPhase >= RequestedPhase {
		return Ok(Value::Null);
	}
	// Simple poll with short sleep - production should use a channel/notify
	let mut Retries = 0u8;
	loop {
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		let Phase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
		if Phase >= RequestedPhase || Retries >= 50 {
			break;
		}
		Retries += 1;
	}
	Ok(Value::Null)
}

/// Initiate a graceful application shutdown via Tauri.
async fn handle_lifecycle_request_shutdown(app_handle:AppHandle) -> Result<Value, String> {
	app_handle.exit(0);
	Ok(Value::Null)
}

// ============================================================================
// Navigation History Handlers
// ============================================================================

/// Navigate backward in the editor history stack.
async fn handle_history_go_back(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = runtime.Environment.ApplicationState.Feature.NavigationHistory.GoBack();
	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}

/// Navigate forward in the editor history stack.
async fn handle_history_go_forward(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = runtime.Environment.ApplicationState.Feature.NavigationHistory.GoForward();
	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}

/// Return whether backward navigation is available.
async fn handle_history_can_go_back(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Ok(Value::Bool(
		runtime.Environment.ApplicationState.Feature.NavigationHistory.CanGoBack(),
	))
}

/// Return whether forward navigation is available.
async fn handle_history_can_go_forward(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Ok(Value::Bool(
		runtime.Environment.ApplicationState.Feature.NavigationHistory.CanGoForward(),
	))
}

/// Push a URI onto the navigation history stack.
async fn handle_history_push(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("history:push requires uri".to_string())?
		.to_owned();

	runtime.Environment.ApplicationState.Feature.NavigationHistory.Push(Uri);
	Ok(Value::Null)
}

/// Clear the entire navigation history stack.
async fn handle_history_clear(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	runtime.Environment.ApplicationState.Feature.NavigationHistory.Clear();
	Ok(Value::Null)
}

/// Return the full navigation history stack as an array of URI strings.
async fn handle_history_get_stack(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Stack = runtime.Environment.ApplicationState.Feature.NavigationHistory.GetStack();
	Ok(Value::Array(Stack.into_iter().map(Value::String).collect()))
}

// ============================================================================
// Label Handlers
// ============================================================================

/// Resolve a human-readable display label for a URI.
///
/// Args: [uri: string, relative: bool]
/// Returns: string label
async fn handle_label_get_uri(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getUri requires uri".to_string())?
		.to_owned();

	let Relative = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	if !Relative {
		// Absolute: strip file:// scheme if present, return raw path
		let Label = if Uri.starts_with("file://") {
			Uri.trim_start_matches("file://").to_owned()
		} else {
			Uri.clone()
		};
		return Ok(Value::String(Label));
	}

	// Relative: make path relative to workspace root if possible
	let WorkspaceRoot = runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.URI.to_string())
		.unwrap_or_default();

	let RawPath = if Uri.starts_with("file://") {
		Uri.trim_start_matches("file://").to_owned()
	} else {
		Uri.clone()
	};

	let RootPath = if WorkspaceRoot.starts_with("file://") {
		WorkspaceRoot.trim_start_matches("file://").to_owned()
	} else {
		WorkspaceRoot
	};

	let Label = if !RootPath.is_empty() && RawPath.starts_with(&RootPath) {
		RawPath[RootPath.len()..].trim_start_matches('/').to_owned()
	} else {
		RawPath
	};

	Ok(Value::String(Label))
}

/// Return the display label for the current workspace root folder.
async fn handle_label_get_workspace(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Label = runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| {
			if !F.Name.is_empty() {
				F.Name
			} else {
				F.URI
					.path_segments()
					.and_then(|mut S| S.next_back())
					.map(|S| S.to_owned())
					.unwrap_or_else(|| F.URI.to_string())
			}
		})
		.unwrap_or_default();

	Ok(Value::String(Label))
}

/// Return only the basename (filename + extension) of a URI.
async fn handle_label_get_base(args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getBase requires uri".to_string())?;

	let Base = Uri.split('/').next_back().unwrap_or(Uri);
	Ok(Value::String(Base.to_owned()))
}

// ============================================================================
// Model (Text Model Registry) Handlers
// ============================================================================

/// Open a text model: read content from disk and register in DocumentState.
/// Returns { uri, content, version, languageId }.
async fn handle_model_open(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:open requires uri".to_string())?
		.to_owned();

	// Derive file path from URI
	let FilePath = if Uri.starts_with("file://") {
		Uri.trim_start_matches("file://").to_owned()
	} else {
		Uri.clone()
	};

	// Read file content from disk
	let Content = tokio::fs::read_to_string(&FilePath).await.unwrap_or_default();

	// Detect language from extension
	let LanguageId = std::path::Path::new(&FilePath)
		.extension()
		.and_then(|E| E.to_str())
		.map(|Ext| {
			match Ext {
				"rs" => "rust",
				"ts" | "tsx" => "typescript",
				"js" | "jsx" | "mjs" | "cjs" => "javascript",
				"json" | "jsonc" => "json",
				"toml" => "toml",
				"yaml" | "yml" => "yaml",
				"md" => "markdown",
				"html" | "htm" => "html",
				"css" | "scss" | "less" => "css",
				"sh" | "bash" | "zsh" => "shellscript",
				"py" => "python",
				"go" => "go",
				"c" | "h" => "c",
				"cpp" | "cc" | "cxx" | "hpp" => "cpp",
				_ => "plaintext",
			}
		})
		.unwrap_or("plaintext")
		.to_owned();

	// Determine next version (1 if new, increment if exists)
	let Version = runtime
		.Environment
		.ApplicationState
		.Feature
		.Documents
		.Get(&Uri)
		.map(|D| D.Version + 1)
		.unwrap_or(1);

	// Register in document state
	{
		use crate::ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO;

		if let Ok(ParsedUri) = url::Url::parse(&Uri) {
			let Lines:Vec<String> = Content.lines().map(|L| L.to_owned()).collect();
			let Eol = if Content.contains("\r\n") { "\r\n" } else { "\n" }.to_owned();

			let Document = DocumentStateDTO {
				URI:ParsedUri,
				LanguageIdentifier:LanguageId.clone(),
				Version,
				Lines,
				EOL:Eol,
				IsDirty:false,
				Encoding:"utf-8".to_owned(),
				VersionIdentifier:Version,
			};

			runtime
				.Environment
				.ApplicationState
				.Feature
				.Documents
				.AddOrUpdate(Uri.clone(), Document);
		}
	}

	Ok(json!({
		"uri": Uri,
		"content": Content,
		"version": Version,
		"languageId": LanguageId,
	}))
}

/// Close a text model and remove it from DocumentState.
async fn handle_model_close(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:close requires uri".to_string())?;

	runtime.Environment.ApplicationState.Feature.Documents.Remove(Uri);
	Ok(Value::Null)
}

/// Get the current snapshot of an open text model, or null if not open.
async fn handle_model_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:get requires uri".to_string())?;

	match runtime.Environment.ApplicationState.Feature.Documents.Get(Uri) {
		None => Ok(Value::Null),
		Some(Document) => {
			Ok(json!({
				"uri": Uri,
				"content": Document.Lines.join(&Document.EOL),
				"version": Document.Version,
				"languageId": Document.LanguageIdentifier,
			}))
		},
	}
}

/// Return all currently open text models.
async fn handle_model_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = runtime
		.Environment
		.ApplicationState
		.Feature
		.Documents
		.GetAll()
		.into_iter()
		.map(|(Uri, Document)| {
			json!({
				"uri": Uri,
				"content": Document.Lines.join(&Document.EOL),
				"version": Document.Version,
				"languageId": Document.LanguageIdentifier,
			})
		})
		.collect::<Vec<_>>();

	Ok(Value::Array(All))
}

/// Update the content of an open text model, incrementing its version.
async fn handle_model_update_content(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires uri".to_string())?
		.to_owned();

	let NewContent = args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("model:updateContent requires content".to_string())?
		.to_owned();

	let (NewVersion, LanguageId) = match runtime.Environment.ApplicationState.Feature.Documents.Get(&Uri) {
		None => return Err(format!("model:updateContent - model not open: {}", Uri)),
		Some(mut Document) => {
			Document.Version += 1;
			Document.Lines = NewContent.lines().map(|L| L.to_owned()).collect();
			Document.IsDirty = true;
			let Version = Document.Version;
			let LangId = Document.LanguageIdentifier.clone();
			runtime
				.Environment
				.ApplicationState
				.Feature
				.Documents
				.AddOrUpdate(Uri.clone(), Document);
			(Version, LangId)
		},
	};

	Ok(json!({
		"uri": Uri,
		"content": NewContent,
		"version": NewVersion,
		"languageId": LanguageId,
	}))
}

// =============================================================================
// Native file system handlers (use extract_path_from_arg for URI
// deserialization)
// =============================================================================

/// Read file with URI arg support (VS Code sends { scheme, path } objects)
/// Returns { buffer: number[] } where buffer is the raw byte content.
/// VS Code's DiskFileSystemProviderClient wraps this with VSBuffer.wrap().
async fn handle_file_read_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	dev_log!("vfs", "readFile: {}", Path);

	// Read as raw bytes (not string) to preserve binary content
	let Bytes = tokio::fs::read(&Path)
		.await
		.map_err(|E| format!("Failed to read file: {} (path: {})", E, Path))?;

	dev_log!("vfs", "readFile OK: {} ({} bytes)", Path, Bytes.len());

	// Return as { buffer: [byte, byte, ...] } - VS Code reconstructs as VSBuffer
	// The buffer field must be an array of u8 values for proper deserialization
	let ByteArray:Vec<Value> = Bytes.iter().map(|B| json!(*B)).collect();
	Ok(json!({ "buffer": ByteArray }))
}

/// Write file with URI arg support
async fn handle_file_write_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	// args[1] is VSBuffer (content), args[2] is options
	let Content = args.get(1).ok_or("Missing file content")?;

	let Bytes = if let Some(S) = Content.as_str() {
		S.as_bytes().to_vec()
	} else if let Some(Obj) = Content.as_object() {
		// VSBuffer wraps { buffer: Uint8Array } - extract bytes
		if let Some(Buf) = Obj.get("buffer") {
			if let Some(Arr) = Buf.as_array() {
				Arr.iter().filter_map(|V| V.as_u64().map(|N| N as u8)).collect()
			} else if let Some(S) = Buf.as_str() {
				S.as_bytes().to_vec()
			} else {
				return Err("Unsupported buffer format".to_string());
			}
		} else {
			serde_json::to_string(Content).unwrap_or_default().into_bytes()
		}
	} else {
		return Err("File content must be a string or VSBuffer".to_string());
	};

	// Ensure parent directory exists
	if let Some(Parent) = std::path::Path::new(&Path).parent() {
		tokio::fs::create_dir_all(Parent).await.ok();
	}

	tokio::fs::write(&Path, &Bytes)
		.await
		.map_err(|E| format!("Failed to write file: {} (path: {})", E, Path))?;

	Ok(Value::Null)
}

/// Rename/move file with URI arg support
async fn handle_file_rename_native(args:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(args.get(0).ok_or("Missing source path")?)?;
	let Target = extract_path_from_arg(args.get(1).ok_or("Missing target path")?)?;

	tokio::fs::rename(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to rename: {} -> {} ({})", Source, Target, E))?;

	Ok(Value::Null)
}

/// Resolve real path (follow symlinks)
async fn handle_file_realpath(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing path")?)?;

	let Canonical = tokio::fs::canonicalize(&Path)
		.await
		.map_err(|E| format!("Failed to realpath: {} ({})", Path, E))?;

	Ok(json!({
		"scheme": "file",
		"path": Canonical.to_string_lossy(),
		"authority": ""
	}))
}

/// Clone file (copy with metadata)
async fn handle_file_clone_native(args:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(args.get(0).ok_or("Missing source path")?)?;
	let Target = extract_path_from_arg(args.get(1).ok_or("Missing target path")?)?;

	tokio::fs::copy(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to clone: {} -> {} ({})", Source, Target, E))?;

	Ok(Value::Null)
}

// =============================================================================
// Native host handlers
// =============================================================================

/// Pick folder using Tauri dialog plugin and reload webview with folder param.
/// In Electron, pickFolderAndOpen causes the main process to reload the window
/// with the new workspace. We replicate this by navigating the webview to the
/// same origin with `?folder=<path>`, which ResolveConfiguration reads.
async fn handle_native_pick_folder(app_handle:AppHandle, _args:Vec<Value>) -> Result<Value, String> {
	use tauri_plugin_dialog::DialogExt;
	use tauri::WebviewWindow;

	dev_log!("folder", "pickFolderAndOpen requested");

	dev_log!("folder", "pickFolderAndOpen requested");

	let Handle = app_handle.clone();
	tokio::task::spawn_blocking(move || {
		let FolderPath = Handle.dialog().file().blocking_pick_folder();

		if let Some(Path) = FolderPath {
			let PathStr = Path.to_string();
			dev_log!("folder", "picked: {}", PathStr);

			// Navigate the webview to reload with the folder as workspace.
			// This mirrors Electron's behaviour of reloading the renderer.
			if let Some(Window) = Handle.get_webview_window("main") {
				if let Ok(CurrentUrl) = Window.url() {
					let Origin = CurrentUrl.origin().unicode_serialization();
					let EncodedPath = url::form_urlencoded::Serializer::new(String::new())
						.append_pair("folder", &PathStr)
						.finish();
					let NewUrl = format!("{}/?{}", Origin, EncodedPath);
					dev_log!("folder", "navigating: {}", NewUrl);
					let _ = Window.navigate(NewUrl.parse().unwrap());
				}
			}
		}
	});

	Ok(Value::Null)
}

/// Show open dialog with file/folder picker
async fn handle_native_show_open_dialog(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	dev_log!("folder", "showOpenDialog: {:?}", args);
	// Return canceled for now - real dialog integration needs tauri_plugin_dialog
	Ok(json!({ "canceled": true, "filePaths": [] }))
}

/// Get OS properties - cross-platform (macOS, Windows, Linux)
async fn handle_native_os_properties() -> Result<Value, String> {
	use sysinfo::System;

	let OsType = match std::env::consts::OS {
		"macos" => "Darwin",
		"windows" => "Windows_NT",
		"linux" => "Linux",
		_ => std::env::consts::OS,
	};

	// Get OS release version
	let Release = {
		#[cfg(target_os = "macos")]
		{
			std::process::Command::new("sw_vers")
				.arg("-productVersion")
				.output()
				.ok()
				.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_string())
				.unwrap_or_else(|| "14.0".to_string())
		}
		#[cfg(target_os = "windows")]
		{
			// Windows 10/11 version from registry or ver command
			std::process::Command::new("cmd")
				.args(["/c", "ver"])
				.output()
				.ok()
				.map(|O| {
					let Output = String::from_utf8_lossy(&O.stdout);
					// Extract version number from "Microsoft Windows [Version 10.0.22631.4890]"
					Output
						.split('[')
						.nth(1)
						.and_then(|S| S.split(']').next())
						.and_then(|S| S.strip_prefix("Version "))
						.unwrap_or("10.0.0")
						.to_string()
				})
				.unwrap_or_else(|| "10.0.0".to_string())
		}
		#[cfg(target_os = "linux")]
		{
			// Linux kernel version from uname -r
			std::process::Command::new("uname")
				.arg("-r")
				.output()
				.ok()
				.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_string())
				.unwrap_or_else(|| "6.1.0".to_string())
		}
		#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
		{
			"0.0.0".to_string()
		}
	};

	// CPU info via sysinfo
	let mut Sys = System::new();
	Sys.refresh_cpu_all();
	let Cpus:Vec<Value> = Sys
		.cpus()
		.iter()
		.map(|Cpu| {
			json!({
				"model": Cpu.brand(),
				"speed": Cpu.frequency()
			})
		})
		.collect();

	Ok(json!({
		"type": OsType,
		"release": Release,
		"arch": std::env::consts::ARCH,
		"platform": std::env::consts::OS,
		"cpus": Cpus
	}))
}

/// Get OS statistics - cross-platform memory/load stats
async fn handle_native_os_statistics() -> Result<Value, String> {
	use sysinfo::System;

	let mut Sys = System::new();
	Sys.refresh_memory();

	let TotalMem = Sys.total_memory();
	let FreeMem = Sys.available_memory();

	// Load average: available on Unix, not on Windows
	let LoadAvg = {
		#[cfg(unix)]
		{
			let Load = System::load_average();
			vec![Load.one, Load.five, Load.fifteen]
		}
		#[cfg(not(unix))]
		{
			vec![0.0, 0.0, 0.0]
		}
	};

	Ok(json!({
		"totalmem": TotalMem,
		"freemem": FreeMem,
		"loadavg": LoadAvg
	}))
}

/// Check if window is fullscreen
async fn handle_native_is_fullscreen(app_handle:AppHandle) -> Result<Value, String> {
	use tauri::Manager;
	let Window = app_handle.get_webview_window("main");
	if let Some(W) = Window {
		Ok(json!(W.is_fullscreen().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}

/// Check if window is maximized
async fn handle_native_is_maximized(app_handle:AppHandle) -> Result<Value, String> {
	use tauri::Manager;
	let Window = app_handle.get_webview_window("main");
	if let Some(W) = Window {
		Ok(json!(W.is_maximized().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}

/// Find a free port starting from a given port
async fn handle_native_find_free_port(args:Vec<Value>) -> Result<Value, String> {
	let StartPort = args.get(0).and_then(|V| V.as_u64()).unwrap_or(9000) as u16;

	for Port in StartPort..StartPort + 100 {
		if std::net::TcpListener::bind(("127.0.0.1", Port)).is_ok() {
			return Ok(json!(Port));
		}
	}
	Ok(json!(0))
}

// =============================================================================
// Local PTY handlers
// =============================================================================

/// Detect available terminal profiles - cross-platform
async fn handle_local_pty_get_profiles() -> Result<Value, String> {
	let mut Profiles = Vec::new();

	#[cfg(unix)]
	{
		let DefaultShell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

		// Common Unix shells - macOS, Ubuntu, RHEL, Fedora, Arch, etc.
		let UnixShells = [
			"/bin/zsh",
			"/bin/bash",
			"/bin/sh",
			"/usr/bin/zsh",
			"/usr/bin/bash",
			"/usr/bin/fish",
			"/usr/local/bin/fish",
			"/usr/local/bin/zsh",
			"/usr/local/bin/bash",
			"/bin/dash",     // Ubuntu/Debian default /bin/sh symlink target
			"/usr/bin/ksh",  // KornShell (RHEL, Solaris)
			"/usr/bin/tcsh", // C Shell variant
			"/bin/csh",      // C Shell
			"/usr/bin/pwsh", // PowerShell on Linux/macOS
			"/usr/local/bin/pwsh",
		];

		for Shell in &UnixShells {
			if std::path::Path::new(Shell).exists() {
				let Name = std::path::Path::new(Shell)
					.file_name()
					.and_then(|N| N.to_str())
					.unwrap_or("shell");

				Profiles.push(json!({
					"profileName": Name,
					"path": Shell,
					"isDefault": *Shell == DefaultShell.as_str(),
					"args": [],
					"env": {},
					"icon": "terminal"
				}));
			}
		}

		// Also check /etc/shells for additional entries
		if let Ok(ShellsFile) = std::fs::read_to_string("/etc/shells") {
			for Line in ShellsFile.lines() {
				let Trimmed = Line.trim();
				if Trimmed.starts_with('/') && !Trimmed.starts_with('#') {
					let AlreadyAdded = Profiles.iter().any(|P| P.get("path").and_then(|V| V.as_str()) == Some(Trimmed));
					if !AlreadyAdded && std::path::Path::new(Trimmed).exists() {
						let Name = std::path::Path::new(Trimmed)
							.file_name()
							.and_then(|N| N.to_str())
							.unwrap_or("shell");

						Profiles.push(json!({
							"profileName": Name,
							"path": Trimmed,
							"isDefault": Trimmed == DefaultShell.as_str(),
							"args": [],
							"env": {},
							"icon": "terminal"
						}));
					}
				}
			}
		}
	}

	#[cfg(target_os = "windows")]
	{
		// Windows terminal profiles
		let SystemRoot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
		let ProgramFiles = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
		let LocalAppData =
			std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\Users\\User\\AppData\\Local".to_string());

		let WindowsShells:Vec<(&str, String, Vec<&str>)> = vec![
			(
				"PowerShell",
				format!("{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", SystemRoot),
				vec!["-NoLogo"],
			),
			(
				"PowerShell 7",
				format!("{}\\PowerShell\\7\\pwsh.exe", ProgramFiles),
				vec!["-NoLogo"],
			),
			("Command Prompt", format!("{}\\System32\\cmd.exe", SystemRoot), vec![]),
			(
				"Git Bash",
				format!("{}\\Git\\bin\\bash.exe", ProgramFiles),
				vec!["--login", "-i"],
			),
			(
				"Git Bash (User)",
				format!("{}\\Programs\\Git\\bin\\bash.exe", LocalAppData),
				vec!["--login", "-i"],
			),
			("WSL", format!("{}\\System32\\wsl.exe", SystemRoot), vec![]),
			("MSYS2", "C:\\msys64\\usr\\bin\\bash.exe".to_string(), vec!["--login", "-i"]),
			("Cygwin", "C:\\cygwin64\\bin\\bash.exe".to_string(), vec!["--login", "-i"]),
		];

		let mut IsFirstFound = true;
		for (Name, Path, Args) in &WindowsShells {
			if std::path::Path::new(Path).exists() {
				Profiles.push(json!({
					"profileName": Name,
					"path": Path,
					"isDefault": IsFirstFound,
					"args": Args,
					"env": {},
					"icon": "terminal"
				}));
				IsFirstFound = false;
			}
		}
	}

	Ok(json!(Profiles))
}

/// Get default system shell - cross-platform
async fn handle_local_pty_get_default_shell() -> Result<Value, String> {
	#[cfg(unix)]
	{
		let Shell = std::env::var("SHELL").unwrap_or_else(|_| {
			// Try common fallbacks
			for Path in &["/bin/zsh", "/bin/bash", "/bin/sh"] {
				if std::path::Path::new(Path).exists() {
					return Path.to_string();
				}
			}
			"/bin/sh".to_string()
		});
		Ok(json!(Shell))
	}

	#[cfg(target_os = "windows")]
	{
		let SystemRoot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
		// Check for PowerShell 7 first, then Windows PowerShell, then cmd
		let PwshPath = format!("{}\\PowerShell\\7\\pwsh.exe", std::env::var("ProgramFiles").unwrap_or_default());
		if std::path::Path::new(&PwshPath).exists() {
			return Ok(json!(PwshPath));
		}
		Ok(json!(format!(
			"{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
			SystemRoot
		)))
	}

	#[cfg(not(any(unix, target_os = "windows")))]
	{
		Ok(json!("/bin/sh"))
	}
}

/// Get terminal environment
async fn handle_local_pty_get_environment() -> Result<Value, String> {
	let Env:HashMap<String, String> = std::env::vars().collect();
	Ok(json!(Env))
}

/// Detect OS color scheme - cross-platform dark mode detection
async fn handle_native_get_color_scheme() -> Result<Value, String> {
	let Dark = detect_dark_mode();
	// High contrast detection
	let HighContrast = {
		#[cfg(target_os = "windows")]
		{
			// Windows: check SystemParametersInfo for HIGH_CONTRAST
			std::process::Command::new("reg")
				.args(["query", "HKCU\\Control Panel\\Accessibility\\HighContrast", "/v", "Flags"])
				.output()
				.ok()
				.map(|O| {
					let Output = String::from_utf8_lossy(&O.stdout);
					// Flag 1 = HCF_HIGHCONTRASTON
					Output.contains("0x1") || Output.contains("REG_DWORD    1")
				})
				.unwrap_or(false)
		}
		#[cfg(not(target_os = "windows"))]
		{
			// macOS/Linux: high contrast not natively detectable the same way
			// GTK: gsettings get org.gnome.desktop.a11y.interface high-contrast
			#[cfg(target_os = "linux")]
			{
				std::process::Command::new("gsettings")
					.args(["get", "org.gnome.desktop.a11y.interface", "high-contrast"])
					.output()
					.ok()
					.map(|O| String::from_utf8_lossy(&O.stdout).trim() == "true")
					.unwrap_or(false)
			}
			#[cfg(not(target_os = "linux"))]
			{
				false
			}
		}
	};

	Ok(json!({ "dark": Dark, "highContrast": HighContrast }))
}

/// Cross-platform dark mode detection
fn detect_dark_mode() -> bool {
	#[cfg(target_os = "macos")]
	{
		// macOS: defaults read -g AppleInterfaceStyle returns "Dark" if dark mode
		std::process::Command::new("defaults")
			.args(["read", "-g", "AppleInterfaceStyle"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_lowercase().contains("dark"))
			.unwrap_or(false)
	}

	#[cfg(target_os = "windows")]
	{
		// Windows: HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\
		// AppsUseLightTheme 0 = dark, 1 = light
		std::process::Command::new("reg")
			.args([
				"query",
				"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
				"/v",
				"AppsUseLightTheme",
			])
			.output()
			.ok()
			.map(|O| {
				let Output = String::from_utf8_lossy(&O.stdout);
				Output.contains("0x0") || Output.contains("REG_DWORD    0")
			})
			.unwrap_or(false)
	}

	#[cfg(target_os = "linux")]
	{
		// Linux: Try multiple approaches
		// 1. GTK theme (GNOME, Ubuntu, Fedora, etc.)
		let GtkDark = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "color-scheme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).contains("dark"))
			.unwrap_or(false);

		if GtkDark {
			return true;
		}

		// 2. GTK theme name contains "dark"
		let GtkTheme = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "gtk-theme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).to_lowercase().contains("dark"))
			.unwrap_or(false);

		if GtkTheme {
			return true;
		}

		// 3. KDE/Plasma
		let KdeDark = std::env::var("KDE_COLOR_SCHEME")
			.ok()
			.map(|V| V.to_lowercase().contains("dark"))
			.unwrap_or(false);

		if KdeDark {
			return true;
		}

		// 4. xfce4
		let XfceDark = std::process::Command::new("xfconf-query")
			.args(["-c", "xsettings", "-p", "/Net/ThemeName"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).to_lowercase().contains("dark"))
			.unwrap_or(false);

		XfceDark
	}

	#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
	{
		false
	}
}

// =============================================================================
// Native file system handlers (stat, exists, delete, mkdir, readdir)
// =============================================================================

/// Stat file - pure stat, no side effects. Returns IStat shape.
async fn handle_file_stat_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	dev_log!("vfs", "stat: {}", Path);

	let Metadata = tokio::fs::symlink_metadata(&Path).await.map_err(|E| {
		dev_log!("vfs", "stat ENOENT: {}", Path);
		format!("Failed to stat file: {} (path: {})", E, Path)
	})?;

	dev_log!("vfs", "stat OK: {} (dir={})", Path, Metadata.is_dir());
	Ok(metadata_to_istat(&Metadata))
}

/// Check file existence with URI arg support
async fn handle_file_exists_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	Ok(json!(tokio::fs::try_exists(&Path).await.unwrap_or(false)))
}

/// Delete file or directory with URI arg support
async fn handle_file_delete_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	// Options may include { recursive, useTrash }
	let Recursive = args
		.get(1)
		.and_then(|V| V.as_object())
		.and_then(|O| O.get("recursive"))
		.and_then(|V| V.as_bool())
		.unwrap_or(false);

	let PathBuf = std::path::Path::new(&Path);

	if PathBuf.is_dir() {
		if Recursive {
			tokio::fs::remove_dir_all(&Path).await
		} else {
			tokio::fs::remove_dir(&Path).await
		}
	} else {
		tokio::fs::remove_file(&Path).await
	}
	.map_err(|E| format!("Failed to delete: {} ({})", Path, E))?;

	Ok(Value::Null)
}

/// Create directory with URI arg support
async fn handle_file_mkdir_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing directory path")?)?;

	tokio::fs::create_dir_all(&Path)
		.await
		.map_err(|E| format!("Failed to mkdir: {} ({})", Path, E))?;

	Ok(Value::Null)
}

/// Read directory contents with URI arg support
/// Returns array of [name, fileType] tuples matching VS Code's ReadDirResult
async fn handle_file_readdir_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing directory path")?)?;

	dev_log!("vfs", "readdir: {}", Path);

	let mut Entries = tokio::fs::read_dir(&Path)
		.await
		.map_err(|E| format!("Failed to readdir: {} ({})", Path, E))?;

	let mut Result = Vec::new();

	while let Some(Entry) = Entries.next_entry().await.map_err(|E| E.to_string())? {
		let Name = Entry.file_name().to_string_lossy().to_string();
		let FileType = Entry.file_type().await.map_err(|E| E.to_string())?;

		let TypeValue = if FileType.is_symlink() {
			64 // SymbolicLink
		} else if FileType.is_dir() {
			2 // Directory
		} else {
			1 // File
		};

		Result.push(json!([Name, TypeValue]));
	}

	Ok(json!(Result))
}

// =============================================================================
// Storage handlers (VS Code NativeWorkbenchStorageService)
// =============================================================================

/// Get all storage items as [key, value] tuples.
/// VS Code's NativeWorkbenchStorageService calls this on initialization.
async fn handle_storage_get_items(runtime:Arc<ApplicationRunTime>, _args:Vec<Value>) -> Result<Value, String> {
	let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

	match provider.GetAllStorage(true).await {
		Ok(State) => {
			// Convert JSON object to array of [key, value] tuples
			if let Some(Obj) = State.as_object() {
				let Tuples:Vec<Value> = Obj
					.iter()
					.map(|(K, V)| {
						let ValStr = match V {
							Value::String(S) => S.clone(),
							_ => V.to_string(),
						};
						json!([K, ValStr])
					})
					.collect();
				Ok(json!(Tuples))
			} else {
				Ok(json!([]))
			}
		},
		Err(_) => Ok(json!([])),
	}
}

/// Update storage items. VS Code sends { insert, delete } where:
/// - insert: Array of [key, value] tuples or Map<string, string>
/// - delete: Array of keys to remove
async fn handle_storage_update_items(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

	if let Some(Updates) = args.get(0).and_then(|V| V.as_object()) {
		// Handle inserts
		if let Some(Inserts) = Updates.get("insert") {
			if let Some(Arr) = Inserts.as_array() {
				for Item in Arr {
					if let Some(Pair) = Item.as_array() {
						if let (Some(Key), Some(Val)) = (Pair.get(0).and_then(|V| V.as_str()), Pair.get(1)) {
							let _ = provider.UpdateStorageValue(true, Key.to_string(), Some(Val.clone())).await;
						}
					}
				}
			} else if let Some(Obj) = Inserts.as_object() {
				for (Key, Val) in Obj {
					let _ = provider.UpdateStorageValue(true, Key.clone(), Some(Val.clone())).await;
				}
			}
		}

		// Handle deletes
		if let Some(Deletes) = Updates.get("delete").and_then(|V| V.as_array()) {
			for Key in Deletes {
				if let Some(K) = Key.as_str() {
					let _ = provider.UpdateStorageValue(true, K.to_string(), None).await;
				}
			}
		}
	}

	Ok(Value::Null)
}
