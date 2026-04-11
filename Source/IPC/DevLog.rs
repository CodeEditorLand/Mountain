//! # DevLog - Tag-filtered development logging
//!
//! Controlled by `LAND_DEV_LOG` environment variable.
//! The same tags work in both Mountain (Rust) and Wind/Sky (TypeScript).
//!
//! ## Usage
//! ```bash
//! LAND_DEV_LOG=vfs,ipc ./Mountain          # only VFS + IPC
//! LAND_DEV_LOG=all ./Mountain              # everything
//! LAND_DEV_LOG=short ./Mountain            # everything, compressed + deduped
//! LAND_DEV_LOG=terminal,exthost ./Mountain # terminal + extension host
//! ./Mountain                               # nothing (only normal log!() output)
//! ```
//!
//! ## Short Mode
//!
//! `LAND_DEV_LOG=short` enables all tags but compresses output:
//! - Long app-data paths aliased to `$APP`
//! - Consecutive duplicate messages counted (`(x14)` suffix)
//! - Rust log targets compressed (`D::Binary::Main::Entry` → `Entry`)
//!
//! ## Tags (38 granular tags across all Elements)
//!
//! | Tag           | Scope                                               |
//! |---------------|-----------------------------------------------------|
//! | `vfs`         | File stat, read, write, readdir, mkdir, delete, copy|
//! | `ipc`         | IPC routing: invoke dispatch, channel calls          |
//! | `config`      | Configuration get/set, env paths, workbench config   |
//! | `lifecycle`   | Startup, shutdown, phases, window events             |
//! | `storage`     | Storage get/set/delete, items, optimize              |
//! | `folder`      | Folder picker, workspace navigation                  |
//! | `exthost`     | Extension host: create, start, kill, exit info       |
//! | `extensions`  | Extension scanning, activation, management           |
//! | `terminal`    | Terminal/PTY: create, sendText, profiles, shell      |
//! | `search`      | Search: findFiles, findInFiles                       |
//! | `themes`      | Theme: list, get active, set                         |
//! | `window`      | Window: focus, maximize, minimize, fullscreen        |
//! | `nativehost`  | OS integration: process, devtools, shell             |
//! | `clipboard`   | Clipboard: read/write text, buffer, image            |
//! | `commands`    | Command registry: execute, getAll                    |
//! | `model`       | Text model: open, close, get, updateContent          |
//! | `output`      | Output channels: create, append, show                |
//! | `notification`| Notifications: show, progress                        |
//! | `progress`    | Progress: begin, end, report                         |
//! | `quickinput`  | Quick input: showQuickPick, showInputBox             |
//! | `workingcopy` | Working copy: dirty state                            |
//! | `workspaces`  | Workspace: folders, recent, enter                    |
//! | `keybinding`  | Keybindings: add, remove, lookup                     |
//! | `label`       | Label service: getBase, getUri                       |
//! | `history`     | Navigation history: push, goBack, goForward          |
//! | `decorations` | Decorations: get, set, clear                         |
//! | `textfile`    | Text file operations: read, write, save              |
//! | `update`      | Update service: check, download, apply               |
//! | `encryption`  | Encryption: encrypt, decrypt                         |
//! | `menubar`     | Menubar updates                                      |
//! | `url`         | URL handler: registerExternalUriOpener               |
//! | `grpc`        | gRPC/Vine: server, client, connections               |
//! | `cocoon`      | Cocoon sidecar: spawn, health, handshake             |
//! | `bootstrap`   | Effect-TS bootstrap stages                           |
//! | `preload`     | Preload: globals, polyfills, ipcRenderer             |

use std::sync::{Mutex, OnceLock};

static ENABLED_TAGS:OnceLock<Vec<String>> = OnceLock::new();
static SHORT_MODE:OnceLock<bool> = OnceLock::new();

// ── Path alias ──────────────────────────────────────────────────────────
// The app-data directory name is absurdly long. In short mode, alias it.
static APP_DATA_PREFIX:OnceLock<Option<String>> = OnceLock::new();

fn DetectAppDataPrefix() -> Option<String> {
	// Match the bundle identifier pattern used by Mountain
	if let Ok(Home) = std::env::var("HOME") {
		let Base = format!("{}/Library/Application Support", Home);
		if let Ok(Entries) = std::fs::read_dir(&Base) {
			for Entry in Entries.flatten() {
				let Name = Entry.file_name();
				let Name = Name.to_string_lossy();
				if Name.starts_with("land.editor.") && Name.contains("mountain") {
					return Some(format!("{}/{}", Base, Name));
				}
			}
		}
	}
	None
}

/// Get the app-data path prefix for aliasing (cached).
pub fn AppDataPrefix() -> &'static Option<String> { APP_DATA_PREFIX.get_or_init(DetectAppDataPrefix) }

/// Replace the long app-data path with `$APP` in a string.
pub fn AliasPath(Input:&str) -> String {
	if let Some(Prefix) = AppDataPrefix() {
		Input.replace(Prefix.as_str(), "$APP")
	} else {
		Input.to_string()
	}
}

// ── Dedup buffer ────────────────────────────────────────────────────────

pub struct DedupState {
	pub LastKey:String,
	pub Count:u64,
}

pub static DEDUP:Mutex<DedupState> = Mutex::new(DedupState { LastKey:String::new(), Count:0 });

/// Flush the dedup buffer - prints the pending count if > 1.
pub fn FlushDedup() {
	if let Ok(mut State) = DEDUP.lock() {
		if State.Count > 1 {
			eprintln!("  (x{})", State.Count);
		}
		State.LastKey.clear();
		State.Count = 0;
	}
}

// ── Tag resolution ──────────────────────────────────────────────────────

fn EnabledTags() -> &'static Vec<String> {
	ENABLED_TAGS.get_or_init(|| {
		match std::env::var("LAND_DEV_LOG") {
			Ok(Val) => Val.split(',').map(|S| S.trim().to_lowercase()).collect(),
			Err(_) => vec![],
		}
	})
}

/// Whether `LAND_DEV_LOG=short` is active.
pub fn IsShort() -> bool { *SHORT_MODE.get_or_init(|| EnabledTags().iter().any(|T| T == "short")) }

/// Check if a tag is enabled.
pub fn IsEnabled(Tag:&str) -> bool {
	let Tags = EnabledTags();
	if Tags.is_empty() {
		return false;
	}
	let Lower = Tag.to_lowercase();
	Tags.iter().any(|T| T == "all" || T == "short" || T == Lower.as_str())
}

/// Log a tagged dev message. Only prints if the tag is enabled via
/// LAND_DEV_LOG.
///
/// In `short` mode: aliases long paths, deduplicates consecutive identical
/// lines.
#[macro_export]
macro_rules! dev_log {
	($Tag:expr, $($Arg:tt)*) => {
		if $crate::IPC::DevLog::IsEnabled($Tag) {
			let RawMessage = format!($($Arg)*);
			let TagUpper = $Tag.to_uppercase();
			if $crate::IPC::DevLog::IsShort() {
				let Aliased = $crate::IPC::DevLog::AliasPath(&RawMessage);
				let Key = format!("{}:{}", TagUpper, Aliased);
				let ShouldPrint = {
					if let Ok(mut State) = $crate::IPC::DevLog::DEDUP.lock() {
						if State.LastKey == Key {
							State.Count += 1;
							false
						} else {
							let PrevCount = State.Count;
							let HadPrev = !State.LastKey.is_empty();
							State.LastKey = Key;
							State.Count = 1;
							if HadPrev && PrevCount > 1 {
								eprintln!("  (x{})", PrevCount);
							}
							true
						}
					} else {
						true
					}
				};
				if ShouldPrint {
					eprintln!("[DEV:{}] {}", TagUpper, Aliased);
				}
			} else {
				eprintln!("[DEV:{}] {}", TagUpper, RawMessage);
			}
		}
	};
}
