//! Snapshot the Mountain process environment as a `HashMap`.
//! Inherited by every PTY spawned through `TerminalCreate`. Includes
//! the keys merged in by `EnhanceShellEnvironment` at boot, so a
//! Finder-launched `.app` exposes the user's interactive shell PATH
//! / NVM_DIR / HOMEBREW_PREFIX / … to terminals it spawns.

use std::collections::HashMap;

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
	let Env:HashMap<String, String> = std::env::vars().collect();

	Ok(json!(Env))
}
