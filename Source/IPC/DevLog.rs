//! # DevLog — Tag-filtered development logging
//!
//! Controlled by `LAND_DEV_LOG` environment variable.
//!
//! ## Usage
//! ```bash
//! LAND_DEV_LOG=vfs,ipc ./Mountain          # only VFS + IPC
//! LAND_DEV_LOG=all ./Mountain              # everything
//! LAND_DEV_LOG=folder,config ./Mountain    # folder picker + config
//! ./Mountain                               # nothing (only normal log!() output)
//! ```
//!
//! ## Tags
//! - `vfs`       — file stat, read, write, readdir, path resolution
//! - `ipc`       — MountainIPCInvoke routing, channel dispatch
//! - `config`    — configuration loading, environment paths
//! - `lifecycle` — startup, shutdown, dir creation
//! - `storage`   — StorageProvider read/write/persist
//! - `exthost`   — extension host starter, cocoon
//! - `folder`    — folder picker, workspace open, navigation
//! - `logger`    — VS Code logger channel commands

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
