#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Canonical userdata base directory (Tauri `app_data_dir`) + first-access
//! scaffolding. Seeded by `AppLifecycle::Dirs` so every `/User/...` URI the
//! renderer emits lands under the bundle-identifier-qualified Application
//! Support path VS Code's profile system expects.

use crate::dev_log;

/// Canonical userdata base directory, set once from Tauri's PathResolver.
static USERDATA_BASE_DIR:std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Lazy-init flag - `ensure_userdata_dirs` is idempotent; the flag skips
/// the directory walk after the first successful pass.
static USERDATA_INITIALIZED:std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_userdata_base_dir(Path:String) { let _ = USERDATA_BASE_DIR.set(Path); }

pub fn get_userdata_base_dir() -> String {
	if let Some(Dir) = USERDATA_BASE_DIR.get() {
		return Dir.clone();
	}

	if let Ok(Home) = std::env::var("HOME") {
		#[cfg(target_os = "macos")]
		return format!("{}/Library/Application Support/Land", Home);

		#[cfg(target_os = "linux")]
		return format!("{}/.local/share/Land", Home);
	}

	"/tmp/Land".to_string()
}

pub fn ensure_userdata_dirs() {
	if USERDATA_INITIALIZED.swap(true, std::sync::atomic::Ordering::Relaxed) {
		return;
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
