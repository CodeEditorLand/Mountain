//! # DevLog — Tag-filtered development logging
//!
//! Controlled by `LAND_DEV_LOG` environment variable.
//! The same tags work in both Mountain (Rust) and Wind/Sky (TypeScript).
//!
//! ## Usage
//! ```bash
//! LAND_DEV_LOG=vfs,ipc ./Mountain          # only VFS + IPC
//! LAND_DEV_LOG=all ./Mountain              # everything
//! LAND_DEV_LOG=terminal,exthost ./Mountain # terminal + extension host
//! ./Mountain                               # nothing (only normal log!() output)
//! ```
//!
//! Browser console: `window.__LAND_DEV_LOG = "config,vfs"`
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

use std::sync::OnceLock;

static ENABLED_TAGS: OnceLock<Vec<String>> = OnceLock::new();

fn EnabledTags() -> &'static Vec<String> {
	ENABLED_TAGS.get_or_init(|| {
		match std::env::var("LAND_DEV_LOG") {
			Ok(Val) => Val.split(',').map(|S| S.trim().to_lowercase()).collect(),
			Err(_) => vec![],
		}
	})
}

/// Check if a tag is enabled.
pub fn IsEnabled(Tag: &str) -> bool {
	let Tags = EnabledTags();
	if Tags.is_empty() {
		return false;
	}
	Tags.iter().any(|T| T == "all" || T == Tag.to_lowercase().as_str())
}

/// Log a tagged dev message. Only prints if the tag is enabled via LAND_DEV_LOG.
#[macro_export]
macro_rules! dev_log {
	($Tag:expr, $($Arg:tt)*) => {
		if $crate::IPC::DevLog::IsEnabled($Tag) {
			eprintln!("[DEV:{}] {}", $Tag.to_uppercase(), format!($($Arg)*));
		}
	};
}
