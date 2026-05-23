#![allow(unused_variables, dead_code, unused_imports)]

//! First-access scaffolding: creates userdata directories and default files.
//! Idempotent - the atomic flag skips the walk after the first pass.

use crate::dev_log;

pub fn Fn() {
	if super::INITIALIZED.swap(true, std::sync::atomic::Ordering::Relaxed) {
		return;
	}

	let Base = super::Get::Fn();

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
