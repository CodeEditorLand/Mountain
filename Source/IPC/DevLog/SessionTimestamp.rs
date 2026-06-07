//! Local-time session timestamp (`%Y%m%dT%H%M%S`) cached once
//! per process. Must agree with
//! `WindServiceHandlers::nativeHost:getEnvironmentPaths` so the
//! Mountain dev log and VS Code's `window<N>/output_*` log
//! land in the same session directory.

use std::sync::OnceLock;

pub fn Fn() -> String {

	static STAMP:OnceLock<String> = OnceLock::new();

	STAMP
		.get_or_init(|| chrono::Local::now().format("%Y%m%dT%H%M%S").to_string())
		.clone()
}
