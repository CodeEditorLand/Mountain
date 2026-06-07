//! Returns the userdata base directory, falling back to platform defaults.

pub fn Fn() -> String {

	if let Some(Dir) = super::BASE_DIR.get() {
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
